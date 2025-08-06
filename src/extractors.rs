use chrono::TimeDelta;
use pheasant_core::{Cors, Method};
use pheasant_uri::{Origin, OriginSet};
use proc_macro2::Span;
use std::collections::HashSet;
use syn::parse::{Error as ParseError, Parse, ParseBuffer, ParseStream, Result as PRes};
use syn::parse_macro_input;
use syn::{Ident, ItemFn, Lit, Meta, MetaNameValue, Token, bracketed, token::Bracket};

use super::parsers::{CorsAttr, StrAttr, StrVec};

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

    ml.parse_args::<StrVec>().map(|r| r.into_vec()).ok()
}

pub fn cors(fun: &mut ItemFn) -> Option<CorsAttr> {
    let Some(idx) = fun
        .attrs
        .iter()
        .map(|a| a.path())
        .position(|p| p.get_ident() == Some(&Ident::new("cors", Span::call_site())))
    else {
        return None;
    };
    let attr = fun.attrs.remove(idx);
    let Meta::List(ml) = attr.meta else {
        return None;
    };

    ml.parse_args::<CorsAttr>().ok()
}
