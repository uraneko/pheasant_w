#![no_main]
use pheasant_macro_get::get;

struct Params;

#[get("/")]
#[re("/index.html")] // <- redirection 303 see other
async fn abc(p: Params) -> Response {
    Response
}
