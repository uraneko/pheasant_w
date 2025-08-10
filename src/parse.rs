use crate::{ParseError, Query, Scheme, Token, Url, lex};

#[derive(Debug)]
pub struct PrologueParser {
    tokens: std::vec::IntoIter<Token>,
    scheme: Option<Scheme>,
}

#[derive(Debug)]
pub enum Parser {
    Prologue(PrologueParser),
    Domain(DomainParser),
    Path(PathParser),
    Absolute(AbsoluteParser),
    Epilogue(EpilogueParser),
}

impl Parser {
    pub fn new(uri: &str) -> Option<Self> {
        let tokens = lex(uri);

        Some(Self::Prologue(PrologueParser::new(tokens)?))
    }

    pub fn is_prologue(&self) -> bool {
        match self {
            Self::Prologue(_) => true,
            _ => false,
        }
    }

    pub fn parse(mut self) -> Result<Url, ParseError> {
        if !self.is_prologue() {
            unreachable!("the only public methods are new and parse, noone can get here")
        };

        while !self.is_epilogue() {
            match self {
                Self::Epilogue(_) => (),
                Self::Prologue(p) => self = p.parse()?,
                Self::Domain(p) => self = p.parse()?,
                Self::Path(p) => self = p.parse()?,
                Self::Absolute(p) => self = p.parse()?,
            }
        }

        let Self::Epilogue(epi) = self else {
            unreachable!("other case handled 4 lines ago");
        };

        Ok(epi.parse())
    }
}

impl PrologueParser {
    fn new(tokens: Vec<Token>) -> Option<Self> {
        if tokens.is_empty() {
            return None;
        }

        Some(Self {
            tokens: tokens.into_iter(),
            scheme: None,
        })
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn parse(mut self) -> Result<Parser, ParseError> {
        let Some(tok) = self.next() else {
            unreachable!("self.next() should only be called from here and self is not empty");
        };

        match tok {
            // scheme | path
            Token::Seq(s) => self.parse_seq(s),
            // domain | path
            Token::Slash => self.parse_slash(),
            _ => ParseError::url(0).map(|_| unsafe { std::mem::zeroed() }),
        }
    }

    fn parse_seq(mut self, seq: String) -> Result<Parser, ParseError> {
        if Scheme::is_scheme(&seq) {
            self.parse_scheme(seq)?;

            Ok(Parser::Domain(DomainParser::new(self)))
        } else {
            Ok(Parser::Path(PathParser::with_seq(self, seq)))
        }
    }

    fn parse_slash(mut self) -> Result<Parser, ParseError> {
        match self.next() {
            // domain incoming
            Some(Token::Slash) => Ok(Parser::Domain(DomainParser::scheme_relative(self))),
            // path starts from value s
            Some(Token::Seq(s)) => Ok(Parser::Path(PathParser::with_sep(self, s))),
            // error unexpected token
            Some(_) => Err(ref_res(ParseError::url(0)).unwrap()),
            // 1 slash token stream means a root route (path absolute url)
            // with no query and no fragment
            // Route("/")
            None => Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end))),
        }
    }

    fn parse_scheme(&mut self, seq: String) -> Result<(), ParseError> {
        let scheme = seq.parse::<Scheme>();
        if scheme.is_ok() {
            self.scheme = scheme.ok();
            return Ok(());
        }

        scheme.map(|_| ())
    }
}

// TODO scheme is case insensitive
// TODO same document references are also valid uris: '#fragment'
// TODO use this array to run a check before lexing
// if it fails then the url is invalid and we exit with url error 0
const GENERALLY_VALID: [char; 84] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '-', '.', '_', '~', ':', '/', '?', '#', '[', ']', '@', '!', '$', '&',
    '\'', '(', ')', '*', '+', ',', ';', '=',
];

trait Parsing {
    fn is_epilogue(&self) -> bool {
        false
    }

    fn next(&mut self) -> Option<Token> {
        None
    }

    fn is_valid_seq(s: &str) -> bool {
        true
    }

    fn is_valid_sep(sep: &Token) -> bool {
        true
    }
}

impl Parsing for DomainParser {
    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }
}

