use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use syn::{Ident, ItemFn};

fn fn_ident(fun: &ItemFn) -> Ident {
    fun.sig.ident.clone()
}

fn suffix_ident(fun: &mut ItemFn) {
    let i = fun.sig.ident.to_string();
    fun.sig.ident = Ident::new(&(i + "_service"), Span::call_site());
}

pub fn wrapper_fn(
    route: String,
    re: Option<Vec<String>>,
    mime: Option<String>,
    mut fun: ItemFn,
) -> TS2 {
    let ident = fn_ident(&fun);
    suffix_ident(&mut fun);
    let suffixed = fn_ident(&fun);

    let vis = &fun.vis;

    let mime = if let Some(mime) = mime {
        quote! { #mime }
    } else {
        quote! { "" }
    };

    let re = if let Some(re) = re {
        quote! { [#(#re,)*] }
    } else {
        quote! { [] }
    };

    quote! {
        #fun

        #vis fn #ident() -> pheasant_core::Service {
            pheasant_core::Service::new(pheasant_core::Method::Get, #route, #re, #mime, #suffixed)
        }
    }
}
