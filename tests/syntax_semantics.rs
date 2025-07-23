use pheasant_macro_get::get;

struct Service;

impl Service {
    fn new<F, O, R>(m: Method, r: &str, mime: &str, f: F) -> Self
    where
        F: Fn(R) -> O + Send + Sync + 'static,
        O: Future<Output = Vec<u8>> + Send + 'static,
        R: From<Request>,
    {
        Self
    }
}

struct Request {}

impl From<Request> for String {
    fn from(r: Request) -> Self {
        format!("")
    }
}

enum Method {
    Get,
}

#[get("/")]
#[mime("text/html")]
async fn abc(y: String) -> Vec<u8> {
    vec![y.len() as u8]
}

#[get("/hello")]
async fn hello(name: String) -> Vec<u8> {
    format!("<h1>hello {}</h1>", name).into_bytes()
}

#[test]
fn test() {
    let _v = vec![abc, hello];
}