impl Parsing for EpilogueParser {
    fn is_epilogue(&self) -> bool {
        true
    }
}

impl Parsing for Parser {
    fn is_epilogue(&self) -> bool {
        match self {
            Self::Epilogue(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct DomainParser {
    tokens: std::vec::IntoIter<Token>,
    scheme: Option<Scheme>,
    domain: Vec<String>,
    port: Option<u16>,
    is_relative: bool,
    trailing_dot: bool,
}

impl DomainParser {
    fn new(parser: PrologueParser) -> Self {
        Self {
            tokens: parser.tokens,
            scheme: parser.scheme,
            domain: vec![],
            port: None,
            is_relative: false,
            trailing_dot: false,
        }
    }

    fn scheme_relative(parser: PrologueParser) -> Self {
        Self {
            tokens: parser.tokens,
            scheme: parser.scheme,
            domain: vec![],
            port: None,
            is_relative: true,
            trailing_dot: false,
        }
    }

    fn push(&mut self, s: String) -> Result<(), ParseError> {
        // check if string is a domain name valid string
        Self::is_valid_seq(&s)
            .then(|| {
                self.domain.push(s);
                ()
            })
            .ok_or_else(|| ref_res(ParseError::url(0)).unwrap())
    }

    fn parse(mut self) -> Result<Parser, ParseError> {
        match self.next() {
            // scheme relative url
            Some(Token::Seq(s)) if self.is_relative => self.parse_domain(s),
            // scheme relative url but it's not
            Some(Token::Seq(_)) if !self.is_relative => Err(ref_res(ParseError::url(0)).unwrap()),
            // scheme absolute url
            Some(Token::Colon) if !self.is_relative => {
                let s = self.validate_absolute()?;

                self.parse_domain(s)
            }
            // scheme absolute url but it's actually relative
            Some(Token::Colon) if !self.is_relative => Err(ref_res(ParseError::url(0)).unwrap()),
            Some(_) => Err(ref_res(ParseError::url(0)).unwrap()),
            // basically url = '//', which we interpret as '/' which is just a root path Route
            None if self.is_relative => Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end))),
            // scheme (e.g., 'https' or 'foobar') followed by nothing -> url too short, expected ':'
            _ => Err(ref_res(ParseError::url(0)).unwrap()),
        }
    }

    // if self is absolute domain we check that '//' is next
    // then we return the next seq token to be returned to parse_domain
    fn validate_absolute(&mut self) -> Result<String, ParseError> {
        let [Some(Token::Slash), Some(Token::Slash)] = [self.next(), self.next()] else {
            return Err(ref_res(ParseError::url(0)).unwrap());
        };

        match self.next() {
            Some(Token::Seq(s)) => Ok(s),
            Some(token) if Self::is_valid_sep(&token) => Ok(token.as_str().to_owned()),
            Some(token) if !Self::is_valid_sep(&token) => Err(ref_res(ParseError::url(0)).unwrap()),
            Some(_) => unreachable!("Self::is_valid_sep should handle all tokens exhaustively"),
            None => Err(ref_res(ParseError::url(0)).unwrap()),
        }
    }

    fn parse_domain(mut self, s: String) -> PRslt {
        self.push(s)?;

        match self.next() {
            Some(Token::Dot) => self.parse_domain_name(),
            Some(Token::Seq(_)) => unreachable!(
                "lexer cant generate 2 sequences in a row, it wasnt written like that... probably"
            ),
            // NOTE have to have a dot after something else in the domain name
            // localhost broke that rule
            Some(Token::Colon) => self.parse_port(),
            // // found '/' or '?' indicating the start of a path or query
            Some(sep) if [Token::Slash, Token::QuestionMark].contains(&sep) => {
                Ok(Parser::Absolute(AbsoluteParser::new(self)))
            }
            // // found '#', we expect a fragment next
            Some(Token::Pound) => Ok(Parser::Epilogue(EpilogueParser::new(self, parse_fragment))),
            // // TODO see which of the separator tokens are valid in a domain name
            // // and handle them when found: push token char value to self.domain.last
            Some(_) => Err(ref_res(ParseError::url(0)).unwrap()),
            // // a domain name can not be 1 level long
            None => return Err(ref_res(ParseError::url(0)).unwrap()),
            // _ => return Err(ref_res(ParseError::url(0)).unwrap()),
        }
    }

    fn parse_domain_name(mut self) -> PRslt {
        let mut trailing = true;
        loop {
            match self.next() {
                Some(Token::Dot) => trailing = true,
                // if a host valid separator char is found
                // then it should be pushed into self.domain.last

                // if '/' then path start else if '?' then query start
                // NOTE query start signifies an implicit path start
                Some(Token::Slash) | Some(Token::QuestionMark) => {
                    return Ok(Parser::Absolute(AbsoluteParser::new(self)));
                }
                // if '#' then start of a fragment
                Some(Token::Pound) => {
                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_fragment)));
                }
                Some(Token::Colon) => {
                    self.trailing_dot = trailing;

                    return self.parse_port();
                }

                Some(token) => {
                    match token {
                        Token::Seq(seq) => {
                            if !Self::is_valid_seq(&seq) {
                                return Err(ref_res(ParseError::url(0)).unwrap());
                            }

                            if trailing {
                                self.domain.push(seq)
                            } else {
                                self.domain.last_mut().map(|s| s.push_str(&seq));
                            }
                        }

                        token => {
                            if !Self::is_valid_sep(&token) {
                                return Err(ref_res(ParseError::url(0)).unwrap());
                            }

                            self.domain
                                .last_mut()
                                .map(|s| s.push(token.to_char().unwrap()));
                        }
                    }
                    trailing = false;
                }

                None => {
                    self.trailing_dot = trailing;

                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end)));
                }
            }
        }
    }

    fn parse_port(mut self) -> PRslt {
        match self.next() {
            Some(Token::Seq(s)) => self.assign_port(s).unwrap(), // should be '?'
            // expected port after ':', found something else
            Some(_) => return Err(ref_res(ParseError::url(0)).unwrap()),
            // expected port after ':', found nothing, uri too short
            None => return Err(ref_res(ParseError::url(0)).unwrap()),
        }

        self.parse_next()
    }

    fn assign_port(&mut self, s: String) -> Result<(), ParseError> {
        let port = s.parse::<u16>();
        if port.is_err() {
            return Err(ref_res(ParseError::url(0)).unwrap());
        }
        self.port = Some(port.unwrap());

        Ok(())
    }

    fn parse_next(mut self) -> PRslt {
        match self.next() {
            // if '/' then path start else if '?' then query start
            // NOTE query start signifies an implicit path start
            Some(Token::Slash) | Some(Token::QuestionMark) => {
                Ok(Parser::Absolute(AbsoluteParser::new(self)))
            }
            // if '#' then start of a fragment
            Some(Token::Pound) => Ok(Parser::Epilogue(EpilogueParser::new(self, parse_fragment))),
            // uri is done, move to epilogue
            None => Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end))),
            // expected '/', '?', '#' or nothing found something else
            Some(_) => Err(ref_res(ParseError::url(0)).unwrap()),
        }
    }
}

