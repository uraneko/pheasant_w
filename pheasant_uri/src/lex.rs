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
    let mut last = 0;

    while let Some(idx) = s.find(ch) {
        v.push((idx + last, ch));
        last += idx + 1;
        s = &s[idx + 1..];
    }

    v
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Token {
    Seq(String),
    Slash,
    Dot,
    Colon,
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

    pub fn to_char(&self) -> Option<char> {
        match self {
            Self::Seq(s) => None,
            token => Some(match token {
                Self::Seq(_) => unreachable!("matched out 2 line ago"),
                Self::QuestionMark => '?',
                Self::Pound => '#',
                Self::Colon => ':',
                Self::Slash => '/',
                Self::AddressSign => '@',
                Self::Equality => '=',
                Self::AmperSand => '&',
                Self::Dot => '.',
            }),
        }
    }
}

// TODO return Result + error accordingly following the standard
pub fn lex(mut s: &str) -> Vec<Token> {
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
