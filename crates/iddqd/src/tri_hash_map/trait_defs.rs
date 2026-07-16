//! Trait definitions for `TriHashMap`.

use crate::{Feed, ForLtEquivalent};
use alloc::{boxed::Box, rc::Rc, sync::Arc};

/// An item in a [`TriHashMap`].
///
/// This trait is used to define the keys.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "default-hasher")] {
/// use iddqd::{TriHashItem, TriHashMap, tri_upcast};
///
/// // Define a struct with three keys.
/// #[derive(Debug, PartialEq, Eq, Hash)]
/// struct Person {
///     id: u32,
///     name: String,
///     email: String,
/// }
///
/// // Implement TriHashItem for the struct.
/// impl TriHashItem for Person {
///     type K1<'a> = u32;
///     type K2<'a> = &'a str;
///     type K3<'a> = &'a str;
///
///     fn key1(&self) -> Self::K1<'_> {
///         self.id
///     }
///
///     fn key2(&self) -> Self::K2<'_> {
///         &self.name
///     }
///
///     fn key3(&self) -> Self::K3<'_> {
///         &self.email
///     }
///
///     tri_upcast!();
/// }
///
/// // Create a TriHashMap and insert items.
/// let mut map = TriHashMap::new();
/// map.insert_unique(Person {
///     id: 1,
///     name: "Alice".to_string(),
///     email: "alice@example.com".to_string(),
/// })
/// .unwrap();
/// map.insert_unique(Person {
///     id: 2,
///     name: "Bob".to_string(),
///     email: "bob@example.com".to_string(),
/// })
/// .unwrap();
/// # }
/// ```
///
/// [`TriHashMap`]: crate::TriHashMap
pub trait TriHashItem {
    /// The first key type.
    type K1: ForLtEquivalent;

    /// The second key type.
    type K2: ForLtEquivalent;

    /// The third key type.
    type K3: ForLtEquivalent;

    /// Retrieves the first key.
    fn key1(&self) -> Feed<'_, Self::K1>;

    /// Retrieves the second key.
    fn key2(&self) -> Feed<'_, Self::K2>;

    /// Retrieves the third key.
    fn key3(&self) -> Feed<'_, Self::K3>;

    /// Upcasts the first key to a shorter lifetime, in effect asserting that
    /// the lifetime `'a` on [`TriHashItem::K1`] is covariant.
    ///
    /// Typically implemented via the [`tri_upcast`] macro.
    ///
    /// [`tri_upcast`]: crate::tri_upcast
    fn upcast_key1<'short, 'long: 'short>(
        long: Feed<'long, Self::K1>,
    ) -> Feed<'short, Self::K1>;

    /// Upcasts the second key to a shorter lifetime, in effect asserting that
    /// the lifetime `'a` on [`TriHashItem::K2`] is covariant.
    ///
    /// Typically implemented via the [`tri_upcast`] macro.
    ///
    /// [`tri_upcast`]: crate::tri_upcast
    fn upcast_key2<'short, 'long: 'short>(
        long: Feed<'long, Self::K2>,
    ) -> Feed<'short, Self::K2>;

    /// Upcasts the third key to a shorter lifetime, in effect asserting that
    /// the lifetime `'a` on [`TriHashItem::K3`] is covariant.
    ///
    /// Typically implemented via the [`tri_upcast`] macro.
    ///
    /// [`tri_upcast`]: crate::tri_upcast
    fn upcast_key3<'short, 'long: 'short>(
        long: Feed<'long, Self::K3>,
    ) -> Feed<'short, Self::K3>;
}

macro_rules! impl_for_ref {
    ($type:ty) => {
        impl<'b, T: 'b + ?Sized + TriHashItem> TriHashItem for $type {
            type K1 = T::K1;
            type K2 = T::K2;
            type K3 = T::K3;

            fn key1(&self) -> Feed<'_, Self::K1> {
                (**self).key1()
            }

            fn key2(&self) -> Feed<'_, Self::K2> {
                (**self).key2()
            }

            fn key3(&self) -> Feed<'_, Self::K3> {
                (**self).key3()
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

            fn upcast_key3<'short, 'long: 'short>(
                long: Feed<'long, Self::K3>,
            ) -> Feed<'short, Self::K3> {
                T::upcast_key3(long)
            }
        }
    };
}

impl_for_ref!(&'b T);
impl_for_ref!(&'b mut T);

macro_rules! impl_for_box {
    ($type:ty) => {
        impl<T: ?Sized + TriHashItem> TriHashItem for $type {
            type K1 = T::K1;
            type K2 = T::K2;
            type K3 = T::K3;

            fn key1(&self) -> Feed<'_, Self::K1> {
                (**self).key1()
            }

            fn key2(&self) -> Feed<'_, Self::K2> {
                (**self).key2()
            }

            fn key3(&self) -> Feed<'_, Self::K3> {
                (**self).key3()
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

            fn upcast_key3<'short, 'long: 'short>(
                long: Feed<'long, Self::K3>,
            ) -> Feed<'short, Self::K3> {
                T::upcast_key3(long)
            }
        }
    };
}

impl_for_box!(Box<T>);
impl_for_box!(Rc<T>);
impl_for_box!(Arc<T>);