type PRslt = Result<Parser, ParseError>;

#[derive(Debug)]
pub struct PathParser {
    tokens: std::vec::IntoIter<Token>,
    path: Vec<String>,
    query: Option<Query>,
    sep_start: bool,
    trailing_slash: bool,
}

impl Parsing for PathParser {
    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }
}

impl PathParser {
    fn with_seq(parser: PrologueParser, s: String) -> Self {
        Self {
            tokens: parser.tokens,
            path: vec![s],
            query: None,
            sep_start: true,
            trailing_slash: false,
        }
    }

    fn with_sep(parser: PrologueParser, token: String) -> Self {
        Self {
            tokens: parser.tokens,
            path: vec![token],
            query: None,
            sep_start: false,
            trailing_slash: false,
        }
    }

    fn push(&mut self, s: String) -> Result<(), ParseError> {
        // check if string is a uri path valid string
        Self::is_valid_seq(&s)
            .then(|| {
                self.path.push(s);

                ()
            })
            .ok_or_else(|| ref_res(ParseError::url(0)).unwrap())
    }

    fn parse(mut self) -> PRslt {
        match self.next() {
            // we parse path
            Some(Token::Seq(seq)) if self.sep_start => {
                self.path.last_mut().map(|s| s.push_str(&seq));

                self.parse_path()
            }
            // unreachable, assuming that path (cant start with a separator token/started with seq)
            Some(Token::Seq(_)) if !self.sep_start => Err(ref_res(ParseError::url(0)).unwrap()),
            Some(Token::QuestionMark) => self.parse_query(),
            // root path then fragment: domain#fragment | domain/#fragment
            Some(Token::Pound) => {
                return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_fragment)));
            }
            Some(Token::Slash) => {
                let s = self.find_segment()?;
                if s.is_none() {
                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end)));
                }
                self.push(s.unwrap())?;

                self.parse_path()
            }
            Some(token) => {
                self.path
                    .last_mut()
                    .map(|s| s.push(token.to_char().unwrap()));
                self.parse_path()
            }

            None => Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end))),
        }
    }

    fn find_segment(&mut self) -> Result<Option<String>, ParseError> {
        loop {
            match self.next() {
                Some(Token::Slash) => (),
                Some(Token::Seq(s)) => return Ok(Some(s)),
                Some(token) => return Ok(Some(token.as_str().to_owned())),
                None => return Ok(None),
            }
        }
    }

    fn parse_path(mut self) -> PRslt {
        let mut trailing = false;

        loop {
            match self.next() {
                Some(Token::Seq(seq)) => {
                    if trailing {
                        self.push(seq)?;
                    } else {
                        self.path.last_mut().map(|s| s.push_str(&seq));
                    }
                }
                Some(Token::Slash) => trailing = true,
                Some(Token::QuestionMark) => return self.parse_query(),
                Some(Token::Pound) => {
                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_fragment)));
                }

                Some(token) => {
                    if !Self::is_valid_sep(&token) {
                        return Err(ref_res(ParseError::url(0)).unwrap());
                    }

                    if trailing {
                        self.push(token.as_str().to_owned())?;
                    } else {
                        self.path
                            .last_mut()
                            .map(|s| s.push(token.to_char().unwrap()));
                    }
                }

                None => {
                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end)));
                }
                _ => unreachable!("match_sep should satisfy separator token exhaustively"),
            }
        }
    }

    fn is_valid_query_comp(comp: &Token) -> bool {
        true
    }

    fn parse_query(mut self) -> PRslt {
        let mut query = Query::default();
        let mut atkey = true;
        let [mut key, mut val] = [format!(""), format!("")];
        loop {
            match self.next() {
                Some(Token::Seq(ref s)) => {
                    if atkey {
                        key.push_str(s);
                    } else {
                        val.push_str(s);
                    }
                }

                Some(Token::AmperSand) => {
                    if atkey {
                        query.insert_iter_attr(key.drain(..));
                    } else {
                        query.insert_iter_param(key.drain(..), val.drain(..));
                    }

                    atkey = true;
                }

                Some(Token::Equality) => atkey = false,

                Some(Token::Pound) => {
                    if atkey {
                        query.insert_iter_attr(key.drain(..));
                    } else {
                        query.insert_iter_param(key.drain(..), val.drain(..));
                    }
                    self.query = Some(query);

                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_fragment)));
                }

                Some(token) => {
                    if !Self::is_valid_query_comp(&token) {
                        return Err(ref_res(ParseError::url(0)).unwrap());
                    }

                    let ch = token.to_char().unwrap();
                    if atkey {
                        key.push(ch);
                    } else {
                        val.push(ch)
                    }
                }

                None => {
                    if atkey {
                        query.insert_iter_attr(key.drain(..));
                    } else {
                        query.insert_iter_param(key.drain(..), val.drain(..));
                    }
                    self.query = Some(query);

                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end)));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct AbsoluteParser {
    tokens: std::vec::IntoIter<Token>,
    scheme: Option<Scheme>,
    domain: Vec<String>,
    port: Option<u16>,
    path: Vec<String>,
    query: Option<Query>,
    trailing_dot: bool,
    sep_start: bool,
}

