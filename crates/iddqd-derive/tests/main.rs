use ::core::cmp::Ordering;

extern crate self as iddqd;

trait Equivalent<K: ?Sized> {
    /// Compare self to `key` and return `true` if they are equal.
    fn equivalent(&self, key: &K) -> bool;
}

impl Equivalent<str> for str {
    fn equivalent(&self, key: &str) -> bool {
        <str>::eq(self, key)
    }
}

impl Equivalent<&str> for &str {
    fn equivalent(&self, key: &&str) -> bool {
        <str>::equivalent(self, key)
    }
}

trait Comparable<K: ?Sized>: Equivalent<K> {
    /// Compare self to `key` and return their ordering.
    fn compare(&self, key: &K) -> Ordering;
}

impl Comparable<str> for str {
    fn compare(&self, key: &str) -> Ordering {
        <str>::cmp(self, key)
    }
}

impl Comparable<&str> for &str {
    fn compare(&self, key: &&str) -> Ordering {
        <str>::compare(self, key)
    }
}

#[derive(::iddqd_derive::Equivalent, ::iddqd_derive::Comparable)]
struct _S<'a> {
    x: &'a str,
}

#[derive(::iddqd_derive::Equivalent, ::iddqd_derive::Comparable)]
enum Foo<'a, T: ?Sized> {
    S(&'a T),
    None,
}

#[test]
fn test_enum() {
    let test_cases = &[
        (Foo::S("abc"), Ordering::Greater, Foo::S("a")),
        (Foo::S("abc"), Ordering::Equal, Foo::S("abc")),
        (Foo::S("abc"), Ordering::Less, Foo::S("c")),
        (Foo::None, Ordering::Greater, Foo::S("abc")),
        (Foo::S("abc"), Ordering::Less, Foo::None),
    ];
    for (l, expected, r) in test_cases {
        assert_eq!(l.compare(r), *expected);
        assert_eq!(l.equivalent(r), *expected == Ordering::Equal);
    }
}
