use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use std::collections::HashSet;
use syn::{FnArg, Ident, ItemFn, PatType, Type, Visibility};

use super::{Altering, Inscriptions, option_iter_quote, option_quote};
use crate::{Mining, ServicePlumber};
use pheasant_core::{Cors, Method, Mime};
use pheasant_uri::Route;

#[derive(Debug)]
pub struct ServicePoet {
    fun: ItemFn,
    method: Method,
    route: Route,
    mime: Option<Mime>,
    // if the user service returns a Response then true
    // els if it returns Vec<u8> then false
    decorated: bool,
    // if Some then an Options service corresponding to the user service has to be generated
    // with the passed cors headers, service route and client request origin
    cors: Option<Cors>,
    // requesting any of the Routes in redirections triggers a redirection response towards
    // this service
    re: Option<HashSet<Route>>,
}

impl ServicePoet {
    pub fn new(mut plumber: ServicePlumber) -> Self {
        let method = plumber.method();
        let route = plumber.take_route();
        let mime = plumber.take_mime();
        let cors = plumber.take_cors();
        let re = plumber.take_re();

        let fun = plumber.into_fun();
        let decorated = fun.is_decorated();

        Self {
            fun,
            decorated,
            method,
            route,
            mime,
            cors,
            re,
        }
    }
}

impl Inscriptions for ServicePoet {
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

