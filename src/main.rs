use pheasant_macro_get::get;

#[get("123")]
async fn abc(x: u8, y: String) -> Vec<bool> {
    vec![]
}

fn main() {}
