//! Original compatibility code for privately supplied Marvel Heroes assets.
//! Modules document their observed format scope; this is not a retail-parity claim.

pub const TARGET_BUILD: &str = "1.52.0.1700";
pub const BUILD_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod animation;
pub mod avm1;
pub mod calligraphy;
pub mod camera;
pub mod frontend;
pub mod gfx;
pub mod integrity;
pub mod localization;
pub mod material;
pub mod options;
pub mod prototype_catalog;
pub mod prototype_values;
#[cfg(feature = "renderer")]
pub mod render;
pub mod sip;
pub mod skeletal_mesh;
pub mod static_mesh;
pub mod texture;
pub mod texture_cache;
pub mod upk;
#[cfg(feature = "window")]
pub mod window;
pub mod world_scene;
pub mod world_transform;
