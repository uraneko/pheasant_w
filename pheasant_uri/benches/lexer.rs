use std::time::{Duration, Instant};

fn main() {
    let s = "http://127.0.0.1:9998/ftree?path=src&ssr&file=_File_1eed6_1&dir=_Dir_1eed6_13&chidren=_Children_1eed6_22&parent=_Parent_1eed6_20";

    println!("{}", s);

    // NOTE realease mode: by_map seems twice as fast
    println!("by_while---> {:?}", bench(s, by_while));
    println!("by_map   ---> {:?}", bench(s, by_map));
    // println!("{}", by_map(s) == by_while(s));
    println!("by_while\n{:?}", by_while(s));
    // println!("by_map\n{:#?}", by_map(s));
}

fn bench(s: &str, f: fn(s: &str) -> Vec<Token>) -> Duration {
    let start = Instant::now();
    f(s);

    Instant::now().duration_since(start)
}

macro_rules! token {
    ($s: expr) => {
        match $s {
            '/' => Token::Slash,
            ':' => Token::Colon,
            '?' => Token::QuestionMark,
            '#' => Token::Pound,
            '@' => Token::AddressSign,
            '=' => Token::Equality,
            '&' => Token::AmperSand,
            '.' => Token::Dot,
            _ => panic!("declmacro, unexpected char token value"),
        }
    };
}

const SEPS: [char; 8] = ['@', '/', ':', '?', '#', '=', '&', '.'];

fn find_all(mut s: &str, ch: char) -> Vec<(usize, char)> {
    let mut v = vec![];
    let mut acc = 0;

    while let Some(idx) = s.find(ch) {
        v.push((idx + acc, ch));
        acc += idx + 1;
        s = &s[idx + 1..];
    }

    v
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Token {
    Seq(String),
    Slash,
    Colon,
    Dot,
    QuestionMark,
    Pound,
    AddressSign,
    AmperSand,
    Equality,
}

impl Token {
    pub fn seq(s: &str) -> Self {
        Self::Seq(s.to_owned())
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Seq(s) => s,
            Self::QuestionMark => "?",
            Self::Pound => "#",
            Self::Colon => ":",
            Self::Slash => "/",
            Self::AddressSign => "@",
            Self::Equality => "=",
            Self::AmperSand => "&",
            Self::Dot => ".",
        }
    }
}

fn by_while(mut s: &str) -> Vec<Token> {
    let mut breakpoints = SEPS
        .into_iter()
        .map(|sep| find_all(s, sep))
        .flatten()
        .collect::<Vec<(usize, char)>>();
    breakpoints.sort_by(|a, b| a.0.cmp(&b.0));

    let mut v = vec![];
    let mut last = 0;
    let mut iter = breakpoints.into_iter();
    while let Some((idx, ch)) = iter.next() {
        if idx > last {
            v.extend([Token::seq(&s[..idx - last]), token!(ch)]);
        } else {
            v.push(token!(ch));
        }
        s = &s[idx + 1 - last..];
        last = idx + 1;
    }
    v.push(Token::seq(s));

    v
}

fn by_map(mut s: &str) -> Vec<Token> {
    let mut breakpoints = SEPS
        .into_iter()
        .map(|sep| find_all(s, sep))
        .flatten()
        .collect::<Vec<(usize, char)>>();
    breakpoints.sort_by(|a, b| a.0.cmp(&b.0));

    let mut last = 0;

    let mut v = breakpoints
        .into_iter()
        .map(|(idx, ch)| {
            let toks = if idx > last {
                Some(vec![Token::seq(&s[..idx - last]), token!(ch)])
            } else {
                Some(vec![token!(ch)])
            };

            s = &s[idx + 1 - last..];
            last = idx + 1;

            toks
        })
        .map(|toks| toks.unwrap())
        .flatten()
        .collect::<Vec<Token>>();

    v.push(Token::seq(s));

    v
}
