use ::core::cmp::Ordering;

extern crate self as iddqd;

trait Equivalent<K: ?Sized> {
    /// Compare self to `key` and return `true` if they are equal.
    fn equivalent(&self, key: &K) -> bool;
}

impl Equivalent<&str> for &str {
    fn equivalent(&self, key: &&str) -> bool {
        self == key
    }
}

trait Comparable<K: ?Sized>: Equivalent<K> {
    /// Compare self to `key` and return their ordering.
    fn compare(&self, key: &K) -> Ordering;
}

impl Comparable<&str> for &str {
    fn compare(&self, key: &&str) -> Ordering {
        <str>::cmp(self, key)
    }
}

#[derive(::iddqd_derive::Equivalent, ::iddqd_derive::Comparable)]
struct _S<'a> {
    x: &'a str,
}
