// use pheasant_uri::{Scheme, Uri};
//
// #[test]
// fn glob() {
//     assert_eq!("*".parse::<Uri>().unwrap(), Uri::Glob);
// }
//
// #[test]
// fn origin() {
//     let uri = "example.uri";
//     let from_parts = Uri::origin(None, "example.uri".to_owned(), None);
//
//     assert_eq!(uri.parse::<Uri>().unwrap(), from_parts);
// }
//
// #[test]
// fn path() {
//     let uri = "/this/is/a/path/only/uri?with=some&query=params";
//     let from_parts = Uri::path(
//         "/this/is/a/path/only/uri".to_owned(),
//         Some("with=some&query=params".to_owned()),
//     );
//
//     assert_eq!(uri.parse::<Uri>().unwrap(), from_parts);
// }
//
// #[test]
// fn full() {
//     let uri = "https://www.domain.com:9865/path/to/some/resource?this=query&then=parameter";
//     let from_parts = Uri::full(
//         Some(Scheme::Https),
//         "www.domain.com".to_owned(),
//         Some(9865),
//         Some("/path/to/some/resource".to_owned()),
//         Some("this=query&then=parameter".to_owned()),
//     );
//     assert_eq!(uri.parse::<Uri>().unwrap(), from_parts)
// }
