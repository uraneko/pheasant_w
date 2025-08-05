use pheasant_uri::SyntaxTree;

fn main() {
    let url = "https://domain.com/path/leading/somewhere?query=needs&params#then fragment"
        .parse::<SyntaxTree>()
        .unwrap();

    println!("{:#?}", url);
}
