use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::parse_macro_input;
use syn::{Attribute, Expr, Ident, ItemFn, Lit, LitStr, Token};

use pheasant_macro_utils::Method;
use pheasant_macro_utils::{Plumber, Poet, ServiceInscriptions};

// TODO add a new attribute; CORS, e.g.,
// #[CORS(methods = [get, post, options], origin = "*", credentials = false, headers = ["X-PingOther", "Content-Type"])]

#[proc_macro_attribute]
pub fn get(attr: TokenStream, fun: TokenStream) -> TokenStream {
    // let [attr, fun]: [TS2; 2] = [attr.into(), fun.into()];
    // println!("{}\n{}", quote! {#attr}, quote! { #fun  });

    let plumber = Plumber::new(Method::Get, attr, fun).unwrap();
    println!("{:?}", plumber);

    let mut poet = Poet::new(plumber);

    poet.assemble().into()
}
