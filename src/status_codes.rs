use crate::PheasantError;

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

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Informational {
    EarlyHints = 103,
    ProcessingDeprecated = 102,
    SwitchingProtocols = 101,
    Continue = 100,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Status {
    Informational(Informational),
    Successful(Successful),
    Redirection(Redirection),
    ClientError(ClientError),
    ServerError(ServerError),
}

impl From<PheasantError> for Status {
    fn from(err: PheasantError) -> Self {
        match err {
            PheasantError::ClientError(ce) => Self::ClientError(ce),

            PheasantError::ServerError(se) => Self::ServerError(se),
        }
    }
}

pub trait ResponseStatus {
    fn text(&self) -> &str;

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
