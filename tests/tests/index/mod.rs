macro_rules! open_idx {
    ($db: expr, $mode: ident > $idx: ident) => {
        open_tx!($db, $mode > (tx, store));
        let $idx = store.index(&store.name()).expect("index()");
    };
}

pub mod create;
pub mod query_source;
