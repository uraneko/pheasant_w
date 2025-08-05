use pheasant_uri::{Token, lex};

#[test]
fn empty() {
    let preset: Vec<Token> = vec![];
    let lexed = lex("");

    assert_eq!(preset, lexed);
}

#[test]
fn mail() {
    let preset = vec![
        Token::word("mailto".chars()),
        Token::Colon,
        Token::word("username".chars()),
        Token::AddressSign,
        Token::word("example".chars()),
    ];
    let lexed = lex("mailto:username@example");

    assert_eq!(lexed, preset);
}

#[test]
fn glob() {
    let preset = vec![Token::Asterisk];
    let lexed = lex("*");

    assert_eq!(preset, lexed);
}

#[test]
fn origin() {
    let lexed = lex("example.uri");
    let preset = vec![
        Token::word("example".chars()),
        Token::Dot,
        Token::word("uri".chars()),
    ];

    assert_eq!(lexed, preset);
}

#[test]
fn path() {
    let lexed = lex("/this/is/a/path/only/uri?with=some&query=params");
    let preset = vec![
        Token::Slash,
        Token::word("this".chars()),
        Token::Slash,
        Token::word("is".chars()),
        Token::Slash,
        Token::word("a".chars()),
        Token::Slash,
        Token::word("path".chars()),
        Token::Slash,
        Token::word("only".chars()),
        Token::Slash,
        Token::word("uri".chars()),
        Token::QuestionMark,
        Token::word("with".chars()),
        Token::Equality,
        Token::word("some".chars()),
        Token::AmperSand,
        Token::word("query".chars()),
        Token::Equality,
        Token::word("params".chars()),
    ];

    assert_eq!(preset, lexed);
}

#[test]
fn full() {
    let lexed = lex("https://www.domain.com:9865/path/to/some/resource?this=query&then=parameter");
    let preset = vec![
        Token::word("https".chars()),
        Token::Colon,
        Token::Slash,
        Token::Slash,
        Token::word("www".chars()),
        Token::Dot,
        Token::word("domain".chars()),
        Token::Dot,
        Token::word("com".chars()),
        Token::Colon,
        Token::number("9865".chars()),
        Token::Slash,
        Token::word("path".chars()),
        Token::Slash,
        Token::word("to".chars()),
        Token::Slash,
        Token::word("some".chars()),
        Token::Slash,
        Token::word("resource".chars()),
        Token::QuestionMark,
        Token::word("this".chars()),
        Token::Equality,
        Token::word("query".chars()),
        Token::AmperSand,
        Token::word("then".chars()),
        Token::Equality,
        Token::word("parameter".chars()),
    ];

    assert_eq!(lexed, preset);
}
