//! Original compatibility code for privately supplied Marvel Heroes assets.
//! Modules document their observed format scope; this is not a retail-parity claim.

pub const TARGET_BUILD: &str = "1.52.0.1700";
pub const BUILD_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod calligraphy;
pub mod options;
pub mod sip;
pub mod upk;
