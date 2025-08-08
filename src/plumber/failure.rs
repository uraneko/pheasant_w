use std::collections::HashSet;
extern crate proc_macro;
use proc_macro::TokenStream;
use syn::ItemFn;
use syn::parse;
use syn::parse::Result as PRes;

use crate::{IntAttr, Mining};
use pheasant_core::Mime;

#[derive(Debug)]
pub struct FailurePlumber {
    fun: ItemFn,
    mime: Option<Mime>,
    status: u16,
}

impl FailurePlumber {
    pub fn new(attr: TokenStream, fun: TokenStream) -> PRes<Self> {
        let mut fun: ItemFn = parse(fun)?;
        let status: IntAttr = parse(attr)?;
        let status = status.to_u16();
        let mime = fun.mime();

        Ok(Self { status, mime, fun })
    }

    pub fn into_fun(self) -> ItemFn {
        self.fun
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn take_mime(&mut self) -> Option<Mime> {
        std::mem::take(&mut self.mime)
    }
}
