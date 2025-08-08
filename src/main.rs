use pheasant_macro_fail::fail;

fn main() {}

#[fail(404)]
#[mime("text/toml")]
async fn not_found() -> Vec<u8> {
    std::fs::read("Cargo.toml").unwrap()
}
