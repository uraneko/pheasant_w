#[repr(u16)]
#[derive(Debug)]
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
#[derive(Debug)]
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
pub enum Informational {
    EarlyHints = 103,
    ProcessingDeprecated = 102,
    SwitchingProtocols = 101,
    Continue = 100,
}
