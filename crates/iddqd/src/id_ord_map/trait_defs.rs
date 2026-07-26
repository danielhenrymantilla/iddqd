//! Trait definitions for `IdOrdMap`.

use crate::{Feed, ForLtComparable};
use alloc::{boxed::Box, rc::Rc, sync::Arc};

/// An element stored in an [`IdOrdMap`].
///
/// This trait is used to define the key type for the map.
///
/// # Examples
///
/// ```
/// use iddqd::{IdOrdItem, IdOrdMap, id_upcast, Feed, ForLt};
///
/// // Define a struct with a key.
/// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// struct MyItem {
///     id: String,
///     value: u32,
/// }
///
/// // Implement IdOrdItem for the struct.
/// impl IdOrdItem for MyItem {
///     // Keys can borrow from the item.
///     type Key = ForLt![<'a> = &'a str];
///
///     fn key(&self) -> Feed<'_, Self::Key> {
///         &self.id
///     }
///
///     id_upcast!();
/// }
///
/// // Create an IdOrdMap and insert items.
/// let mut map = IdOrdMap::new();
/// map.insert_unique(MyItem { id: "foo".to_string(), value: 42 }).unwrap();
/// map.insert_unique(MyItem { id: "bar".to_string(), value: 20 }).unwrap();
/// ```
///
/// [`IdOrdMap`]: crate::IdOrdMap
pub trait IdOrdItem {
    /// The key type.
    type Key: ForLtComparable;

    /// Retrieves the key.
    fn key(&self) -> Feed<'_, Self::Key>;

    /// Upcasts the key to a shorter lifetime, in effect asserting that the
    /// lifetime `'a` on [`IdOrdItem::Key`] is covariant.
    ///
    /// Typically implemented via the [`id_upcast`] macro.
    ///
    /// [`id_upcast`]: crate::id_upcast
    fn upcast_key<'short, 'long: 'short>(
        long: Feed<'long, Self::Key>,
    ) -> Feed<'short, Self::Key>;
}

macro_rules! impl_for_ref {
    ($type:ty) => {
        impl<'b, T: 'b + ?Sized + IdOrdItem> IdOrdItem for $type {
            type Key = T::Key;

            fn key(&self) -> Feed<'_, Self::Key> {
                (**self).key()
            }

            fn upcast_key<'short, 'long: 'short>(
                long: Feed<'long, Self::Key>,
            ) -> Feed<'short, Self::Key> {
                T::upcast_key(long)
            }
        }
    };
}

impl_for_ref!(&'b T);
impl_for_ref!(&'b mut T);

macro_rules! impl_for_box {
    ($type:ty) => {
        impl<T: ?Sized + IdOrdItem> IdOrdItem for $type {
            type Key = T::Key;

            fn key(&self) -> Feed<'_, Self::Key> {
                (**self).key()
            }

            fn upcast_key<'short, 'long: 'short>(
                long: Feed<'long, Self::Key>,
            ) -> Feed<'short, Self::Key> {
                T::upcast_key(long)
            }
        }
    };
}

impl_for_box!(Box<T>);
impl_for_box!(Rc<T>);
impl_for_box!(Arc<T>);
