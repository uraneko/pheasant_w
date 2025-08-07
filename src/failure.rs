use std::pin::Pin;

use crate::{ErrorStatus, Mime, Response, ResponseStatus};

pub struct Failure {
    mime: Option<Mime>,
    status: ErrorStatus,
    fail: BoxFun,
}

unsafe impl Send for Failure {}
unsafe impl Sync for Failure {}

// the future return type
type BoxFut<'a> = Pin<Box<dyn Future<Output = Response> + Send + 'a>>;

// the wrapper function type
type BoxFun = Box<dyn Fn() -> BoxFut<'static> + Send + Sync>;

impl Failure {
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
