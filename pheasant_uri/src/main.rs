use pheasant_uri::{Origin, Resource, Route, Url};

fn main() {
    // let url = "http://www.the.web.site.com.:9998/ftree/ftro/ftrei?path=src&ssr&file=_File_1eed6_1&dir=_Dir_1eed6_13&children=_Children_1eed6_22&parent=_Parent_1eed6_20";
    let url = "/ftree?path=src&ssr&file=_File_1gri4_1&dir=_Dir_1gri4_13&chidren=_Children_1gri4_22&parent=_Parent_1gri4_20";
    // let url = "http://127.0.0.1:9998/dh?truncate";
    // let url = "/dh?truncate";

    let res = url.parse::<Url>().unwrap();

    println!("{}", url);
    println!("{:?}:{}", res, res.sequence() == url);
    println!("{}", res.sequence());
}
