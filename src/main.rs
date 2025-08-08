use pheasant_uri::Resource;

fn main() {
    let url = "/drive/file_tree?path=src&ssr".parse::<Resource>().unwrap();

    println!("{:#?}", url);
}
