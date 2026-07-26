//! Trait definitions for `BiHashMap`.

use crate::{Equivalent, Feed, ForLt};
use alloc::{boxed::Box, rc::Rc, sync::Arc};
use core::hash::Hash;

/// An item in a [`BiHashMap`].
///
/// This trait is used to define the keys.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "default-hasher")] {
/// use iddqd::{BiHashItem, BiHashMap, bi_upcast, Feed, ForLt};
///
/// // Define a struct with two keys.
/// #[derive(Debug, PartialEq, Eq, Hash)]
/// struct MyPair {
///     id: u32,
///     name: String,
/// }
///
/// // Implement BiHashItem for the struct.
/// impl BiHashItem for MyPair {
///     type K1 = ForLt![<'a> = u32];
///     type K2 = ForLt![<'a> = &'a str];
///
///     fn key1(&self) -> Feed<'_, Self::K1> {
///         self.id
///     }
///
///     fn key2(&self) -> Feed<'_, Self::K2> {
///         &self.name
///     }
///
///     bi_upcast!();
/// }
///
/// // Create a BiHashMap and insert items.
/// let mut map = BiHashMap::new();
/// map.insert_unique(MyPair { id: 1, name: "Alice".to_string() }).unwrap();
/// map.insert_unique(MyPair { id: 2, name: "Bob".to_string() }).unwrap();
/// # }
/// ```
///
/// [`BiHashMap`]: crate::BiHashMap
pub trait BiHashItem {
    /// The first key type.
    type K1: for<'a> ForLt<
        Of<'a>: Eq + Hash + for<'b> Equivalent<Feed<'b, Self::K1>>,
    >;

    /// The second key type.
    type K2: for<'a> ForLt<
        Of<'a>: Eq + Hash + for<'b> Equivalent<Feed<'b, Self::K2>>,
    >;

    /// Retrieves the first key.
    fn key1(&self) -> Feed<'_, Self::K1>;

    /// Retrieves the second key.
    fn key2(&self) -> Feed<'_, Self::K2>;

    /// Upcasts the first key to a shorter lifetime, in effect asserting that
    /// the lifetime `'a` on [`BiHashItem::K1`] is covariant.
    ///
    /// Typically implemented via the [`bi_upcast`] macro.
    ///
    /// [`bi_upcast`]: crate::bi_upcast
    fn upcast_key1<'short, 'long: 'short>(
        long: Feed<'long, Self::K1>,
    ) -> Feed<'short, Self::K1>;

    /// Upcasts the second key to a shorter lifetime, in effect asserting that
    /// the lifetime `'a` on [`BiHashItem::K2`] is covariant.
    ///
    /// Typically implemented via the [`bi_upcast`] macro.
    ///
    /// [`bi_upcast`]: crate::bi_upcast
    fn upcast_key2<'short, 'long: 'short>(
        long: Feed<'long, Self::K2>,
    ) -> Feed<'short, Self::K2>;
}

macro_rules! impl_for_ref {
    ($type:ty) => {
        impl<'b, T: 'b + ?Sized + BiHashItem> BiHashItem for $type {
            type K1 = T::K1;
            type K2 = T::K2;

            fn key1(&self) -> Feed<'_, Self::K1> {
                (**self).key1()
            }

            fn key2(&self) -> Feed<'_, Self::K2> {
                (**self).key2()
            }

            fn upcast_key1<'short, 'long: 'short>(
                long: Feed<'long, Self::K1>,
            ) -> Feed<'short, Self::K1> {
                T::upcast_key1(long)
            }

            fn upcast_key2<'short, 'long: 'short>(
                long: Feed<'long, Self::K2>,
            ) -> Feed<'short, Self::K2> {
                T::upcast_key2(long)
            }
        }
    };
}

impl_for_ref!(&'b T);
impl_for_ref!(&'b mut T);

macro_rules! impl_for_box {
    ($type:ty) => {
        impl<T: ?Sized + BiHashItem> BiHashItem for $type {
            type K1 = T::K1;
            type K2 = T::K2;

            fn key1(&self) -> Feed<'_, Self::K1> {
                (**self).key1()
            }

            fn key2(&self) -> Feed<'_, Self::K2> {
                (**self).key2()
            }

            fn upcast_key1<'short, 'long: 'short>(
                long: Feed<'long, Self::K1>,
            ) -> Feed<'short, Self::K1> {
                T::upcast_key1(long)
            }

            fn upcast_key2<'short, 'long: 'short>(
                long: Feed<'long, Self::K2>,
            ) -> Feed<'short, Self::K2> {
                T::upcast_key2(long)
            }
        }
    };
}

impl_for_box!(Box<T>);
impl_for_box!(Rc<T>);
impl_for_box!(Arc<T>);
