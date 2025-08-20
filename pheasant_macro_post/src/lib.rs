use proc_macro::TokenStream;

use pheasant_macro_utils::{
    Method, Plumber, Poet, ServiceInscriptions, ServicePlumber, ServicePoet,
};

#[proc_macro_attribute]
pub fn post(attr: TokenStream, fun: TokenStream) -> TokenStream {
    let plumber = Plumber::<ServicePlumber>::new(Method::Post, attr, fun).unwrap();
    let mut poet = Poet::<ServicePoet>::new(plumber);

    poet.assemble().into()
}
