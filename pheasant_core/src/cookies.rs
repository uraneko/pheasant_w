use chrono::{DateTime, TimeDelta, Utc};

// #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
// struct Cookies(HashSet<Cookie>);

/// if no expires or max-age attrs are set then the cookie is auto expired at browser session shutdown
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, Hash)]
pub struct Cookie {
    expires: Option<DateTime<Utc>>,
    max_age: Option<TimeDelta>,
    http_only: bool,
    // requires the Secure attr
    partitioned: bool,
    secure: bool,
    path: Option<String>,
    domain: Option<String>,
    same_site: Option<SameSite>,
    key: String,
    val: String,
}

impl Cookie {
    pub fn new(k: &str, v: &str) -> Self {
        Self {
            key: k.to_owned(),
            val: v.to_owned(),
            ..Default::default()
        }
    }

    // adds an expiration datetime to the cookie
    // takes a datetim after which the cookie should be expired
    // this sets the Expires cookie attribute
    //
    // WARN the server sets this attribute using its own clock, which may differ from the client
    // side's clock,
    // it is advised to use the less error prone Max-Age attr instead
    pub fn expires(&mut self, datetime: DateTime<Utc>) -> &mut Self {
        self.expires = Some(datetime);

        self
    }

    // adds an expiration datetime to the cookie
    // takes a duration after which the cookie should be expired
    // this sets the Max-Age cookie attribute
    pub fn max_age(&mut self, delta: TimeDelta) -> &mut Self {
        self.max_age = Some(delta);

        self
    }

    // if switch is set to true then the client side can not access this cookie using javascript
    pub fn http_only(&mut self, switch: bool) -> &mut Self {
        self.http_only = switch;

        self
    }

    /// sets the request uri path that triggers this cookie to be send with the request
    /// sub paths will also be considered as matches
    pub fn path(&mut self, path: &str) -> &mut Self {
        self.path = Some(path.into());

        self
    }

    /// sets the domain to which the client will send this cookie with requests
    /// if the domain value does not include the cookie defining server, then
    /// the cookie is rejected
    /// that includes server subdomains;
    /// example.com can not send a cookie with Domain=foo.example.com
    pub fn domain(&mut self, domain: &str) -> &mut Self {
        self.domain = Some(domain.into());

        self
    }

    pub fn same_site<SS>(&mut self, ss: SS) -> &mut Self
    where
        SS: TryInto<SameSite>,
        <SS as TryInto<SameSite>>::Error: std::fmt::Debug,
    {
        self.same_site = Some(ss.try_into().unwrap());

        self
    }

    /// switch for whether the cookie should be stored in partitioned storage
    /// requires the Secure attr
    pub fn partitioned(&mut self, switch: bool) -> &mut Self {
        self.partitioned = switch;

        self
    }

    /// only send this cookie with requests coming from the `https:` scheme
    pub fn secure(&mut self, switch: bool) -> &mut Self {
        self.secure = switch;

        self
    }
}

// WARN browsers' session restore feature also restores session cookies
impl Cookie {
    pub fn format(&self) -> String {
        let mut cookie = format!("{}={}", self.key, self.val);
        let mut temp;
        if let Some(ma) = self.max_age {
            let ma = ma.num_seconds();
            temp = format!("; Max-Age={} ", ma);
            cookie.push_str(&temp);
        }

        if let Some(exp) = self.expires {
            temp = format!("; Expires={} ", exp);
            cookie.push_str(&temp)
        }

        if self.http_only {
            cookie.push_str("; HttpOnly");
        }

        if self.secure {
            cookie.push_str("; Secure");

            if self.partitioned {
                cookie.push_str("; Partitioned");
            }
        }

        if let Some(path) = &self.path {
            temp = format!("; Path={}", path);
            cookie.push_str(&temp);
        }

        if let Some(domain) = &self.domain {
            temp = format!("; Domain={}", domain);
            cookie.push_str(&temp);
        }

        if let Some(ss) = &self.same_site {
            temp = format!("; SameSite={:?}", ss);
            cookie.push_str(&temp);
        }

        cookie
    }
}

impl std::fmt::Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

// NOTE the same domain with a different scheme is considered a different domain
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, Hash)]
pub enum SameSite {
    // only send this cookie on requests made to the same site that defined it
    Strict = 1,
    // same as strict but also includes cross site requests that
    // - are top level navigation requests; i.e., they move pages
    // - use a safe http method (dont set data in the server)
    Lax = 2,
    // the cookie is send with both same site and cross site requests
    // requires the secure attr
    #[default]
    None = 0,
}

impl TryFrom<u8> for SameSite {
    type Error = ();

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::None),
            1 => Ok(Self::Strict),
            2 => Ok(Self::Lax),
            _ => Err(()),
        }
    }
}
