use pheasant_uri::Url;

fn main() {
    let url = "https://domain.com/path/leading/somewhere?query=needs&params#then fragment"
        .parse::<Url>()
        .unwrap();

    println!("{:#?}", url);
}
