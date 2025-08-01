use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;

use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::parse_macro_input;
use syn::{Attribute, Expr, Ident, ItemFn, Lit, LitInt, LitStr, Token};

use quote::quote;

struct ErrorCode(u16);

impl Parse for ErrorCode {
    fn parse(s: ParseStream) -> PRes<Self> {
        Ok(Self({
            let Ok(Lit::Int(li)) = Lit::parse(s) else {
                return Err(syn::parse::Error::new(
                    Span::call_site(),
                    "wrong lit variant, expected int",
                ));
            };

            li.base10_parse::<u16>()?
        }))
    }
}

impl ErrorCode {
    fn code(&self) -> u16 {
        self.0
    }
}

// extract mime type
fn mime(fun: &mut ItemFn) -> Option<String> {
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

#[proc_macro_attribute]
pub fn error_code(attr: TokenStream, fun: TokenStream) -> TokenStream {
    // let [attr, fun]: [TS2; 2] = [attr.into(), fun.into()];
    // println!("{}\n{}", quote! {#attr}, quote! { #fun  });

    let resou = parse_macro_input!(attr as ErrorCode);
    let mut fun = parse_macro_input!(fun as ItemFn);
    let mime = mime(&mut fun);

    let funs = generate_errora(resou.code(), mime, fun);

    quote! {
        #funs
    }
    .into()
}

fn gen_mime(mime: Option<String>) -> TS2 {
    if let Some(mime) = mime {
        quote! { #mime }
    } else {
        quote! { "" }
    }
}

// suffixes an ident with the passed &str value
fn suffix_fn_ident(fun: &mut ItemFn, suffix: &str) {
    let i = fun.sig.ident.to_string();
    fun.sig.ident = Ident::new(&(i + suffix), Span::call_site());
}

fn suffix_ident(i: &Ident, suffix: &str) -> Ident {
    Ident::new(&(i.to_string() + suffix), Span::call_site())
}

fn clone_ident(fun: &ItemFn) -> Ident {
    fun.sig.ident.clone()
}

fn ref_ident(fun: &ItemFn) -> &Ident {
    &fun.sig.ident
}



fn generate_errora(ec: u16, mime: Option<String>,mut fun: ItemFn) -> TS2 {
    let error_fun = clone_ident(&fun);
    suffix_fn_ident(&mut fun, "_fail");
    let user_fun = ref_ident(&fun);

    let mime = gen_mime(mime);
    let vis = &fun.vis;

    quote! {
        #fun

        fn #error_fun() -> pheasant_core::Fail {
            pheasant_core::Fail::new(pheasant_core::ErrorStatus::from(#ec), #mime, #user_fun)
        }
    }
}
