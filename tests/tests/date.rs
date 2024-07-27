use crate::prelude::*;
use idb_fut::date::*;

#[wasm_bindgen_test]
pub async fn serialisation() {
    // Strip nanos
    let now = {
        let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        UNIX_EPOCH + Duration::from_secs(dur.as_secs())
    };

    let as_js = now.serialise_dyn().unwrap();
    let as_rs = SystemTime::deserialise_dyn(as_js).unwrap();

    assert_eq!(as_rs, now);
}
