use pheasant_core::HeaderMap;
use pheasant_core::Request;
use pheasant_uri::Origin;

pub struct RequestOrigin(Option<Origin>);

impl From<&Request> for RequestOrigin {
    fn from(req: &Request) -> Self {
        let Some(ori) = req.header::<Origin>("Origin") else {
            return RequestOrigin(None);
        };

        Self(Some(ori))
    }
}

impl RequestOrigin {
    pub fn origin(&self) -> Option<&Origin> {
        self.0.as_ref()
    }
}
