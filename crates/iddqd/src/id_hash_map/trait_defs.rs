use crate::{Feed, ForLtEquivalent};
use alloc::{boxed::Box, rc::Rc, sync::Arc};

/// An element stored in an [`IdHashMap`].
///
/// This trait is used to define the key type for the map.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "default-hasher")] {
/// use iddqd::{IdHashItem, IdHashMap, id_upcast};
///
/// // Define a struct with a key.
/// #[derive(Debug, PartialEq, Eq, Hash)]
/// struct MyItem {
///     id: String,
///     value: u32,
/// }
///
/// // Implement IdHashItem for the struct.
/// impl IdHashItem for MyItem {
///     // Keys can borrow from the item.
///     type Key<'a> = &'a str;
///
///     fn key(&self) -> Self::Key<'_> {
///         &self.id
///     }
///
///     id_upcast!();
/// }
///
/// // Create an IdHashMap and insert items.
/// let mut map = IdHashMap::new();
/// map.insert_unique(MyItem { id: "foo".to_string(), value: 42 }).unwrap();
/// map.insert_unique(MyItem { id: "bar".to_string(), value: 20 }).unwrap();
/// # }
/// ```
///
/// [`IdHashMap`]: crate::IdHashMap
pub trait IdHashItem {
    /// The key type.
    type Key: ForLtEquivalent;

    /// Retrieves the key.
    fn key(&self) -> Feed<'_, Self::Key>;

    /// Upcasts the key to a shorter lifetime, in effect asserting that the
    /// lifetime `'a` on [`IdHashItem::Key`] is covariant.
    ///
    /// Typically implemented via the [`id_upcast`] macro.
    fn upcast_key<'short, 'long: 'short>(
        long: Feed<'long, Self::Key>,
    ) -> Feed<'short, Self::Key>;
}

macro_rules! impl_for_ref {
    ($type:ty) => {
        impl<'b, T: 'b + ?Sized + IdHashItem> IdHashItem for $type {
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
        impl<T: ?Sized + IdHashItem> IdHashItem for $type {
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
