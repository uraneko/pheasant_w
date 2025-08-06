use std::collections::HashSet;


use chrono::TimeDelta;
use pheasant_uri::{ OriginSet, Origin};
use pheasant_core::{Cors, Method};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse::{Error as ParseError, Parse, ParseBuffer, ParseStream, Result as PRes};
use syn::parse_macro_input;
use syn::{Ident, ItemFn, Lit, Meta, MetaNameValue, Token, bracketed, token::Bracket};

#[derive(Debug)]
pub struct Resource {
    route: String,
}

impl Resource {
    pub fn route(self) -> String {
        self.route
    }
}

impl Parse for Resource {
    fn parse(s: ParseStream) -> PRes<Self> {
        Ok(Self {
            route: str_lit(Lit::parse(s)?)?,
        })
    }
}

// extract mime type
pub fn mime(fun: &mut ItemFn) -> Option<String> {
    let Some(idx) = fun
        .attrs
        .iter()
        .map(|a| a.path())
        .position(|p| p.get_ident() == Some(&Ident::new("mime", Span::call_site())))
    else {
        return None;
    };

    if let Lit::Str(sl) = fun.attrs.remove(idx).parse_args::<Lit>().unwrap() {
        Some(sl.value())
    } else {
        None
    }
}

// extract redirections
pub fn re(fun: &mut ItemFn) -> Option<Vec<String>> {
    let Some(idx) = fun
        .attrs
        .iter()
        .map(|a| a.path())
        .position(|p| p.get_ident() == Some(&Ident::new("re", Span::call_site())))
    else {
        return None;
    };

    let attr = fun.attrs.remove(idx);
    let Meta::List(ml) = attr.meta else {
        return None;
    };

    ml.parse_args::<StrVec>().map(|r| r.0).ok()
}

pub fn cors(fun: &mut ItemFn) -> Option<CorsAttr> {
    let Some(idx) = fun
        .attrs
        .iter()
        .map(|a| a.path())
        .position(|p| p.get_ident() == Some(&Ident::new("cors", Span::call_site())))
    else {
        return None;
    };
    let attr = fun.attrs.remove(idx);
    let Meta::List(ml) = attr.meta else {
        return None;
    };

    ml.parse_args::<CorsAttr>().ok()
}

