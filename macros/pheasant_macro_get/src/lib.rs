use pheasant_macro_utils::{
    Method, Plumber, Poet, ServiceInscriptions, ServicePlumber, ServicePoet,
};
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn get(attr: TokenStream, fun: TokenStream) -> TokenStream {
    let plumber = Plumber::<ServicePlumber>::new(Method::Get, attr, fun).unwrap();
    let mut poet = Poet::<ServicePoet>::new(plumber);

    poet.assemble().into()
}
