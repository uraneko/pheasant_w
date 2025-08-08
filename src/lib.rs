pub mod mining;
pub mod parsers;
pub mod plumber;
pub mod poet;
pub mod request_origin;

pub use mining::Mining;
pub use parsers::{CorsAttr, IntAttr, StrAttr, StrVec};
pub use plumber::Plumber;
pub use poet::{FailureInscriptions, Poet, ServiceInscriptions};
pub use request_origin::RequestOrigin;
// public re-export of Method so that
// macros dont have to add core as a dependency only to access this type
pub use pheasant_core::Method;

// should have been internal only (pub(crate)) exports
// but the compiler appears unable to guess which Plumber::new & Poet::new
// to call based on the arg type and the args themselves;
pub use plumber::{FailurePlumber, ServicePlumber};
pub use poet::{FailurePoet, ServicePoet};
