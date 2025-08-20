use std::collections::HashSet;
extern crate proc_macro;
use proc_macro::TokenStream;
use syn::ItemFn;
use syn::parse;
use syn::parse::Result as PRes;

use crate::{CorsAttr, Mining, StrAttr};
use pheasant_core::{Cors, Method, Mime};
use pheasant_uri::Route;

#[derive(Debug)]
pub struct ServicePlumber {
    fun: ItemFn,
    method: Method,
    route: Route,
    mime: Option<Mime>,
    cors: Option<CorsAttr>,
    re: Option<HashSet<Route>>,
}

impl ServicePlumber {
    /// consumes the macro inputs into a new Plumber
    pub fn new(method: Method, attr: TokenStream, fun: TokenStream) -> PRes<Self> {
        let mut fun: ItemFn = parse(fun)?;
        let route: StrAttr = parse(attr)?;
        let route = route.as_str().parse::<Route>().unwrap();

        let mime = fun.mime();
        let re = fun.re();
        let cors = fun.cors();

        Ok(Self {
            method,
            route,
            fun,
            mime,
            re,
            cors,
        })
    }

    /// consumes self and returns the ItemFn
    pub fn into_fun(self) -> ItemFn {
        self.fun
    }

    /// copies self.method
    pub fn method(&self) -> Method {
        self.method
    }

    /// takes self.route
    pub fn take_route(&mut self) -> Route {
        std::mem::take(&mut self.route)
    }

    /// takes self.mime
    pub fn take_mime(&mut self) -> Option<Mime> {
        std::mem::take(&mut self.mime)
    }

    /// takes self.cors
    pub fn take_cors(&mut self) -> Option<Cors> {
        std::mem::take(&mut self.cors).map(|ca| ca.cors())
    }

    /// takes self.re
    pub fn take_re(&mut self) -> Option<HashSet<Route>> {
        std::mem::take(&mut self.re)
    }
}
