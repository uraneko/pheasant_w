pub struct Uri {
    value: String,
    kind: UriKind,
}

impl Uri {}

pub enum UriKind {
    // *
    WildCard,
    // full uri
    Origin,
    // standalone path
    Path,
    // IPv4
    IPv4,
    // IPv6
    IPv6,
    // email
    Email,
}

fn interpret() -> Uri {
    todo!()
}
