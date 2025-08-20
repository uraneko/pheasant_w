use proc_macro2::Span;
use std::collections::HashSet;
use syn::{Ident, ItemFn, Lit, Meta, ReturnType, Type};

use crate::{CorsAttr, StrVec};
use pheasant_core::Mime;
use pheasant_uri::Route;

pub trait Mining {
    fn is_decorated(&self) -> bool;

    fn mime(&mut self) -> Option<Mime>;

    fn re(&mut self) -> Option<HashSet<Route>>;

    fn cors(&mut self) -> Option<CorsAttr>;
}

impl Mining for ItemFn {
    /// checks wether the service ItemFn returns a Response or not
    fn is_decorated(&self) -> bool {
        let ReturnType::Type(_, ty) = &self.sig.output else {
            panic!("return type variant was default instead of type");
        };

        let Type::Path(tp) = &**ty else {
            panic!("service return type has to be a PathType");
        };

        let Some(ps) = tp.path.segments.last() else {
            panic!("service return type (path type) can't have no segments");
        };

        let i = &ps.ident;

        i == &Ident::new("Response", Span::call_site())
    }

    fn mime(&mut self) -> Option<Mime> {
        let Some(idx) = self
            .attrs
            .iter()
            .map(|a| a.path())
            .position(|p| p.get_ident() == Some(&Ident::new("mime", Span::call_site())))
        else {
            return None;
        };

        if let Lit::Str(sl) = self.attrs.remove(idx).parse_args::<Lit>().unwrap() {
            Some(sl.value().parse::<Mime>().unwrap())
        } else {
            None
        }
    }

    fn re(&mut self) -> Option<HashSet<Route>> {
        let Some(idx) = self
            .attrs
            .iter()
            .map(|a| a.path())
            .position(|p| p.get_ident() == Some(&Ident::new("re", Span::call_site())))
        else {
            return None;
        };

        let attr = self.attrs.remove(idx);
        let Meta::List(ml) = attr.meta else {
            return None;
        };

        ml.parse_args::<StrVec>()
            .map(|r| {
                r.into_iter()
                    .map(|r| r.parse::<Route>().unwrap())
                    .collect::<HashSet<Route>>()
            })
            .ok()
    }

    fn cors(&mut self) -> Option<CorsAttr> {
        let Some(idx) = self
            .attrs
            .iter()
            .map(|a| a.path())
            .position(|p| p.get_ident() == Some(&Ident::new("cors", Span::call_site())))
        else {
            return None;
        };
        let attr = self.attrs.remove(idx);
        let Meta::List(ml) = attr.meta else {
            return None;
        };

        ml.parse_args::<CorsAttr>().ok()
    }
}