    fn route(&self) -> TS2 {
        let s = self.route.as_str();
        quote! { pheasant::Route::macro_checked(#s) }
    }

    fn origin_set(&self) -> TS2 {
        let Some(cors) = &self.cors else {
            panic!(
                "this function should only be called from inside the is_some block of impl Inscriptions::cors for ServicePoet"
            );
        };

        let origins = cors.cors_origins();
        if origins.is_any_origin() {
            quote! { pheasant::OriginSet::AnyOrigin }
        } else {
            let ori = origins.origins_ref().unwrap().into_iter().map(|ori| {
                let s = ori.sequence();

                quote! { #s.parse::<pheasant::Origin>().unwrap() }
            });

            quote! { pheasant::OriginSet::macro_checked(std::collections::HashSet::from([ #(#ori,)* ])) }
        }
    }

    fn cors(&self) -> TS2 {
        if let Some(ref cors) = self.cors {
            let methods = cors.cors_methods().into_iter();
            let headers = cors
                .cors_headers()
                .into_iter()
                .map(|h| quote! { std::string::String::from(#h) });
            let expose = cors.cors_expose().map(|exp| {
                exp.into_iter()
                    .map(|e| quote! { std::string::String::from(#e) })
            });
            let expose = option_iter_quote(expose);
            let origins = self.origin_set();
            let max_age = cors.cors_max_age();
            let max_age = option_quote(max_age);

            quote! {
                Some(pheasant::Cors::macro_checked( std::collections::HashSet::from([ #(pheasant:: #methods,)* ]),  std::collections::HashSet::from([ #(#headers,)* ]), #expose,  #origins, #max_age))
            }
        } else {
            quote! { None }
        }
    }

    fn re(&self) -> TS2 {
        // WARN this ruins the whole point of
        // generating the ready types inside the macro
        if let Some(ref re) = self.re {
            let re = re
                .into_iter()
                .map(|re| re.as_str())
                .map(|re| quote! { pheasant::Route::macro_checked(#re)  });

            quote! {
                Some(std::collections::HashSet::from([ #(#re,)* ]))
            }
        } else {
            quote! { None }
        }
    }
}

// the user function -> fun_service -> Vec<u8> / -> Response
//
// the user fun wrapper -> fun_respond
// <- wraps Fn(From<Req>) -> Vec<u8> into Fn(From<Req>) -> Response
//
// the service function -> fun
// <- the wrapper function is what the user passes to server.service()
pub trait ServiceInscriptions {
    // this is only called if self.decorated is false
    // wraps the user fn in a Fn(UserInputType) -> Response
    fn assemble_decorator_fun(&self) -> TS2;

    // makes the fun that returns a Service bundle
    fn assemble_bundler_fun(&self) -> TS2;

    // returns the proc_macro2::TokenStream value of this ServicePoet's preflight fun
    // i.e., this implements an Options service for this ServicePoet's Route
    // using the Cors at hand
    fn assemble_preflight_fun(&self) -> TS2;

    fn assemble(&mut self) -> TS2;
}

fn service(method: Method, route: &TS2, re: &TS2, mime: &TS2, cors: &TS2, fun: &Ident) -> TS2 {
    quote! {pheasant::Service::new(pheasant::#method, #route, #re, #mime, #cors, #fun) }
}

impl ServiceInscriptions for ServicePoet {
    fn assemble_decorator_fun(&self) -> TS2 {
        let fun = &self.fun;
        let vis = fun.vis();
        let ident = fun.decorate_ident("_decorator");
        let service = fun.decorate_ident("_service");
        let arg = fun.user_argtype();

        if self.decorated {
            quote! {
                #vis async fn #ident(i: #arg, p: pheasant::Protocol) -> pheasant::Response {
                    let mut resp = #service(i).await;
                    resp.update_proto(p);

                    resp
                }
            }
        } else {
            quote! {
                #vis async fn #ident(i: #arg, proto: pheasant::Protocol) -> pheasant::Response {
                    let mut resp = pheasant::Response::with_proto(proto);
                    let data = #service(i).await;
                    resp.update_body(data);

                    resp
                }
            }
        }
    }

    fn assemble_preflight_fun(&self) -> TS2 {
        let method = Method::Options;
        let fun = &self.fun;
        let vis = fun.vis();
        let preflight = fun.decorate_ident("_preflight");
        let service = fun.decorate_ident("_preflight_service");
        let route = self.route();
        let cors = self.cors();

        quote! {
            #vis async fn #preflight(origin: pheasant::RequestOrigin, proto: pheasant::Protocol) -> pheasant::Response {
                let mut resp = pheasant::Response::preflight(& #cors .unwrap(), origin.origin());
                resp.update_status(pheasant::Status::Successful(pheasant::Successful::NoContent), None, "");
                resp.update_proto(proto);

                resp
            }
        }
    }

    fn assemble_bundler_fun(&self) -> TS2 {
        let fun = &self.fun;
        let vis = fun.vis();
        let bundler = &fun.sig.ident;
        let route = self.route();
        let mime = self.mime();
        let re = self.re();
        let cors = self.cors();
        let method = self.method;

        let preflight = fun.decorate_ident("_preflight");
        let preflight = self.cors.is_some().then(|| {
            service(
                Method::Options,
                &route,
                &quote! { None },
                &quote! {None},
                &quote! { None },
                &preflight,
            )
        });
        let decorated = fun.decorate_ident("_decorator");
        let decorated = service(method, &route, &re, &mime, &cors, &decorated);
        let (return_type, service_bundle) = if self.cors.is_some() {
            (
                Type::Verbatim("[pheasant::Service; 2]".parse().unwrap()),
                quote! {[
                    #preflight,
                    #decorated
                ]},
            )
        } else {
            (
                Type::Verbatim("pheasant::Service".parse().unwrap()),
                quote! {#decorated},
            )
        };

        quote! {
            #vis fn #bundler() -> #return_type {
                #service_bundle
            }
        }
    }

    fn assemble(&mut self) -> TS2 {
        let fun = &self.fun;
        let vis = fun.vis();

        let decorator_fun = self.assemble_decorator_fun();
        let preflight_funs = self.cors.is_some().then(|| self.assemble_preflight_fun());
        // TODO still doesnt bundle preflight service in
        // TODO too many redundant assignments in the functions assemblers
        // just assign everythin here and pass them by ref
        let bundler_fun = self.assemble_bundler_fun();

        self.fun.sig.ident = fun.decorate_ident("_service");
        let fun = &self.fun;

        quote! {
            #fun

            #decorator_fun

            #preflight_funs

            #bundler_fun
        }
    }
}
