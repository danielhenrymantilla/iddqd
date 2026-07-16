#![expect(clippy::needless_lifetimes)] // What a moronic lint.

use ::core::cmp::Ordering;
pub use ::iddqd_derive::{Comparable, Equivalent};

/// Key equivalence trait.
///
/// This trait allows hash table lookup to be customized. It has one blanket
/// implementation that uses the regular solution with `Borrow` and `Eq`, just
/// like `HashMap` does, so that you can pass `&str` to lookup into a map with
/// `String` keys and so on.
///
/// # Contract
///
/// The implementor **must** hash like `K`, if it is hashable.
pub trait Equivalent<K: ?Sized> {
    /// Compare self to `key` and return `true` if they are equal.
    fn equivalent(&self, key: &K) -> bool;
}

// impl<Q: ?Sized, K: ?Sized> Equivalent<K> for Q
// where
//     Q: Eq,
//     K: Borrow<Q>,
// {
//     #[inline]
//     fn equivalent(&self, key: &K) -> bool {
//         PartialEq::eq(self, key.borrow())
//     }
// }

/// Key ordering trait.
///
/// This trait allows ordered map lookup to be customized. It has one blanket
/// implementation that uses the regular solution with `Borrow` and `Ord`, just
/// like `BTreeMap` does, so that you can pass `&str` to lookup into a map with
/// `String` keys and so on.
pub trait Comparable<K: ?Sized>: Equivalent<K> {
    /// Compare self to `key` and return their ordering.
    fn compare(&self, key: &K) -> Ordering;
}

// impl<Q: ?Sized, K: ?Sized> Comparable<K> for Q
// where
//     Q: Ord,
//     K: Borrow<Q>,
// {
//     #[inline]
//     fn compare(&self, key: &K) -> Ordering {
//         Ord::cmp(self, key.borrow())
//     }
// }

simple_impl! {
    u8, u16, u32, usize, u64, u128,
    i8, i16, i32, isize, i64, i128,
    bool, char,
    str,
    (),
}

#[cfg(feature = "std")]
simple_impl! {
    ::std::path::Path,
    ::std::string::String,
}

/// TODO
#[macro_export]
macro_rules! __simple_impl {(
    $(
        $(@for[$($generics:tt)*])? $T:ty
    ),* $(,)?
) => ($(
    impl $(<$($generics)*>)?
        $crate::Equivalent<Self>
    for
        $T
    where
        Self : ::core::cmp::Eq,
    {
        #[inline]
        fn equivalent(&self, other: &Self) -> ::core::primitive::bool {
            Self::eq(self, other)
        }
    }

    impl<'r, $($($generics)*)?>
        $crate::Equivalent<&'r Self>
    for
        $T
    where
        Self : ::core::cmp::Eq,
    {
        #[inline]
        fn equivalent(&self, other: &&'r Self) -> ::core::primitive::bool {
            Self::eq(self, *other)
        }
    }

    impl $(<$($generics)*>)?
        $crate::Comparable<Self>
    for
        $T
    where
        Self : ::core::cmp::Ord,
    {
        #[inline]
        fn compare(&self, other: &Self) -> ::core::cmp::Ordering {
            Self::cmp(self, other)
        }
    }

    impl<'r, $($($generics)*)?>
        $crate::Comparable<&'r Self>
    for
        $T
    where
        Self : ::core::cmp::Ord,
    {
        #[inline]
        fn compare(&self, other: &&'r Self) -> ::core::cmp::Ordering {
            Self::cmp(self, *other)
        }
    }
)*)}
pub use __simple_impl as simple_impl;

impl<'a, 'b, T: ?Sized, U: ?Sized> Equivalent<&'b U> for &'a T
where
    T: Equivalent<U>,
{
    fn equivalent(&self, key: &&'b U) -> bool {
        T::equivalent(self, key)
    }
}

impl<'a, 'b, T: ?Sized, U: ?Sized> Comparable<&'b U> for &'a T
where
    T: Comparable<U>,
{
    fn compare(&self, key: &&'b U) -> Ordering {
        T::compare(self, key)
    }
}

impl<T, U> Equivalent<[U]> for [T]
where
    T: Equivalent<U>,
{
    fn equivalent(&self, key: &[U]) -> bool {
        self.len() == key.len()
            && ::core::iter::zip(self, key).all(|(a, b)| T::equivalent(a, b))
    }
}

impl<T, U> Equivalent<Option<U>> for Option<T>
where
    T: Equivalent<U>,
{
    fn equivalent(&self, key: &Option<U>) -> bool {
        match (self, key) {
            (None, None) => true,
            (Some(a), Some(b)) if T::equivalent(a, b) => true,
            _ => false,
        }
    }
}
