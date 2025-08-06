#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ParseError {
    Idna(IDNAError),
    Host(HostError),
    Url(URLError),
}

pub type ParseResult<T> = Result<T, ParseError>;

impl ParseError {
    pub fn idna(repr: u8) -> Result<Self, ()> {
        if repr > 2 {
            return Err(());
        }
        Ok(Self::Idna(unsafe {
            std::mem::transmute::<u8, IDNAError>(repr)
        }))
    }

    pub fn host(repr: u8) -> Result<Self, ()> {
        if repr > 15 {
            return Err(());
        }
        Ok(Self::Host(unsafe {
            std::mem::transmute::<u8, HostError>(repr)
        }))
    }

    pub fn url(repr: u8) -> Result<Self, ()> {
        if repr > 9 {
            return Err(());
        }
        Ok(Self::Url(unsafe {
            std::mem::transmute::<u8, URLError>(repr)
        }))
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IDNAError {
    DomainToASCII = 0,
    DomainInvalidCodePoint = 1,
    DomainToUnicode = 2,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HostError {
    HostInvalidCodePoint = 0,
    IPv4EmptyPart = 1,
    IPv4TooManyParts = 2,
    IPv4NonNumericPart = 3,
    IPv4NonDecimalPart = 4,
    IPv4OutOfRangePart = 5,
    IPv6Unclosed = 6,
    IPv6InvalidCompression = 7,
    IPv6TooManyPieces = 8,
    IPv6MultipleCompression = 9,
    IPv6InvalidCodePoint = 10,
    IPv6TooFewPieces = 11,
    IPv4InIPv6TooManyPieces = 12,
    IPv4InIPv6InvalidCodePoint = 13,
    IPv4InIPv6OutOfRangePart = 14,
    IPv4InIPv6TooFewParts = 15,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum URLError {
    InvalidURLUnit = 0,
    SpecialSchemeMissingFollowingSolidus = 1,
    MissingSchemeNonRelativeURL = 2,
    InvalidReverseSolidus = 3,
    InvalidCredentials = 4,
    HostMissing = 5,
    PortOutOfRange = 6,
    PortInvalid = 7,
    FileInvalidWindowsDriveLetter = 8,
    FileInvalidWindowsDriveLetterHost = 9,
}
