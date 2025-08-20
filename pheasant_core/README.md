<h1>pheasant</h1>
pheasant is a http web server framework written in rust

[<img alt="crates.io" src="https://img.shields.io/crates/v/pheasant.svg?style=for-the-badge&color=E4004
6&logo=rust&labelColor=3a3a3a" height="25">](https://crates.io/crates/pheasant)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-pheasant-495c9f?style=for-the-badge&logo=
docsdotrs&labelColor=3a3a3a" height="25">](https://docs.rs/pheasant)
[<img alt="build:test" src="https://img.shields.io/github/actions/workflow/status/uraneko/pheasant/rust
-ci.yml?branch=main&style=for-the-badge&labelColor=3a3a3a" height="25">](https://github.com/uraneko/pheasant/actions?query=branch%3Amain)
[<img alt="license" src="https://img.shields.io/github/license/uraneko/pheasant?style=for-the-badge&lab
elColor=3a3a3a&color=ECD53F" height="25">](https://github.com/uraneko/pheasant/blob/main/LICENSE)

## ToC
- [Goals](#Goals)
- [Features](#Features)
- [MSRV](#MSRV)
- [License](#License)

###
### Goals
- http1.1 protocol (origin server)
- http2 protocol (origin server)
- worker pool / multithreaded
- async operations
- GET POST PUT PATCH DELETE methods
- tls / ssl HTTPS
- middleware
- databases support
- http server caching 

### 
### Features
- http 1.1 request parsing + response auto generation
- http request redirection 
- http client/server error responses
- services as async functions 
- `get` attribute macro 

###
### Usage

####
#### Install
```bash
cargo add pheasant_core
```

###
### MSRV 
1.88.0

###
### License 
<a href="LICENSE">MIT</a>
