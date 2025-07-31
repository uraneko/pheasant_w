use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;

use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::parse_macro_input;
use syn::{Attribute, Expr, Ident, ItemFn, Lit, LitStr, Token};

use quote::quote;

mod callback;
mod resource;

use callback::generate_service;
use resource::{Resource, mime, re};

#[proc_macro_attribute]
pub fn get(attr: TokenStream, fun: TokenStream) -> TokenStream {
    // let [attr, fun]: [TS2; 2] = [attr.into(), fun.into()];
    // println!("{}\n{}", quote! {#attr}, quote! { #fun  });

    let resou = parse_macro_input!(attr as Resource);
    let mut fun = parse_macro_input!(fun as ItemFn);
    let mime = mime(&mut fun);
    let re = re(&mut fun);

    let funs = generate_service(resou.route(), re, mime, fun);

    quote! {
        #funs
    }
    .into()
}
