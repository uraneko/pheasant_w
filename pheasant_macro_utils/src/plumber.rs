use std::collections::HashSet;
extern crate proc_macro;
use proc_macro::TokenStream;
use syn::ItemFn;
use syn::parse;
use syn::parse::Result as PRes;

use crate::{CorsAttr, Mining, StrAttr};
use pheasant_core::{Cors, Method, Mime};
use pheasant_uri::Route;

pub mod failure;
pub mod service;

pub use failure::FailurePlumber;
pub use service::ServicePlumber;

pub struct Plumber<K>(K);

impl<K> std::ops::Deref for Plumber<K> {
    type Target = K;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K> std::ops::DerefMut for Plumber<K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K> Plumber<K> {
    pub fn plumber(self) -> K {
        self.0
    }
}

impl Plumber<ServicePlumber> {
    pub fn new(method: Method, attr: TokenStream, fun: TokenStream) -> PRes<Self> {
        Ok(Self(ServicePlumber::new(method, attr, fun)?))
    }
}

impl Plumber<FailurePlumber> {
    pub fn new(attr: TokenStream, fun: TokenStream) -> PRes<Self> {
        Ok(Self(FailurePlumber::new(attr, fun)?))
    }
}
