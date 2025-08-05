#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Scheme {
    #[default]
    Http,
    Https,
}

// TODO add support for fragments
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Uri {
    Glob,
    Full {
        scheme: Option<Scheme>,
        domain: String,
        port: Option<u16>,
        path: Option<String>,
        query: Option<String>,
    },
    Origin {
        scheme: Option<Scheme>,
        domain: String,
        port: Option<u16>,
    },
    Path {
        path: String,
        query: Option<String>,
    },
}

pub fn normalize_path(path: &mut String) {
    *path = if !path.starts_with('/') {
        format!("/{}", path)
    } else {
        path.to_string()
    };
}

pub fn normalize_path_opt(path: &mut Option<String>) {
    if let Some(p) = path {
        *path = Some(if !p.starts_with('/') {
            format!("/{}", p)
        } else {
            p.to_string()
        });
    }
}

impl Uri {
    pub fn from_parts(
        scheme: Option<Scheme>,
        domain: Option<String>,
        port: Option<u16>,
        mut path: Option<String>,
        query: Option<String>,
    ) -> Result<Self, ()> {
        normalize_path_opt(&mut path);
        println!("------->{:?}", path);
        match [domain.is_some(), path.is_some()] {
            [false, true] => Ok(Self::Path {
                path: path.unwrap(),
                query,
            }),
            [true, true] => Ok(Self::Full {
                scheme,
                domain: domain.unwrap(),
                query,
                path,
                port,
            }),
            [true, false] => Ok(Self::Origin {
                scheme,
                domain: domain.unwrap(),
                port,
            }),
            [false, false] => Err(()),
        }
    }

    pub fn full(
        scheme: Option<Scheme>,
        domain: String,
        port: Option<u16>,
        mut path: Option<String>,
        query: Option<String>,
    ) -> Self {
        normalize_path_opt(&mut path);
        Self::Full {
            scheme,
            domain,
            port,
            path,
            query,
        }
    }

    pub fn origin(scheme: Option<Scheme>, domain: String, port: Option<u16>) -> Self {
        Self::Origin {
            scheme,
            domain,
            port,
        }
    }

    pub fn path(mut path: String, query: Option<String>) -> Self {
        normalize_path(&mut path);
        Self::Path { path, query }
    }

    pub fn is_glob(&self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(&Self::Glob)
    }

    pub fn is_origin(&self) -> bool {
        std::mem::discriminant(self)
            == std::mem::discriminant(&"example.com".parse::<Self>().unwrap())
    }
}

impl std::str::FromStr for Uri {
    type Err = ();

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if s == "*" {
            return Ok(Self::Glob);
        }

        let scheme = if s.starts_with("http://") {
            s = s.strip_prefix("http://").unwrap();
            Some(Scheme::Http)
        } else if s.starts_with("https://") {
            s = s.strip_prefix("https://").unwrap();
            Some(Scheme::Https)
        } else {
            None
        };

        let [uri, query] = if s.contains('?') {
            s.splitn(2, '?')
                .map(|s| Some(s))
                .collect::<Vec<Option<&str>>>()
                .try_into()
                .unwrap()
        } else {
            [Some(s), None]
        };
        let query = query.map(|q| q.to_owned());
        let uri = uri.unwrap();

        let port_sep = uri.find(":");
        let path_sep = uri.find("/");
        let contains_port = match [port_sep, path_sep] {
            [Some(poi), Some(pai)] => poi < pai,
            [None, _] => false,
            [Some(_), None] => true,
        };
        let [uri, p2] = if contains_port {
            uri.splitn(2, ':')
                .map(|s| Some(s))
                .collect::<Vec<Option<&str>>>()
                .try_into()
                .unwrap()
        } else {
            [Some(uri), None]
        };
        let [port, path] = if let Some(popa) = p2 {
            if popa.contains('/') {
                popa.splitn(2, '/')
                    .map(|s| Some(s))
                    .collect::<Vec<Option<&str>>>()
                    .try_into()
                    .unwrap()
            } else {
                [Some(popa), None]
            }
        } else {
            [None, None]
        };
        let port = port.map(|p| p.parse::<u16>().unwrap());
        let path = path.map(|p| p.to_owned());
        println!("{:?}//{:?}", port, path);

        let domain = uri.unwrap();
        let domain = if domain.is_empty() {
            None
        } else {
            Some(domain.to_owned())
        };

        Self::from_parts(scheme, domain, port, path, query)
    }
}
