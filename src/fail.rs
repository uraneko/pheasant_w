use std::pin::Pin;

use mime::Mime;

use crate::{ErrorStatus, Response, ResponseStatus};

pub struct Fail {
    mime: Option<Mime>,
    status: ErrorStatus,
    fail: BoxFun,
}

unsafe impl Send for Fail {}
unsafe impl Sync for Fail {}

// the future return type
type BoxFut<'a> = Pin<Box<dyn Future<Output = Response> + Send + 'a>>;

// the wrapper function type
type BoxFun = Box<dyn Fn() -> BoxFut<'static> + Send + Sync>;

impl Fail {
    pub fn new<F, O>(status: ErrorStatus, mime: &str, fun: F) -> Self
    where
        F: Fn() -> O + Send + Sync + 'static,
        O: Future<Output = Response> + Send + 'static,
    {
        Self {
            status,
            mime: mime.parse().ok(),
            fail: Box::new(move || Box::pin(fun())),
        }
    }

    pub fn mime(&self) -> Option<&Mime> {
        self.mime.as_ref()
    }

    pub fn code(&self) -> u16 {
        self.status.code()
    }

    pub fn fail(&self) -> &BoxFun {
        &self.fail
    }
}
