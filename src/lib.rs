use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;

use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::parse_macro_input;
use syn::{Attribute, Expr, Ident, ItemFn, Lit, LitStr, Token};

use quote::quote;

use pheasant_macro_utils::generate_service;
use pheasant_macro_utils::{StrAttr, cors, mime, re};

// TODO add a new attribute; CORS, e.g.,
// #[CORS(methods = [get, post, options], origin = "*", credentials = false, headers = ["X-PingOther", "Content-Type"])]

#[proc_macro_attribute]
pub fn get(attr: TokenStream, fun: TokenStream) -> TokenStream {
    // let [attr, fun]: [TS2; 2] = [attr.into(), fun.into()];
    // println!("{}\n{}", quote! {#attr}, quote! { #fun  });

    let resou = parse_macro_input!(attr as StrAttr);
    let mut fun = parse_macro_input!(fun as ItemFn);
    let mime = mime(&mut fun);
    let re = re(&mut fun);
    let cors = cors(&mut fun);

    let funs = generate_service(resou.into_string(), mime, re, cors, fun);

    quote! {
        #funs
    }
    .into()
}
