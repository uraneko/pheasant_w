use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;

use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::parse_macro_input;
use syn::{Attribute, Expr, Ident, ItemFn, Lit, LitStr, Token};

use quote::quote;

mod callback;
mod resource;

use callback::{mime, wrapper_fn};
use resource::Resource;

#[proc_macro_attribute]
pub fn get(attr: TokenStream, func: TokenStream) -> TokenStream {
    // let [attr, func]: [TS2; 2] = [attr.into(), func.into()];
    // println!("{}\n{}", quote! {#attr}, quote! { #func  });

    let resou = parse_macro_input!(attr as Resource);
    let mut func = parse_macro_input!(func as ItemFn);
    let mime = mime(&mut func);

    let wra_fun = wrapper_fn(resou.route(), mime, func);
    println!("{}", quote! { #wra_fun });

    quote! {
        #wra_fun
    }
    .into()
}
