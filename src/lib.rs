use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;

use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::parse_macro_input;
use syn::{Attribute, Expr, Ident, ItemFn, Lit, LitStr, Token};

use quote::quote;

mod callback;
mod resource;

use callback::wrapper_fn;
use resource::{Resource, mime, re};

#[proc_macro_attribute]
pub fn post(attr: TokenStream, fun: TokenStream) -> TokenStream {
    // let [attr, fun]: [TS2; 2] = [attr.into(), fun.into()];
    // println!("{}\n{}", quote! {#attr}, quote! { #fun  });

    let resou = parse_macro_input!(attr as Resource);
    let mut fun = parse_macro_input!(fun as ItemFn);
    let mime = mime(&mut fun);
    let re = re(&mut fun);

    let wra_fun = wrapper_fn(resou.route(), re, mime, fun);

    quote! {
        #wra_fun
    }
    .into()
}
