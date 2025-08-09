use pheasant_uri::Url;

fn main() {
    let url = "http://localhost:9998/ftree?path=src&ssr&file=_File_1eed6_1&dir=_Dir_1eed6_13&chidren=_Children_1eed6_22&parent=_Parent_1eed6_20".parse::<Url>().unwrap();

    println!("{:#?}", url);
}
