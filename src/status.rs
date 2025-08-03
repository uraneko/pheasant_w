use crate::PheasantError;

/// http response server error status
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ServerError {
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

impl TryFrom<u16> for ServerError {
    type Error = ();

    fn try_from(u: u16) -> Result<Self, Self::Error> {
        match u {
            500 => Ok(ServerError::InternalServerError),
            501 => Ok(ServerError::NotImplemented),
            502 => Ok(ServerError::BadGateway),
            503 => Ok(ServerError::ServiceUnavailable),
            504 => Ok(ServerError::GatewayTimeout),
            505 => Ok(ServerError::HTTPVersionNotSupported),
            506 => Ok(ServerError::VariantAlsoNegotiates),
            507 => Ok(ServerError::InsufficientStorage),
            508 => Ok(ServerError::LoopDetected),
            510 => Ok(ServerError::NotExtended),
            511 => Ok(ServerError::NetworkAuthenticationRequired),
            _ => Err(()),
        }
    }
}

/// http response client error status
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ClientError {
    BadRequest = 400,
    Unauthorized = 401,
    // NOTE rarely used
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    ContentTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    Imateapot = 418,
    MisdirectedRequest = 421,
    UnprocessableContent = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,
}

impl TryFrom<u16> for ClientError {
    type Error = ();

    fn try_from(u: u16) -> Result<Self, Self::Error> {
        match u {
            // client errors
            400 => Ok(ClientError::BadRequest),
            401 => Ok(ClientError::Unauthorized),
            // NOTE rarely used
            402 => Ok(ClientError::PaymentRequired),
            403 => Ok(ClientError::Forbidden),
            404 => Ok(ClientError::NotFound),
            405 => Ok(ClientError::MethodNotAllowed),
            406 => Ok(ClientError::NotAcceptable),
            407 => Ok(ClientError::ProxyAuthenticationRequired),
            408 => Ok(ClientError::RequestTimeout),
            409 => Ok(ClientError::Conflict),
            410 => Ok(ClientError::Gone),
            411 => Ok(ClientError::LengthRequired),
            412 => Ok(ClientError::PreconditionFailed),
            413 => Ok(ClientError::ContentTooLarge),
            414 => Ok(ClientError::URITooLong),
            415 => Ok(ClientError::UnsupportedMediaType),
            416 => Ok(ClientError::RangeNotSatisfiable),
            417 => Ok(ClientError::ExpectationFailed),
            418 => Ok(ClientError::Imateapot),
            421 => Ok(ClientError::MisdirectedRequest),
            422 => Ok(ClientError::UnprocessableContent),
            423 => Ok(ClientError::Locked),
            424 => Ok(ClientError::FailedDependency),
            425 => Ok(ClientError::TooEarly),
            426 => Ok(ClientError::UpgradeRequired),
            428 => Ok(ClientError::PreconditionRequired),
            429 => Ok(ClientError::TooManyRequests),
            431 => Ok(ClientError::RequestHeaderFieldsTooLarge),
            451 => Ok(ClientError::UnavailableForLegalReasons),
            _ => Err(()),
        }
    }
}

/// http response redirection status
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Redirection {
    PermanentRedirect = 308,
    TemporaryRedirect = 307,
    #[deprecated(
        note = "This response code is no longer used; but is reserved. It was used in a previous version of the HTTP/1.1 specification."
    )]
    Unused = 306,
    #[deprecated(
        note = "deprecated due to security concerns regarding in-band configuration of a proxy."
    )]
    /// Defined in a previous version of the HTTP specification
    /// to indicate that a requested response must be accessed by a proxy
    UseProxyDeprecated = 305,
    NotModified = 304,
    SeeOther = 303,
    Found = 302,
    MovedPermanently = 301,
    MultipleChoices = 300,
}

impl TryFrom<u16> for Redirection {
    type Error = ();

    fn try_from(u: u16) -> Result<Self, Self::Error> {
        match u {
            308 => Ok(Self::PermanentRedirect),
            307 => Ok(Self::TemporaryRedirect),
            // #[deprecated(
            306 => Ok(Self::Unused),
            // #[deprecated(
            // Defined in a previous version of the HTTP specification
            // to indicate that a requested response must be accessed by a proxy
            305 => Ok(Self::UseProxyDeprecated),
            304 => Ok(Self::NotModified),
            303 => Ok(Self::SeeOther),
            302 => Ok(Self::Found),
            301 => Ok(Self::MovedPermanently),
            300 => Ok(Self::MultipleChoices),
            _ => Err(()),
        }
    }
}

/// http response successful status
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Successful {
    IMUsed = 226,
    AlreadyReported = 208,
    MultiStatus = 207,
    PartialContent = 206,
    ResetContent = 205,
    NoContent = 204,
    NonAuthoritativeInformation = 203,
    Accepted = 202,
    Created = 201,
    OK = 200,
}

