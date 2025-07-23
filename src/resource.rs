use proc_macro2::Span;
use syn::Lit;
use syn::parse::{Parse, ParseStream, Result as PRes};

#[derive(Debug)]
pub struct Resource {
    route: String,
}

impl Resource {
    pub fn route(self) -> String {
        self.route
    }
}

impl Parse for Resource {
    fn parse(s: ParseStream) -> PRes<Self> {
        Ok(Self {
            route: {
                let Ok(Lit::Str(sl)) = Lit::parse(s) else {
                    return Err(syn::parse::Error::new(
                        Span::call_site(),
                        "wrong lit variant, expected str",
                    ));
                };

                sl.value()
            },
        })
    }
}
