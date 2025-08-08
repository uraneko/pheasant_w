use pheasant_uri::Url;

fn main() {
    let url = "drive_path".parse::<Url>().unwrap();

    println!("{:#?}", url);
}
