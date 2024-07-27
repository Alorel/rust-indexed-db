use rand::prelude::*;

pub mod open;

#[cfg(feature = "list-databases")]
pub mod delete_and_list;
pub mod delete_obj_store;
pub mod obj_store_create;
pub mod transaction;

#[cfg(feature = "version-change")]
pub mod versionchange;

pub(crate) fn random_db_versions() -> [u8; 2] {
    const I8_MAX: u8 = i8::MAX as u8;
    let mut rng = thread_rng();
    [rng.gen_range(1..I8_MAX), rng.gen_range(I8_MAX..=u8::MAX)]
}
