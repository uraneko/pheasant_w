use pheasant_macro_get::get;

struct Params;

#[get("/")]
#[cors(methods = [get, post], headers = ["Content-Type", "Origin", "Host"], origins = "*", max_age = 213124)] // <- Options preflight req
#[re("/index.html")] // <- redirection 303 see other to this service
async fn abc(p: Params) -> Response {
    Response
}

fn main() {}
