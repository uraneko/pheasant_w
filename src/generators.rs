use crate::CorsAttr;
use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use syn::{FnArg, Ident, ItemFn, PatType, ReturnType, Type};

// suffixes an ident with the passed &str value
pub fn suffix_fn_ident(fun: &mut ItemFn, suffix: &str) {
    let i = fun.sig.ident.to_string();
    fun.sig.ident = Ident::new(&(i + suffix), Span::call_site());
}

pub fn suffix_ident(i: &Ident, suffix: &str) -> Ident {
    Ident::new(&(i.to_string() + suffix), Span::call_site())
}

pub fn clone_ident(fun: &ItemFn) -> Ident {
    fun.sig.ident.clone()
}

pub fn ref_ident(fun: &ItemFn) -> &Ident {
    &fun.sig.ident
}

pub fn gen_mime(mime: Option<String>) -> TS2 {
    if let Some(mime) = mime {
        quote! { #mime }
    } else {
        quote! { "" }
    }
}

pub fn gen_re(re: Option<Vec<String>>) -> TS2 {
    if let Some(re) = re {
        quote! { [#(#re,)*] }
    } else {
        quote! { [] }
    }
}

pub fn gen_arg_ty(fun: &ItemFn) -> &Type {
    let inputs = &fun.sig.inputs;

    let FnArg::Typed(PatType { ty, .. }) = inputs.first().unwrap() else {
        panic!(
            "bad sigature, this pub fn can't take self, provide a type T: From<Request> instead"
        );
    };

    ty
}

pub fn match_ret_ty(fun: &ItemFn) -> bool {
    let ReturnType::Type(tk, ty) = &fun.sig.output else {
        panic!("return type variant was default instead of type");
    };

    quote! { #ty }.to_string().ends_with("Response")
}

pub fn generate_service(
    route: String,
    mime: Option<String>,
    re: Option<Vec<String>>,
    cors: Option<CorsAttr>,
    fun: ItemFn,
) -> TS2 {
    // let vis = &fun.vis;
    // let mime = gen_mime(mime);
    // let re = gen_re(re);
    let is_responder = match_ret_ty(&fun);

    if is_responder {
        generate_service_from_responder(route, re, cors, fun)
    } else {
        generate_service_from_data_func(route, mime, re, cors, fun)
    }
}

pub fn generate_service_from_data_func(
    route: String,
    mime: Option<String>,
    re: Option<Vec<String>>,
    cors: Option<CorsAttr>,
    mut fun: ItemFn,
) -> TS2 {
    // the func that returns Service
    let service_fun = clone_ident(&fun);
    // the func that wraps the user defined service fn (-> Vec<u8>) into a fn (-> Response)
    let respond_fun = suffix_ident(&service_fun, "_service");
    // suffix user defined fn
    suffix_fn_ident(&mut fun, "_respond");
    let user_fun = ref_ident(&fun);

    let vis = &fun.vis;
    let mime = gen_mime(mime);
    let re = gen_re(re);
    let arg = gen_arg_ty(&fun);

    // this block of code is only called when
    // user defined service returns Vec<u8>
    // need to add a wrapper fn that returns response
    quote! {
        #fun

        // NOTE this here is the new suffixed
        // i.e., the original service function itself should be modified
        // to return a Response template
        #vis async fn #respond_fun(
            i: #arg,
            p: pheasant_core::Protocol
        ) -> pheasant_core::Response {
            let mut resp = pheasant_core::Response::template(p);
            let data = #user_fun(i).await;
            resp.update_body(data);

            resp
        }

        #vis fn #service_fun() -> pheasant_core::Service {
            pheasant_core::Service::new(
                pheasant_core::Method::Get, #route, #re, #mime, #respond_fun
            )
        }
    }
}

pub fn generate_service_from_responder(
    route: String,
    re: Option<Vec<String>>,
    cors: Option<CorsAttr>,
    mut fun: ItemFn,
) -> TS2 {
    // the func that returns Service
    let service_fun = clone_ident(&fun);
    // the func that wraps the user defined service fn (-> Vec<u8>) into a fn (-> Response)
    let respond_fun = suffix_ident(&service_fun, "_service");
    // suffix user defined fn
    suffix_fn_ident(&mut fun, "_respond");
    let user_fun = ref_ident(&fun);

    let vis = &fun.vis;
    let re = gen_re(re);
    let arg = gen_arg_ty(&fun);

    // user defined service returns Response already
    // this always has to be returned
    quote! {
        #fun

        #vis async fn #respond_fun(i: #arg, p: pheasant_core::Protocol) -> pheasant_core::Response {
            let mut resp = #user_fun(i).await;
            resp.update_proto(p);

            resp
        }

        #vis fn #service_fun() -> pheasant_core::Service {
            pheasant_core::Service::new(pheasant_core::Method::Get, #route, #re, "", #respond_fun)
        }
    }
}
