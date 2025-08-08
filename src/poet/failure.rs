use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use std::collections::HashSet;
use syn::{FnArg, Ident, ItemFn, PatType, Type, Visibility};

use super::Altering;
use crate::{FailurePlumber, Mining};
use pheasant_core::{Cors, Method, Mime};
use pheasant_uri::Route;

#[derive(Debug)]
pub struct FailurePoet {
    fun: ItemFn,
    status: u16,
    mime: Option<Mime>,
}

impl FailurePoet {
    pub fn new(mut plumber: FailurePlumber) -> Self {
        FailurePoet {
            status: plumber.status(),
            mime: plumber.take_mime(),
            fun: plumber.into_fun(),
        }
    }
}

pub trait FailureInscriptions {
    fn mime(&self) -> TS2;

    fn status(&self) -> TS2;

    fn assemble_failure_fun(&self) -> TS2;

    fn assemble(&self) -> TS2;
}

impl FailureInscriptions for FailurePoet {
    fn mime(&self) -> TS2 {
        if let Some(ref mime) = self.mime {
            let mime = mime.essence_str();
            quote! {
                Some(pheasant::Mime::macro_checked(#mime))
            }
        } else {
            quote! { None }
        }
    }

    fn status(&self) -> TS2 {
        let status = pheasant_core::ErrorStatus::try_from(self.status).unwrap();

        quote! { pheasant:: #status }
    }

    fn assemble_failure_fun(&self) -> TS2 {
        let fun = &self.fun;
        let mime = self.mime();
        let status = self.status();

        let decorated = &fun.sig.ident;
        let failure = fun.decorate_ident("_failure");

        quote! {
            fn #decorated() -> pheasant::Failure {

                pheasant::Failure::new(#status, #mime, #failure)
            }

        }
    }

    fn assemble(&self) -> TS2 {
        let decorated_fun = self.assemble_failure_fun();
        let fun = &self.fun;

        quote! {
            #fun

            #decorated_fun
        }
    }
}
