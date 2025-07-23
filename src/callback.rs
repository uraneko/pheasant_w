use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::{Attribute, Block, Expr, ExprClosure, Ident, ItemFn, Lit, LitStr, Signature};
use syn::{Token, bracketed};

pub fn mime(ifn: &mut ItemFn) -> Option<String> {
    if let Some(idx) = ifn
        .attrs
        .iter()
        .map(|a| a.path())
        .position(|p| p.get_ident() == Some(&Ident::new("mime", Span::call_site())))
    {
        if let Lit::Str(sl) = ifn.attrs.remove(idx).parse_args::<Lit>().unwrap() {
            Some(sl.value())
        } else {
            None
        }
    } else {
        None
    }
}

fn fn_ident(fun: &ItemFn) -> Ident {
    fun.sig.ident.clone()
}

fn suffix_ident(fun: &mut ItemFn) {
    let i = fun.sig.ident.to_string();
    fun.sig.ident = Ident::new(&(i + "_service"), Span::call_site());
}

pub fn wrapper_fn(route: String, mime: Option<String>, mut fun: ItemFn) -> TS2 {
    let ident = fn_ident(&fun);
    suffix_ident(&mut fun);
    let suffixed = fn_ident(&fun);

    let mime = if let Some(mime) = mime {
        quote! { #mime }
    } else {
        quote! { "" }
    };

    quote! {
        #fun

        fn #ident() -> Service {
            Service::new(Method::Get, #route, #mime, #suffixed)
        }
    }
}