impl Parsing for AbsoluteParser {
    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }
}

impl AbsoluteParser {
    fn new(parser: DomainParser) -> Self {
        Self {
            tokens: parser.tokens,
            scheme: parser.scheme,
            domain: parser.domain,
            port: parser.port,
            path: vec![],
            query: None,
            sep_start: true,
            trailing_dot: parser.trailing_dot,
        }
    }

    fn push(&mut self, s: String) -> Result<(), ParseError> {
        // check if string is a uri path valid string
        Self::is_valid_seq(&s)
            .then(|| {
                self.path.push(s);

                ()
            })
            .ok_or_else(|| ref_res(ParseError::url(0)).unwrap())
    }

    fn assign_query(&mut self, s: String) -> Result<(), ParseError> {
        let query = s.parse::<Query>();
        if query.is_err() {
            return Err(ref_res(ParseError::url(0)).unwrap());
        }
        self.query = Some(query.unwrap());

        Ok(())
    }

    fn parse(mut self) -> PRslt {
        match self.next() {
            // we parse path
            Some(Token::Seq(s)) if self.sep_start => {
                self.push(s)?;

                self.parse_path()
            }
            // unreachable, assuming that path (cant start with a separator token/started with seq)
            Some(Token::Seq(_)) if !self.sep_start => Err(ref_res(ParseError::url(0)).unwrap()),
            // root path then query: domain?query | domain/?query
            Some(Token::QuestionMark) => self.parse_query(),
            // root path then fragment: domain#fragment | domain/#fragment
            Some(Token::Pound) => {
                return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_fragment)));
            }
            Some(Token::Slash) => {
                let s = self.find_segment()?;
                if s.is_none() {
                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end)));
                }
                self.push(s.unwrap())?;

                self.parse_path()
            }
            Some(token) => {
                self.push(token.as_str().to_owned())?;
                self.parse_path()
            }
            // basically url = '//', which we interpret as '/' which is just a root path Route
            None => Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end))),
        }
    }

    fn find_segment(&mut self) -> Result<Option<String>, ParseError> {
        loop {
            match self.next() {
                Some(Token::Slash) => (),
                Some(Token::Seq(s)) => return Ok(Some(s)),
                Some(token) => return Ok(Some(token.as_str().to_owned())),
                None => return Ok(None),
            }
        }
    }

    fn parse_path(mut self) -> PRslt {
        let mut trailing = false;

        loop {
            match self.next() {
                Some(Token::Seq(seq)) => {
                    if trailing {
                        self.push(seq)?;
                    } else {
                        self.path.last_mut().map(|s| s.push_str(&seq));
                    }
                }
                Some(Token::Slash) => trailing = true,
                Some(Token::QuestionMark) => return self.parse_query(),
                Some(Token::Pound) => {
                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_fragment)));
                }

                Some(token) => {
                    if !Self::is_valid_sep(&token) {
                        return Err(ref_res(ParseError::url(0)).unwrap());
                    }

                    if trailing {
                        self.push(token.as_str().to_owned())?;
                    } else {
                        self.path
                            .last_mut()
                            .map(|s| s.push(token.to_char().unwrap()));
                    }
                }

                None => return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end))),
                _ => unreachable!("match_sep should satisfy separator token exhaustively"),
            }
        }
    }

    fn is_valid_query_comp(comp: &Token) -> bool {
        true
    }

    fn parse_query(mut self) -> PRslt {
        let mut query = Query::default();
        let mut atkey = true;
        let [mut key, mut val] = [format!(""), format!("")];
        loop {
            match self.next() {
                Some(Token::Seq(ref s)) => {
                    if atkey {
                        key.push_str(s);
                    } else {
                        val.push_str(s);
                    }
                }

                Some(Token::AmperSand) => {
                    if atkey {
                        query.insert_iter_attr(key.drain(..));
                    } else {
                        query.insert_iter_param(key.drain(..), val.drain(..));
                    }

                    atkey = true;
                }

                Some(Token::Equality) => atkey = false,

                Some(Token::Pound) => {
                    if atkey {
                        query.insert_iter_attr(key.drain(..));
                    } else {
                        query.insert_iter_param(key.drain(..), val.drain(..));
                    }
                    self.query = Some(query);

                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_fragment)));
                }

                Some(token) => {
                    if !Self::is_valid_query_comp(&token) {
                        return Err(ref_res(ParseError::url(0)).unwrap());
                    }

                    let ch = token.to_char().unwrap();
                    if atkey {
                        key.push(ch);
                    } else {
                        val.push(ch)
                    }
                }

                None => {
                    if atkey {
                        query.insert_iter_attr(key.drain(..));
                    } else {
                        query.insert_iter_param(key.drain(..), val.drain(..));
                    }
                    self.query = Some(query);

                    return Ok(Parser::Epilogue(EpilogueParser::new(self, parse_end)));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct EpilogueParser {
    scheme: Option<Scheme>,
    domain: Option<Vec<String>>,
    port: Option<u16>,
    path: Option<Vec<String>>,
    query: Option<Query>,
    fragment: Option<String>,
}

impl EpilogueParser {
    fn new<P>(p: P, fragment_resolution: fn(std::vec::IntoIter<Token>) -> Option<String>) -> Self
    where
        P: Into<UrlParts>,
    {
        let parts = p.into();
        let (tokens, scheme, domain, port, path, query) = parts.spread();
        let fragment = fragment_resolution(tokens);

        Self {
            scheme,
            domain,
            port,
            path,
            query,
            fragment,
        }
    }

    fn parse(self) -> Url {
        Url::from_parts(
            self.scheme,
            self.domain,
            self.port,
            self.path,
            self.query,
            self.fragment,
        )
    }
}

#[derive(Debug, Default)]
pub struct UrlParts(
    // tokens
    std::vec::IntoIter<Token>,
    // scheme
    Option<Scheme>,
    // domain
    Option<Vec<String>>,
    // port
    Option<u16>,
    // path
    Option<Vec<String>>,
    // query
    Option<Query>,
);

impl UrlParts {
    fn spread(
        self,
    ) -> (
        std::vec::IntoIter<Token>,
        Option<Scheme>,
        Option<Vec<String>>,
        Option<u16>,
        Option<Vec<String>>,
        Option<Query>,
    ) {
        (self.0, self.1, self.2, self.3, self.4, self.5)
    }
}

impl From<DomainParser> for UrlParts {
    fn from(parser: DomainParser) -> Self {
        UrlParts(
            parser.tokens,
            parser.scheme,
            Some(parser.domain),
            parser.port,
            None,
            None,
        )
    }
}

impl From<PathParser> for UrlParts {
    fn from(parser: PathParser) -> Self {
        UrlParts(
            parser.tokens,
            None,
            None,
            None,
            Some(parser.path),
            parser.query,
        )
    }
}

impl From<AbsoluteParser> for UrlParts {
    fn from(parser: AbsoluteParser) -> Self {
        UrlParts(
            parser.tokens,
            parser.scheme,
            Some(parser.domain),
            parser.port,
            Some(parser.path),
            parser.query,
        )
    }
}

impl From<PrologueParser> for UrlParts {
    fn from(_parser: PrologueParser) -> Self {
        Self::default()
    }
}

// initializes the parse process
// a url can only start in a sequence or a slash

// parse the start of some url part
// could be the start of
// - the scheme
// - the domain
// - the path
// - the query
//      - a query key
//      - a query val
// - the fragment

pub(crate) fn ref_res<T, E>(res: Result<T, E>) -> Result<E, T> {
    match res {
        Err(e) => Ok(e),
        Ok(o) => Err(o),
    }
}

fn parse_fragment(tokens: std::vec::IntoIter<Token>) -> Option<String> {
    Some(
        tokens
            .map(|t| {
                if let Token::Seq(s) = t {
                    s
                } else {
                    t.as_str().to_owned()
                }
            })
            .collect::<String>(),
    )
}

fn parse_end(_tokens: std::vec::IntoIter<Token>) -> Option<String> {
    None
}
