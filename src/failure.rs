use std::pin::Pin;

use crate::{ErrorStatus, Mime, Response, ResponseStatus};

pub struct Failure {
    mime: Option<Mime>,
    status: u16,
    fail: BoxFun,
}

unsafe impl Send for Failure {}
unsafe impl Sync for Failure {}

// the future return type
type BoxFut<'a> = Pin<Box<dyn Future<Output = Response> + Send + 'a>>;

// the wrapper function type
type BoxFun = Box<dyn Fn() -> BoxFut<'static> + Send + Sync>;

impl Failure {
    pub fn new<F, O>(status: u16, mime: Option<Mime>, fun: F) -> Self
    where
        // probably give the Fn an input of ErrorStatus
        F: Fn() -> O + Send + Sync + 'static,
        O: Future<Output = Response> + Send + 'static,
    {
        Self {
            status,
            mime,
            fail: Box::new(move || Box::pin(fun())),
        }
    }

    pub fn mime(&self) -> Option<&Mime> {
        self.mime.as_ref()
    }

    pub fn code(&self) -> u16 {
        self.status
    }

    pub fn status(&self) -> crate::Status {
        self.status.try_into().unwrap()
    }

    pub fn fail(&self) -> &BoxFun {
        &self.fail
    }
}
