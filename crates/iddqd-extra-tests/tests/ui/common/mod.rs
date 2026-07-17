pub(super) use iddqd::{
    Comparable, Equivalent, Feed, ForLt, IdOrdItem, IdOrdMap, id_upcast,
};
pub(super) use serde::Serialize;

thread_local!(
    pub(super) static S: ::core::cell::Cell<&'static str> =
        const { ::core::cell::Cell::new("unset") };
);

#[derive(Comparable, Equivalent, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Foo<'a>(pub(super) &'a str);

#[derive(Serialize)]
pub(super) struct Bar(String);

impl IdOrdItem for Bar {
    type Key = ForLt![<'a> = Foo<'a>];

    fn key(&self) -> Feed<'_, Self::Key> {
        Foo(&self.0)
    }

    id_upcast!();
}

pub(super) fn bad_map(s: &str) -> IdOrdMap<Bar> {
    ::core::iter::once(Bar(s.to_owned())).collect()
}
