use std::ops::RangeInclusive;
pub struct Parser {}

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Token {
    Word(String),
    Number(String),
    PercentEncoded(String),
    WhiteSpace,
    QuestionMark,
    Pound,
    Colon,
    SemiColon,
    Comma,
    Dot,
    Slash,
    Percent,
    Asterisk,
    AddressSign,
    Equality,
    AmperSand,
    Dollar,
    SingleQuote,
    PluSign,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    ExclamationMark,
}

impl Token {
    pub fn word<I>(i: I) -> Self
    where
        I: Iterator<Item = char>,
    {
        Self::Word(i.collect())
    }

    pub fn number<I>(i: I) -> Self
    where
        I: Iterator<Item = char>,
    {
        Self::Number(i.collect())
    }
}

struct SyntaxTree {}

macro_rules! token {
    ($s: expr) => {
        match $s {
            '/' => Token::Slash,
            ':' => Token::Colon,
            ';' => Token::SemiColon,
            ',' => Token::Comma,
            '?' => Token::QuestionMark,
            '!' => Token::ExclamationMark,
            '#' => Token::Pound,
            '%' => Token::Percent,
            '*' => Token::Asterisk,
            ' ' => Token::WhiteSpace,
            '@' => Token::AddressSign,
            '=' => Token::Equality,
            '&' => Token::AmperSand,
            '$' => Token::Dollar,
            '\'' => Token::SingleQuote,
            '+' => Token::PluSign,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '.' => Token::Dot,
            _ => panic!("declmacro, unexpected char token value")
        }
    };
}

const SYMBOLS: [char; 21] = [
    '/', ':', ';', ',', '?', '!', '#', '%', '*', ' ', '@', '=', '&', '$', '\'', '+', '[', ']', '(',
    ')', '.',
];
const DIGITS: RangeInclusive<char> = '0'..='9';
const UC: RangeInclusive<char> = 'A'..='Z';
const LC: RangeInclusive<char> = 'a'..='z';

fn is_letter(ch: char) -> bool {
    UC.contains(&ch) || LC.contains(&ch)
}
fn is_digit(ch: char) -> bool {
    DIGITS.contains(&ch)
}

fn is_symbol(ch: char) -> bool {
    SYMBOLS.contains(&ch)
}

fn group_letters_and_push_tok<I>(
    chars: &mut I,
    group: &mut Vec<char>,
    toks: &mut Vec<Token>,
) -> Option<char>
where
    I: Iterator<Item = char>,
{
    while let Some(letter) = chars.next() {
        if !is_letter(letter) {
            let tok = Token::word(group.drain(..));
            toks.push(tok);

            match letter {
                digit if is_digit(digit) => {
                    group.push(digit);
                    group_digits_and_push_tok(chars, group, toks);
                }
                sym if is_symbol(sym) => return Some(sym),
                _ => panic!("matching token after exiting word, got unexpected token kind"),
            }
        }

        group.push(letter);
    }
    // if this word token is the last token in the input
    // then we shouldnt forget to push it into toks
    let tok = Token::word(group.drain(..));
    toks.push(tok);

    None
}

fn group_digits_and_push_tok<I>(
    chars: &mut I,
    group: &mut Vec<char>,
    toks: &mut Vec<Token>,
) -> Option<char>
where
    I: Iterator<Item = char>,
{
    while let Some(digit) = chars.next() {
        if !is_digit(digit) {
            let tok = Token::number(group.drain(..));
            toks.push(tok);

            match digit {
                letter if is_letter(letter) => {
                    group.push(letter);
                    group_letters_and_push_tok(chars, group, toks);
                }
                sym if is_symbol(sym) => return Some(sym),
                _ => panic!("matching token after exiting number, got unexpected token kind"),
            }
        }

        group.push(digit);
    }
    // if this number token is the last token in the input
    // then we shouldnt forget to push it into toks
    let tok = Token::number(group.drain(..));
    toks.push(tok);

    None
}

fn match_tok_after_word(t: char) {}
fn match_tok_after_number(t: char) {}

pub fn lex(s: &str) -> Vec<Token> {
    let mut chars = s.chars();
    let mut toks = vec![];
    let mut group = vec![];
    while let Some(ch) = chars.next() {
        match ch {
            sym if is_symbol(sym) => toks.push(token!(sym)),

            letter if is_letter(letter) => {
                group.push(letter);
                let tok = group_letters_and_push_tok(&mut chars, &mut group, &mut toks);
                let Some(tok) = tok else { return toks };
                toks.push(token!(tok));
            }

            digit if is_digit(digit) => {
                group.push(digit);
                let tok = group_digits_and_push_tok(&mut chars, &mut group, &mut toks);
                let Some(tok) = tok else { return toks };
                toks.push(token!(tok));
            }
            err => panic!("lex(): didnt expect this token: `{}`", err),
        }
    }

    toks
}

fn parse() -> SyntaxTree {
    todo!()
}

fn interpret() -> Uri {
    todo!()
}
