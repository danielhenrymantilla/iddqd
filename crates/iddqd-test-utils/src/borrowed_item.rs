#[cfg(feature = "std")]
use iddqd::IdOrdItem;
use iddqd::{
    BiHashItem, Feed, ForLt, IdHashItem, TriHashItem, bi_upcast, id_upcast,
    tri_upcast,
};
use std::{borrow::Cow, path::Path};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BorrowedItem<'a> {
    pub key1: &'a str,
    pub key2: Cow<'a, [u8]>,
    pub key3: &'a Path,
}

impl<'a> IdHashItem for BorrowedItem<'a> {
    type Key = ForLt![<'k> = &'a str];

    fn key(&self) -> Feed<'_, Self::Key> {
        self.key1
    }

    id_upcast!();
}

#[cfg(feature = "std")]
impl<'a> IdOrdItem for BorrowedItem<'a> {
    type Key = ForLt![<'k> = &'a str];

    fn key(&self) -> Feed<'_, Self::Key> {
        self.key1
    }

    id_upcast!();
}

impl<'a> BiHashItem for BorrowedItem<'a> {
    type K1 = ForLt![<'k> = &'a str];
    type K2 = ForLt![<'k> = &'k [u8]];

    fn key1(&self) -> Feed<'_, Self::K1> {
        self.key1
    }

    fn key2(&self) -> Feed<'_, Self::K2> {
        &*self.key2
    }

    bi_upcast!();
}

impl<'a> TriHashItem for BorrowedItem<'a> {
    type K1 = ForLt![<'k> = &'a str];
    type K2 = ForLt![<'k> = &'k [u8]];
    type K3 = ForLt![<'k> = &'a Path];

    fn key1(&self) -> Feed<'_, Self::K1> {
        self.key1
    }

    fn key2(&self) -> Feed<'_, Self::K2> {
        &*self.key2
    }

    fn key3(&self) -> Feed<'_, Self::K3> {
        self.key3
    }

    tri_upcast!();
}