impl TryFrom<u16> for Successful {
    type Error = ();

    fn try_from(u: u16) -> Result<Self, Self::Error> {
        match u {
            226 => Ok(Self::IMUsed),
            208 => Ok(Self::AlreadyReported),
            207 => Ok(Self::MultiStatus),
            206 => Ok(Self::PartialContent),
            205 => Ok(Self::ResetContent),
            204 => Ok(Self::NoContent),
            203 => Ok(Self::NonAuthoritativeInformation),
            202 => Ok(Self::Accepted),
            201 => Ok(Self::Created),
            200 => Ok(Self::OK),
            _ => Err(()),
        }
    }
}

/// http response informational status
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Informational {
    EarlyHints = 103,
    ProcessingDeprecated = 102,
    SwitchingProtocols = 101,
    Continue = 100,
}

impl TryFrom<u16> for Informational {
    type Error = ();

    fn try_from(u: u16) -> Result<Self, Self::Error> {
        match u {
            103 => Ok(Self::EarlyHints),
            102 => Ok(Self::ProcessingDeprecated),
            101 => Ok(Self::SwitchingProtocols),
            100 => Ok(Self::Continue),
            _ => Err(()),
        }
    }
}

impl From<PheasantError> for Status {
    fn from(err: PheasantError) -> Self {
        match err {
            PheasantError::ClientError(ce) => Self::ClientError(ce),

            PheasantError::ServerError(se) => Self::ServerError(se),
        }
    }
}

/// implements shared behavior amongst response status
pub trait ResponseStatus {
    /// returns the status text value
    fn text(&self) -> &str;

    /// returns the status code number
    fn code(&self) -> u16;
}

impl ResponseStatus for ServerError {
    fn text(&self) -> &str {
        match self {
            Self::InternalServerError => "InternalServerError",
            Self::NotImplemented => "NotImplemented",
            Self::BadGateway => "BadGateway",
            Self::ServiceUnavailable => "ServiceUnavailable",
            Self::GatewayTimeout => "GatewayTimeout",
            Self::HTTPVersionNotSupported => "HTTPVersionNotSupported",
            Self::VariantAlsoNegotiates => "VariantAlsoNegotiates",
            Self::InsufficientStorage => "InsufficientStorage",
            Self::LoopDetected => "LoopDetected",
            Self::NotExtended => "NotExtended",
            Self::NetworkAuthenticationRequired => "NetworkAuthenticationRequired",
        }
    }

    fn code(&self) -> u16 {
        unsafe { std::mem::transmute::<Self, u16>(*self) }
    }
}

impl ResponseStatus for ClientError {
    fn text(&self) -> &str {
        match self {
            Self::BadRequest => "BadRequest",
            Self::Unauthorized => "Unauthorized",
            Self::PaymentRequired => "PaymentRequired",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "NotFound",
            Self::MethodNotAllowed => "MethodNotAllowed",
            Self::NotAcceptable => "NotAcceptable",
            Self::ProxyAuthenticationRequired => "ProxyAuthenticationRequired",
            Self::RequestTimeout => "RequestTimeout",
            Self::Conflict => "Conflict",
            Self::Gone => "Gone",
            Self::LengthRequired => "LengthRequired",
            Self::PreconditionFailed => "PreconditionFailed",
            Self::ContentTooLarge => "ContentTooLarge",
            Self::URITooLong => "URITooLong",
            Self::UnsupportedMediaType => "UnsupportedMediaType",
            Self::RangeNotSatisfiable => "RangeNotSatisfiable",
            Self::ExpectationFailed => "ExpectationFailed",
            Self::Imateapot => "Imateapot",
            Self::MisdirectedRequest => "MisdirectedRequest",
            Self::UnprocessableContent => "UnprocessableContent",
            Self::Locked => "Locked",
            Self::FailedDependency => "FailedDependency",
            Self::TooEarly => "TooEarly",
            Self::UpgradeRequired => "UpgradeRequired",
            Self::PreconditionRequired => "PreconditionRequired",
            Self::TooManyRequests => "TooManyRequests",
            Self::RequestHeaderFieldsTooLarge => "RequestHeaderFieldsTooLarge",
            Self::UnavailableForLegalReasons => "UnavailableForLegalReasons",
        }
    }

    fn code(&self) -> u16 {
        unsafe { std::mem::transmute::<Self, u16>(*self) }
    }
}

impl ResponseStatus for Redirection {
    fn text(&self) -> &str {
        match self {
            Self::PermanentRedirect => "PermanentRedirect",
            Self::TemporaryRedirect => "TemporaryRedirect",
            Self::Unused => "Unused",
            Self::UseProxyDeprecated => "UseProxyDeprecated",
            Self::NotModified => "NotModified",
            Self::SeeOther => "SeeOther",
            Self::Found => "Found",
            Self::MovedPermanently => "MovedPermanently",
            Self::MultipleChoices => "MultipleChoices",
        }
    }

