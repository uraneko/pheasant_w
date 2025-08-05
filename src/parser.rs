use std::collections::{HashMap, HashSet};

use crate::{ParseError, Token, lex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Scheme {
    Http,
    Https,
    Ws,
    Wss,
    File,
    Ftp,
}

impl std::str::FromStr for Scheme {
    type Err = ParseError;
    fn from_str(s: &str) -> ParseResult<Self> {
        match s.to_uppercase().as_str() {
            "HTTP" => Ok(Self::Http),
            "HTTPS" => Ok(Self::Https),
            "WS" => Ok(Self::Ws),
            "WSS" => Ok(Self::Wss),
            "FILE" => Ok(Self::File),
            "FTP" => Ok(Self::Ftp),
            _ => Err(ParseError::url(0).unwrap()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Query {
    params: HashMap<String, String>,
    attrs: HashSet<String>,
}

impl Query {
    fn insert_param(&mut self, k: &str, v: &str) {
        self.params.insert(k.to_owned(), v.to_owned());
    }

    fn insert_attr(&mut self, a: &str) {
        self.attrs.insert(a.to_owned());
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn attrs(&self) -> &HashSet<String> {
        &self.attrs
    }
}

impl Query {
    fn from_colls(map: HashMap<&str, &str>, set: HashSet<&str>) -> Self {
        Query {
            params: map.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            attrs: set.into_iter().map(|a| a.into()).collect(),
        }
    }
}

impl std::str::FromStr for Query {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult<Self> {
        let mut query = Query::default();
        str_to_pairs(&mut query, s);

        Ok(query)
    }
}

// parses the query params into key -> value pairs
fn str_to_pairs(query: &mut Query, s: &str) {
    s.split('&')
        // BUG this crashes the server when uri query is badly formatted
        // TODO scan query after getting request and return ClientError::BadRequest if query is faulty
        .map(|e| str_to_pair(e))
        .for_each(|[k, v]| {
            if v.is_empty() {
                query.insert_attr(k);
            } else {
                query.insert_param(k, v);
            }
        });
}

// NOTE this handles the pain points of parse_query
// the check for `=` garentees the operation's success
fn str_to_pair(p: &str) -> [&str; 2] {
    if p.contains('=') {
        p.splitn(2, '=').collect::<Vec<&str>>().try_into().unwrap()
    } else {
        // TODO possibly make a HashSet of bool params alongside the HashMap of k -> v pairs
        [p, ""]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Url {
    scheme: Option<Scheme>,
    domain: Option<Vec<String>>,
    port: Option<u16>,
    path: Option<Vec<String>>,
    query: Option<Query>,
    fragment: Option<String>,
}

impl Url {
    fn update_scheme(&mut self, scheme: Option<Scheme>) {
        self.scheme = scheme;
    }

    fn update_domain(&mut self, domain: Vec<String>) {
        self.domain = Some(domain);
    }

    fn update_port(&mut self, port: u16) {
        self.port = Some(port);
    }

    fn update_path(&mut self, path: Vec<String>) {
        self.path = Some(path);
    }

    fn update_query(&mut self, query: Query) {
        self.query = Some(query);
    }

    fn update_fragment(&mut self, frag: String) {
        self.fragment = Some(frag);
    }
}

type ParseResult<T> = Result<T, ParseError>;

fn parse_init<I>(mut toks: I, mut url: Url) -> ParseResult<Url>
where
    I: Iterator<Item = Token>,
{
    match toks.next().unwrap() {
        // either scheme... or path...
        Token::Word(w) => {
            let scheme = w.parse::<Scheme>().ok();
            url.update_scheme(scheme);
            if url.scheme.is_none() {
                // parse_path_absolute
                url.domain = None;

                return parse_path_absolute(toks, url, w);
            } else {
                // parse_scheme_relative
                let Some(Token::Colon) = toks.next() else {
                    return Err(ParseError::url(0).unwrap());
                };

                return parse_scheme_relative(toks, url);
            }
        }
        // either //{domain} or /{path}
        Token::Slash => {
            let next = toks.next().ok_or(ParseError::url(0).unwrap())?;
            match next {
                // //{domain}
                Token::Slash => {
                    return parse_scheme_relative(toks, url);
                }
                // /{path}
                Token::Word(path) => {
                    parse_path_absolute(toks, url, path);
                }
                _ => return Err(ParseError::url(0).unwrap()),
            }
        }
        _ => panic!(),
    }

    todo!()
}

#[derive(Debug, Eq, PartialEq, Hash)]
enum Last {
    Sep,
    Item,
}

// //{domain}{maybe path}
fn parse_scheme_relative<I>(mut toks: I, mut url: Url) -> ParseResult<Url>
where
    I: Iterator<Item = Token>,
{
    let (Some(Token::Slash), Some(Token::Slash)) = (toks.next(), toks.next()) else {
        return Err(ParseError::url(0).unwrap());
    };

    let mut domain = Vec::new();
    let mut last = Last::Item;
    while let Some(t) = toks.next() {
        match t {
            Token::Word(d) => {
                if last == Last::Item && !domain.is_empty() {
                    return Err(ParseError::url(0).unwrap());
                }
                domain.push(d);
                last = Last::Item;
            }
            Token::Dot => {
                if last == Last::Sep {
                    return Err(ParseError::url(0).unwrap());
                }
                last = Last::Sep;
            }
            Token::Colon => {
                if last == Last::Sep {
                    return Err(ParseError::url(0).unwrap());
                }
                // port is next
                url.update_domain(domain);
                return parse_port(toks, url);
            }
            Token::Slash => {
                if last == Last::Sep {
                    return Err(ParseError::url(0).unwrap());
                }
                // path is next
                url.update_domain(domain);
                return parse_path(toks, url);
            }
            _ => return Err(ParseError::url(0).unwrap()),
        }
    }
    url.update_domain(domain);

    Ok(url)
}

// /{path}
fn parse_path_absolute<I>(mut toks: I, mut url: Url, p: String) -> ParseResult<Url>
where
    I: Iterator<Item = Token>,
{
    let mut path = Vec::default();
    path.push(p);
    let mut last = Last::Item;
    while let Some(t) = toks.next() {
        match t {
            Token::Word(s) => {
                if last == Last::Item {
                    return Err(ParseError::url(0).unwrap());
                }
                path.push(s);
                last = Last::Item;
            }
            Token::Slash => {
                if last == Last::Sep {
                    return Err(ParseError::url(0).unwrap());
                }
                last = Last::Sep;
            }
            Token::QuestionMark => {
                if last == Last::Sep {
                    return Err(ParseError::url(0).unwrap());
                }
                // query is next
                url.update_path(path);
                return parse_query(toks, url);
            }
            Token::Pound => {
                if last == Last::Sep {
                    return Err(ParseError::url(0).unwrap());
                }
                url.update_path(path);
                return parse_fragment(toks, url);
            }
            _ => return Err(ParseError::url(0).unwrap()),
        }
    }
    url.update_path(path);

    Ok(url)
}

fn parse_path<I>(mut toks: I, mut url: Url) -> ParseResult<Url>
where
    I: Iterator<Item = Token>,
{
    let mut path = Vec::default();
    let mut last = Last::Sep;
    while let Some(t) = toks.next() {
        match t {
            Token::Word(s) => {
                if last == Last::Item {
                    return Err(ParseError::url(0).unwrap());
                }
                path.push(s);
                last = Last::Item;
            }
            Token::Slash => {
                if last == Last::Sep && !path.is_empty() {
                    return Err(ParseError::url(0).unwrap());
                }
                last = Last::Sep;
            }
            Token::QuestionMark => {
                if last == Last::Sep {
                    return Err(ParseError::url(0).unwrap());
                }
                // query is next
                url.update_path(path);
                return parse_query(toks, url);
            }
            Token::Pound => {
                if last == Last::Sep {
                    return Err(ParseError::url(0).unwrap());
                }
                url.update_path(path);
                return parse_fragment(toks, url);
            }
            _ => return Err(ParseError::url(0).unwrap()),
        }
    }
    url.update_path(path);

    Ok(url)
}

fn parse_port<I>(mut toks: I, mut url: Url) -> ParseResult<Url>
where
    I: Iterator<Item = Token>,
{
    let Some(Token::Number(num)) = toks.next() else {
        return Err(ParseError::url(0).unwrap());
    };

    let Ok(port) = num.parse::<u16>() else {
        return Err(ParseError::url(7).unwrap());
    };
    url.update_port(port);

    if let Some(t) = toks.next() {
        match t {
            Token::Slash => {
                // path
                return parse_path(toks, url);
            }
            Token::QuestionMark => {
                // query
                return parse_query(toks, url);
            }
            Token::Pound => {
                // fragment
                return parse_fragment(toks, url);
            }
            _ => Err(ParseError::url(0).unwrap()),
        }
    } else {
        return Ok(url);
    }
}

fn parse_query<I>(mut toks: I, mut url: Url) -> ParseResult<Url>
where
    I: Iterator<Item = Token>,
{
    let mut temp = String::new();
    while let Some(t) = toks.next() {
        if t == Token::Pound {
            let query = temp.parse::<Query>()?;
            url.update_query(query);
            return parse_fragment(toks, url);
        }

        temp.push_str(t.as_str());
    }
    let query = temp.parse::<Query>()?;
    url.update_query(query);

    Ok(url)
}

fn parse_fragment<I>(mut toks: I, mut url: Url) -> ParseResult<Url>
where
    I: Iterator<Item = Token>,
{
    let mut frag = String::new();
    while let Some(t) = toks.next() {
        frag.push_str(t.as_str());
    }
    url.update_fragment(frag);

    // fragment comes at the end so we're done
    Ok(url)
}

fn parse_url<I>(toks: I) -> ParseResult<Url>
where
    I: Iterator<Item = Token>,
{
    let url = Url::default();

    parse_init(toks, url)
}

impl std::str::FromStr for Url {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult<Self> {
        let toks = lex(s);

        parse_url(toks.into_iter())
    }
}

impl Url {
    pub fn path_absolute(
        path: Vec<&str>,
        query: Option<(HashMap<&str, &str>, HashSet<&str>)>,
        fragment: Option<String>,
    ) -> Self {
        Self {
            path: Some(path.into_iter().map(|s| s.into()).collect()),
            query: query.map(|(params, attrs)| Query::from_colls(params, attrs)),
            fragment,
            ..Default::default()
        }
    }

    pub fn scheme_relative(
        domain: Vec<String>,
        port: Option<u16>,
        path: Option<Vec<String>>,
        query: Option<(HashMap<&str, &str>, HashSet<&str>)>,
        fragment: Option<String>,
    ) -> Self {
        Self {
            domain: Some(domain),
            port,
            path,
            query: query.map(|(params, attrs)| Query::from_colls(params, attrs)),
            fragment,
            ..Default::default()
        }
    }

    pub fn absolute(
        scheme: Scheme,
        domain: Vec<String>,
        port: Option<u16>,
        path: Option<Vec<String>>,
        query: Option<(HashMap<&str, &str>, HashSet<&str>)>,
        fragment: Option<String>,
    ) -> Self {
        Self {
            domain: Some(domain),
            port,
            path,
            query: query.map(|(params, attrs)| Query::from_colls(params, attrs)),
            fragment,
            scheme: Some(scheme),
        }
    }
}

impl Url {
    pub fn scheme(&self) -> Option<Scheme> {
        self.scheme
    }

    pub fn take_domain(&mut self) -> Option<Vec<String>> {
        let Some(ref mut domain) = self.domain else {
            return None;
        };

        Some(std::mem::take(domain))
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn take_path(&mut self) -> Option<Vec<String>> {
        let Some(ref mut path) = self.path else {
            return None;
        };

        Some(std::mem::take(path))
    }

    pub fn take_query(&mut self) -> Option<Query> {
        let Some(ref mut query) = self.query else {
            return None;
        };

        Some(std::mem::take(query))
    }

    pub fn take_fragment(&mut self) -> Option<String> {
        let Some(ref mut fragment) = self.fragment else {
            return None;
        };

        Some(std::mem::take(fragment))
    }
}
