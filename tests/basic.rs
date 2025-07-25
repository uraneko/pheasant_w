use pheasant_macro_get::get;

mod pheasant_core {

    pub struct Service;

    pub trait IntoRoutes {}

    impl<'a, T> IntoRoutes for &'a T where T: IntoIterator<Item = &'a str> {}
    impl IntoRoutes for [&'static str; 0] {}

    impl Service {
        pub fn new<F, I, O, R>(m: Method, r: &str, re: I, mime: &str, f: F) -> Self
        where
            F: Fn(R) -> O + Send + Sync + 'static,
            I: IntoRoutes,
            O: Future<Output = Vec<u8>> + Send + 'static,
            R: From<Request>,
        {
            Self
        }
    }

    pub struct Request {}

    impl From<Request> for String {
        fn from(r: Request) -> Self {
            format!("")
        }
    }

    pub enum Method {
        Get,
    }
}

#[get("/")]
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