    fn code(&self) -> u16 {
        unsafe { std::mem::transmute::<Self, u16>(*self) }
    }
}

impl ResponseStatus for Successful {
    fn text(&self) -> &str {
        match self {
            Self::IMUsed => "IMUsed",
            Self::AlreadyReported => "AlreadyReported",
            Self::MultiStatus => "MultiStatus",
            Self::PartialContent => "PartialContent",
            Self::ResetContent => "ResetContent",
            Self::NoContent => "NoContent",
            Self::NonAuthoritativeInformation => "NonAuthoritativeInformation",
            Self::Accepted => "Accepted",
            Self::Created => "Created",
            Self::OK => "OK",
        }
    }

    fn code(&self) -> u16 {
        // NOTE this is safe
        // trust me bro
        unsafe { std::mem::transmute::<Self, u16>(*self) }
    }
}

impl ResponseStatus for Informational {
    fn text(&self) -> &str {
        match self {
            Self::EarlyHints => "EarlyHints",
            Self::ProcessingDeprecated => "ProcessingDeprecated",
            Self::SwitchingProtocols => "SwitchingProtocols",
            Self::Continue => "Continue",
        }
    }

    fn code(&self) -> u16 {
        unsafe { std::mem::transmute::<Self, u16>(*self) }
    }
}

/// enum wrapping all response status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Status {
    Informational(Informational),
    Successful(Successful),
    Redirection(Redirection),
    ClientError(ClientError),
    ServerError(ServerError),
}

impl TryFrom<u16> for Status {
    type Error = ();

    fn try_from(u: u16) -> Result<Self, Self::Error> {
        if u < 200 {
            <u16 as TryInto<Informational>>::try_into(u).map(|i| Self::Informational(i))
        } else if u < 300 {
            <u16 as TryInto<Successful>>::try_into(u).map(|s| Self::Successful(s))
        } else if u < 400 {
            <u16 as TryInto<Redirection>>::try_into(u).map(|re| Self::Redirection(re))
        } else if u < 500 {
            <u16 as TryInto<ClientError>>::try_into(u).map(|ce| Self::ClientError(ce))
        } else {
            <u16 as TryInto<ServerError>>::try_into(u).map(|se| Self::ServerError(se))
        }
    }
}

impl ResponseStatus for Status {
    fn text(&self) -> &str {
        match self {
            Self::Redirection(r) => r.text(),
            Self::Successful(s) => s.text(),
            Self::Informational(i) => i.text(),
            Self::ClientError(ce) => ce.text(),
            Self::ServerError(se) => se.text(),
        }
    }

    fn code(&self) -> u16 {
        match self {
            Self::Redirection(r) => r.code(),
            Self::Successful(s) => s.code(),
            Self::Informational(i) => i.code(),
            Self::ClientError(ce) => ce.code(),
            Self::ServerError(se) => se.code(),
        }
    }
}

/// error response status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ErrorStatus {
    Client(ClientError),
    Server(ServerError),
}

impl ResponseStatus for ErrorStatus {
    fn text(&self) -> &str {
        match self {
            Self::Client(ce) => ce.text(),
            Self::Server(se) => se.text(),
        }
    }

    fn code(&self) -> u16 {
        match self {
            Self::Client(ce) => ce.code(),
            Self::Server(se) => se.code(),
        }
    }
}

// WARN this is wrong
// if u == unrecognizable variant repr, then this fn returns the highest u16 repr variant
impl TryFrom<u16> for ErrorStatus {
    type Error = ();

    fn try_from(u: u16) -> Result<Self, Self::Error> {
        if u < 500 {
            <u16 as TryInto<ClientError>>::try_into(u).map(|ce| Self::Client(ce))
        } else {
            <u16 as TryInto<ServerError>>::try_into(u).map(|se| Self::Server(se))
        }
    }
}

/// request accpetance response status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GoodStatus {
    Redirection(Redirection),
    Informational(Informational),
    Successful(Successful),
}

impl ResponseStatus for GoodStatus {
    fn text(&self) -> &str {
        match self {
            Self::Informational(i) => i.text(),
            Self::Successful(s) => s.text(),
            Self::Redirection(re) => re.text(),
        }
    }

    fn code(&self) -> u16 {
        match self {
            Self::Informational(i) => i.code(),
            Self::Successful(s) => s.code(),
            Self::Redirection(re) => re.code(),
        }
    }
}

// enum Status {
//     Reject(ErrorStatus),
//     Accept(AcceptStatus),
// }
