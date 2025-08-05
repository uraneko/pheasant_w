enum URIError {}

enum WriterError {
    Idna(IDNAError),
    Host(HostParsingError),
    Url(URLParsingError),
}

enum IDNAError {}

enum HostParsingError {}

enum URLParsingError {}
