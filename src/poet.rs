use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use std::collections::HashSet;
use syn::{FnArg, Ident, ItemFn, PatType, Type, Visibility};

use crate::{Mining, Plumber};
use pheasant_core::{Cors, Method, Mime, Request};
use pheasant_uri::{Origin, Route};

#[derive(Debug)]
pub struct Poet {
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

impl Poet {
    fn new(mut plumber: Plumber) -> Self {
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

fn option_quote<T>(opt: Option<T>) -> TS2
where
    T: quote::ToTokens,
{
    match opt {
        Some(t) => quote! { #t },
        None => quote! { None },
    }
}

fn option_iter_quote<I, T>(opt: Option<I>) -> TS2
where
    I: IntoIterator<Item = T>,
    T: quote::ToTokens,
{
    match opt {
        Some(i) => {
            let i = i.into_iter();
            quote! { HashSet::from([ #(#i,)* ]) }
        }
        None => quote! { None },
    }
}

impl Inscriptions for Poet {
    fn mime(&self) -> TS2 {
        if let Some(ref mime) = self.mime {
            let mime = mime.essence_str();
            quote! {
                Some(Mime::macro_checked(#mime))
            }
        } else {
            quote! { None }
        }
    }

    fn route(&self) -> TS2 {
        let s = self.route.as_str();
        quote! { Route::macro_checked(#s) }
    }

    fn origin_set(&self) -> TS2 {
        let Some(cors) = &self.cors else {
            panic!(
                "this function should only be called from inside the is_some block of impl Inscriptions::cors for Poet"
            );
        };

        let origins = cors.cors_origins();
        if origins.is_any_origin() {
            quote! { OriginSet::AnyOrigin }
        } else {
            let ori = origins.origins_ref().unwrap().into_iter().map(|ori| {
                let s = ori.sequence();

                quote! { #s.parse::<Origin>().unwrap() }
            });

            quote! { OriginSet::macro_checked(HashSet::from([ #(#ori,)* ])) }
        }
    }

    fn cors(&self) -> TS2 {
        if let Some(ref cors) = self.cors {
            let methods = cors.cors_methods().into_iter().map(|m| m.as_str());
            let headers = cors
                .cors_headers()
                .into_iter()
                .map(|h| quote! { String::from(#h) });
            let expose = cors
                .cors_expose()
                .map(|exp| exp.into_iter().map(|e| quote! { String::from(#e) }));
            let expose = option_iter_quote(expose);
            let origins = self.origin_set();
            let max_age = cors.cors_max_age();
            let max_age = option_quote(max_age);

            quote! {
                Some(Cors::macro_checked( HashSet::from([ #(#methods,)* ]),  HashSet::from([ #(#headers,)* ]), #expose,  #origins, #max_age))
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
                .map(|re| quote! { Route::macro_checked(#re)  });

            quote! {
                Some(HashSet::from([ #(#re,)* ]))
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

    // returns the proc_macro2::TokenStream value of this Poet's preflight fun
    // i.e., this implements an Options service for this Poet's Route
    // using the Cors at hand
    fn assemble_preflight_fun(&self) -> TS2;

    fn assemble(&mut self) -> TS2;
}

fn service(method: Method, route: &TS2, re: &TS2, mime: &TS2, cors: &TS2, fun: &Ident) -> TS2 {
    quote! {Service::new(#method, #route, #re, #mime, #cors, #fun) }
}

impl ServiceInscriptions for Poet {
    fn assemble_decorator_fun(&self) -> TS2 {
        let fun = &self.fun;
        let vis = fun.vis();
        let ident = fun.decorate_ident("_decorator");
        let service = fun.decorate_ident("_service");
        let arg = fun.user_argtype();

        quote! {
            #vis async fn #ident(i: #arg, p: Protocol) -> Response {
                let mut resp = #service(i).await;
                resp.update_proto(p);

                resp
            }
        }
    }

    fn assemble_bundler_fun(&self) -> TS2 {
        let fun = &self.fun;
        let vis = fun.vis();
        let ident = fun.decorate_ident("_bundler");
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
        let maybe_decorated = if self.decorated {
            fun.decorate_ident("_decorator")
        } else {
            fun.decorate_ident("_service")
        };
        let maybe_decorated = service(method, &route, &re, &mime, &cors, &maybe_decorated);
        let service_bundle = if self.cors.is_some() {
            quote! {[
                #preflight,
            #maybe_decorated
            ]}
        } else {
            quote! {#maybe_decorated}
        };

        quote! {
            #vis fn #ident<B>() -> B
            where B: ServiceBundle,
            {
                #service_bundle
            }
        }
    }

    fn assemble_preflight_fun(&self) -> TS2 {
        let method = Method::Options;
        let fun = &self.fun;
        let vis = fun.vis();
        let preflight = fun.decorate_ident("_preflight");
        let service = fun.decorate_ident("_preflight_service");
        let route = self.route.as_str();
        let cors = self.cors();

        quote! {
            #vis async fn #preflight(origin: RequestOrigin) -> Response {
                let mut resp = Response::preflight(& #cors, origin.origin());
                res.update_status(Status::Successful(Successful::NoContent), None, "");

                resp
            }

            #vis fn #service() -> Service {
                Service::new(#method, #route, None, None, #cors, #preflight)
            }
        }
    }

    fn assemble(&mut self) -> TS2 {
        let fun = &self.fun;
        let vis = fun.vis();

        let decorator_fun = self.decorated.then(|| self.assemble_decorator_fun());
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

pub struct RequestOrigin(Option<Origin>);

impl From<&Request> for RequestOrigin {
    fn from(req: &Request) -> Self {
        let Some(ori) = req.param("Origin") else {
            return RequestOrigin(None);
        };

        let ori = ori.parse::<Origin>().unwrap();

        Self(Some(ori))
    }
}

impl RequestOrigin {
    pub fn origin(&self) -> Option<&Origin> {
        self.0.as_ref()
    }
}
