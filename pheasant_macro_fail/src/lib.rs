use pheasant_macro_utils::{FailureInscriptions, FailurePlumber, FailurePoet, Plumber, Poet};
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn fail(attr: TokenStream, fun: TokenStream) -> TokenStream {
    let plumber = Plumber::<FailurePlumber>::new(attr, fun).unwrap();
    let mut poet = Poet::<FailurePoet>::new(plumber);

    poet.assemble().into()
}