fn parse_strings<'a, 'b>(s: ParseStream<'a>, cors_field: &mut HashSet<String> ) -> PRes<()> 
where 'a : 'b
{
    if s.peek(Bracket) {
        let content;
        _ = bracketed!(content in s);
        cors_field.extend(
            content
                .parse_terminated(Lit::parse, Token![,])?
                .into_iter()
                .map(|l| str_lit(l))
                .filter(|res| res.is_ok())
                .map(|res| res.unwrap()),
        );
    } else {
        cors_field.insert(str_lit(Lit::parse(s).unwrap())?);
    }

    Ok(())
}

fn parse_origins<'a, 'b>(s: ParseStream<'a>, cors: &mut Cors ) -> PRes<()> 
where 'a : 'b
{
    if s.peek(Bracket) {
        let content;
        _ = bracketed!(content in s);

        let Some(origins) = cors.origins() else {
            panic!("Cors::default should start with origins = whitelist");
        };
        origins.extend(
            content
                .parse_terminated(Lit::parse, Token![,])?
                .into_iter()
                .map(|l| str_lit(l))
                .filter(|res| res.is_ok())
                .map(|res| serde_json::from_str::<Origin>(&res.unwrap()))
            .filter(|ori| ori.is_ok()).map(|ori| ori.unwrap()),
        );
    } else {
        let str_url = str_lit(Lit::parse(s)?)?;
        println!("origins");
        let origins = str_url.parse::<OriginSet>().unwrap();
        println!("origins");
        cors.overwrite_origins(origins);
    }



    Ok(())
}



fn parse_methods<'a, 'b>(s: ParseStream<'a>, methods: &mut HashSet<Method>, ) -> PRes<()> 
where 'a : 'b
{
    if s.peek(Bracket) {
        let content;
        _ = bracketed!(content in s);
        methods.extend(
            content
                .parse_terminated(Ident::parse, Token![,])?
                .into_iter()
                .map(|i| i.to_string().parse::<Method>())
                .filter(|res| res.is_ok())
                .map(|res| res.unwrap()),
        );
    } else {
        methods.insert(Ident::parse(s)?.to_string().parse::<Method>().unwrap());
    }

    Ok(())
}

fn parse_duration(s: ParseStream, cors: &mut Cors, ) -> PRes<()> {
    let lit = Lit::parse(s)?;
    let int = match lit {
        Lit::Int(il) => TimeDelta::new(il.base10_parse::<i64>()?, 0),
        Lit::Str(sl) => {
            let s = sl.value();
                let t = s[..s.len() - 1].parse::<i64>().unwrap();

            if s.ends_with('w') {
                TimeDelta::try_weeks(t)
            } else if s.ends_with('d') {
                TimeDelta::try_days(t)
            } else if s.ends_with('h') {
                TimeDelta::try_hours(t)
            } else if s.ends_with('m') {
                TimeDelta::try_minutes(t)
            } else if s.ends_with('s') {
                TimeDelta::try_seconds(t)
            } else {
                panic!("badly formatted #[cors] attr max_age value");
            }
        }
        lit => panic!("badly formatted max_age value, expected Lit::Str or Lit::Int, got {:?}", lit)
    };

    let Some(int) = int else {
                    return Err(ParseError::new(Span::call_site(), "max_age value out of bounds"));
    };

    cors.update_max_age(int);

    Ok(())
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CorsAttr(Cors);

impl Parse for CorsAttr {
    fn parse(s: ParseStream) -> PRes<Self> {
        let mut cors = Cors::default();
        while !s.is_empty() {
            _ = <Token![,]>::parse(s);
            let ident = Ident::parse(s)?;
            _ = <Token![=]>::parse(s)?;

            let key = ident.to_string();
            match &key[..] {
                "methods" => 
                    parse_methods(s, cors.methods() )?,
                "headers" => parse_strings(s, cors.headers())?,
                "expose" => {
                    cors.alloc_expose();
                     let Some(expose) = cors.expose() else {
                        panic!("expose was initialized 1 line ago")
                    };
                    parse_strings(s, expose)?;
                }
                "origins" => {
                    // let Some(origins) = cors.origins() else {
                    //     panic!("Cors::default should start with origins = whitelist");
                    // };
                    parse_origins(s, &mut cors)?
                },
                "max_age" => parse_duration(s, &mut cors)?,
                key => panic!("unexpected attr key: {}", key),
            }
        }

        Ok(CorsAttr(cors))
    }
}

#[derive(Debug)]
struct StrVec(Vec<String>);

impl Parse for StrVec {
    fn parse(s: ParseStream) -> PRes<Self> {
        let mut v = vec![];
        while let Ok(Lit::Str(sl)) = Lit::parse(s) {
            v.push(sl.value());
            if !s.is_empty() {
                <Token![,]>::parse(s)?;
            }
        }

        Ok(Self(v))
    }
}

fn str_lit(s: Lit) -> Result<String, syn::parse::Error> {
    let Lit::Str(sl) = s else {
        return Err(syn::parse::Error::new(
            Span::call_site(),
            "wrong lit variant, expected str",
        ));
    };

    Ok(sl.value())
}





fn int_lit<T>(lit: Lit) -> Result<T, syn::parse::Error>
where
    T: std::str::FromStr + std::ops::Shl + std::ops::Mul,
    T::Err: std::fmt::Display,
{
    let Lit::Int(il) = lit else {
        return Err(syn::parse::Error::new(
            Span::call_site(),
            "wrong lit variant, expected int",
        ));
    };

    il.base10_parse::<T>()
}
