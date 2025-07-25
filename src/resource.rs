use proc_macro::TokenStream;

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::parse_macro_input;
use syn::{Expr, ExprArray, Ident, ItemFn, Lit, Meta, Token, punctuated::Punctuated, token::Comma};

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
            route: {
                let Ok(Lit::Str(sl)) = Lit::parse(s) else {
                    return Err(syn::parse::Error::new(
                        Span::call_site(),
                        "wrong lit variant, expected str",
                    ));
                };

                sl.value()
            },
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

    ml.parse_args::<Redirects>().map(|r| r.0).ok()
}

#[derive(Debug)]
struct Redirects(Vec<String>);

impl Parse for Redirects {
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
