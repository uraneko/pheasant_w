use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use std::collections::HashSet;
use syn::{FnArg, Ident, ItemFn, PatType, Type, Visibility};

use crate::{FailurePlumber, Mining, Plumber, ServicePlumber};
use pheasant_core::{Cors, Method, Mime};
use pheasant_uri::Route;

pub mod failure;
pub mod service;

pub use failure::{FailureInscriptions, FailurePoet};
pub use service::{ServiceInscriptions, ServicePoet};

pub struct Poet<K>(K);

impl<K> std::ops::Deref for Poet<K> {
    type Target = K;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K> std::ops::DerefMut for Poet<K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Poet<ServicePoet> {
    pub fn new(plumber: Plumber<ServicePlumber>) -> Self {
        Self(ServicePoet::new(plumber.plumber()))
    }
}

impl Poet<FailurePoet> {
    pub fn new(plumber: Plumber<FailurePlumber>) -> Self {
        Self(FailurePoet::new(plumber.plumber()))
    }
}

pub trait Altering {
    /// makes a new Ident from the String value
    /// of the passed &Ident suffixed by the passed &str
    fn decorate_ident(&self, suffix: &str) -> Ident;

    /// returns user fun visibility
    fn vis(&self) -> &Visibility;

    /// returns the type name of the user fun argument type
    ///
    /// the service fun must only have 1 arg
    fn user_argtype(&self) -> &Type;
}

impl Altering for ItemFn {
    fn decorate_ident(&self, suffix: &str) -> Ident {
        let i = self.sig.ident.to_string() + suffix;

        Ident::new(&i, Span::call_site())
    }

    fn vis(&self) -> &Visibility {
        &self.vis
    }

    fn user_argtype(&self) -> &Type {
        let FnArg::Typed(PatType { ty, .. }) = &self.sig.inputs.first().unwrap() else {
            panic!(
                "bad sigature, this pub fn can't take self, provide a type T that satistifies: From<Request> instead"
            );
        };

        ty
    }
}

trait Inscriptions {
    /// returns the proc_macro2::TokenStream value of this Poet's Mime
    fn mime(&self) -> TS2;

    /// returns the proc_macro2::TokenStream value of this Poet's Route
    fn route(&self) -> TS2;

    /// returns the proc_macro2::TokenStream value of this Poet's redirection hashSet<Route>
    fn re(&self) -> TS2;

    /// returns the proc_macro2::TokenStream value of this Poet's Cors
    fn cors(&self) -> TS2;

    // return proc_macro2::TokenStream repr of the cors.origins field
    fn origin_set(&self) -> TS2;
}

// quotes an option of T: ToTokens as an option instead of unwrapping
fn option_quote<T>(opt: Option<T>) -> TS2
where
    T: quote::ToTokens,
{
    match opt {
        Some(t) => quote! { #t },
        None => quote! { None },
    }
}

// quotes an option of I: IntoIterator<Item = T>, T: ToTokens
// as an option instead of unwrapping
fn option_iter_quote<I, T>(opt: Option<I>) -> TS2
where
    I: IntoIterator<Item = T>,
    T: quote::ToTokens,
{
    match opt {
        Some(i) => {
            let i = i.into_iter();
            quote! { std::collections::HashSet::from([ #(#i,)* ]) }
        }
        None => quote! { None },
    }
}
