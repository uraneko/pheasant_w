use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;

use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::parse_macro_input;
use syn::{Attribute, Lit, LitStr, Token};

use quote::quote;

#[proc_macro_attribute]
pub fn get(attr: TokenStream, func: TokenStream) -> TokenStream {
    // let [attr, func]: [TS2; 2] = [attr.into(), func.into()];
    // println!("{}\n{}", quote! {#attr}, quote! { #func  });

    let attr = parse_macro_input!(attr as HttpMeta);

    println!("{:#?}", attr);

    quote! {}.into()
}

#[derive(Debug)]
struct HttpMeta {
    uri: String,
    mime: Option<String>,
}

impl Parse for HttpMeta {
    fn parse(s: ParseStream) -> PRes<Self> {
        Ok(Self {
            uri: {
                let Ok(Lit::Str(sl)) = Lit::parse(s) else {
                    return Err(syn::parse::Error::new(
                        Span::call_site(),
                        "wrong lit variant, expected str",
                    ));
                };

                sl.value()
            },
            mime: if s.is_empty() {
                None
            } else {
                _ = <Token![,]>::parse(s)?;
                let Ok(Lit::Str(sl)) = Lit::parse(s) else {
                    return Err(syn::parse::Error::new(
                        Span::call_site(),
                        "wrong lit variant, expected str",
                    ));
                };

                Some(sl.value())
            },
        })
    }
}
