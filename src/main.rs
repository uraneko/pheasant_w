use pheasant_uri::Url;

fn main() {
    let url = "/".parse::<Url>().unwrap();

    println!("{:#?}", url.sequence());
}
