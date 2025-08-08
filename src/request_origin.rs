use pheasant_core::Request;
use pheasant_uri::Origin;

pub struct RequestOrigin(Option<Origin>);

impl From<&Request> for RequestOrigin {
    fn from(req: &Request) -> Self {
        let Some(ori) = req.param("Origin") else {
            return RequestOrigin(None);
        };

        let ori = ori.parse::<Origin>().unwrap();

        Self(Some(ori))
    }
}

impl RequestOrigin {
    pub fn origin(&self) -> Option<&Origin> {
        self.0.as_ref()
    }
}
