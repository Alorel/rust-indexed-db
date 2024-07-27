pub use conditional_build::BuildDyn;

#[cfg(feature = "_serialise-deserialise-dyn")]
pub use conditional_build::{DeserialiseDyn, SerialiseDyn};

mod conditional_build;
pub mod dummy_data;
pub mod init;

pub fn random_str() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[inline]
pub fn init_logging() {
    console_log::init_with_level(log::Level::Debug).expect("log init");
}
