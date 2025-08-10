use pheasant_uri::parse::Parser;

fn main() {
    let url = "http://www.the.web.site.com.:9998/ftree/ftro/ftrei?path=src&ssr&file=_File_1eed6_1&dir=_Dir_1eed6_13&children=_Children_1eed6_22&parent=_Parent_1eed6_20";

    println!("{:?}", Parser::new(url).unwrap().parse());
}
