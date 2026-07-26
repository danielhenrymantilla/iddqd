use ::core::{cell::Cell, ptr};

use ::higher_kinded_types::prelude::ForLifetimeMaybeUnsized;

pub struct ScopedTls<T: 'static + ForLifetimeMaybeUnsized> {
    inner: &'static ::std::thread::LocalKey<
        Cell<
            Option<
                // Note: what we'd really want here is `Unsafe![<'a, 'b> = &'a T::Of<'b>]`
                ptr::NonNull<T::Of<'static>>,
            >,
        >,
    >,
}

#[rustfmt::skip]
macro_rules! scoped_tls {(
    $(#[doc $($doc:tt)*])*
    $pub:vis static $NAME:ident: $T:ty;
) => (
    $(#[doc $($doc)*])*
    $pub static $NAME: $crate::support::scoped_tls::ScopedTls<ForLt![<'scoped> = $T]> = {
        ::std::thread_local! {
            static __INNER: ::core::cell::Cell<
                ::core::option::Option<
                    ptr::NonNull<
                        <ForLt![<'scoped> = $T] as ::higher_kinded_types::advanced::ForLifetimeMaybeUnsized>::Of<'static>,
                    >
                >,
            > = const { ::core::cell::Cell::new(::core::option::Option::None) };
        }
        // SAFETY: it's indeed `None` atm; otherwise `__INNER` is only set by
        // `ScopedTls::with_value()`.
        unsafe { $crate::support::scoped_tls::ScopedTls::__new(&__INNER) }
    };
)}
pub(crate) use scoped_tls;

impl<T: 'static + ForLifetimeMaybeUnsized> ScopedTls<T> {
    #[doc(hidden)]
    /// Not part of the public API, macro-only API.
    ///
    /// Safety: `inner` must contain either a null/`None` pointer, or something set by
    /// [`ScopedTls::with_value()`].
    pub const unsafe fn __new(
        inner: &'static ::std::thread::LocalKey<
            Cell<
                Option<
                    // Note: what we'd want here is `Unsafe![<'a, 'b> = &'a T::Of<'b>]`
                    ptr::NonNull<T::Of<'static>>,
                >,
            >,
        >,
    ) -> Self {
        Self { inner }
    }

    pub fn with_value<'r, 'a, R>(
        &'static self,
        value: &'r T::Of<'a>,
        scope: impl FnOnce() -> R,
    ) -> R {
        let outer = self.inner.replace(Some(
            // SAFETY: erasing the lifetimes into an inert value.
            // Morally, the `Thing<'x, 'y> -> unsafe<'a, 'b> Thing<'a, 'b>` inert erasure.
            // All the subtlety lies in the `unsafe` conversion done in the other direction.
            unsafe {
                ::core::mem::transmute::<&'r T::Of<'a>, &'r T::Of<'static>>(
                    value,
                )
            }
            .into(),
        ));

        struct ClearOnUnwindGuard<T: 'static + ForLifetimeMaybeUnsized> {
            tls: &'static ScopedTls<T>,
            outer: Option<ptr::NonNull<T::Of<'static>>>,
        }

        let guard = ClearOnUnwindGuard { tls: self, outer };

        let ret = scope();

        // upon unwinding, do:
        impl<T: ForLifetimeMaybeUnsized> Drop for ClearOnUnwindGuard<T> {
            fn drop(&mut self) {
                self.tls.inner.set(self.outer);
            }
        }
        // else:
        ::core::mem::forget(guard);
        self.inner.set(outer);
        ret
    }

    /// Same as [`Self::with_value()`], but with an artificial, dedicated
    /// `<T_Of>` generic parameter, so `rust-analyzer` on-hover info and whatnot show what this type
    /// is about, for when the layers of generics may obscure things.
    ///
    /// Of course, such hover is limited and *doesn't show lifetimes*, heh, but at least it gives
    /// some amount of info.
    //
    // OTOH, this kind of "point-lifetime" bounds confuses Rust, as `T::Of<'b>` now _might_ or
    // might not be concerned by that bound. Rustc hates this ~~one trick~~. With passion. It
    // palliates the resulting "cognitive dissonance" by thenceforth assuming any such `'b` is, in
    // fact, `'a`, gaslighting against any hope for this not to be necessarily true.
    //
    //   - _e.g._, mention of `T::Of<'static>` in the `fn` body would only be acceptable by
    //     Rust if `'static = 'a`, _i.e._, if `'a : 'static`, which obviously does not necessarily
    //     hold.
    //
    // Hence the two-function split: the non-`_dbg fn` "forgets" the "point-lifetime" bound,
    // avoiding the bug.
    #[doc(hidden)]
    #[expect(nonstandard_style)]
    pub fn _dbg_with_value<'r, 'exists_a, T_Of: ?Sized, R>(
        &'static self,
        value: &'r T_Of,
        scope: impl FnOnce() -> R,
    ) -> R
    where
        // exists<'exists_a>
        T::Of<'exists_a>: Is<ItSelf = T_Of>,
    {
        self.with_value(dbg::helper(value), scope)
    }

    pub fn get_with<R>(
        &'static self,
        yield_: impl for<'r, 'a> FnOnce(Option<&'r T::Of<'a>>) -> R,
    ) -> R {
        self.inner.with(|r: &Cell<Option<ptr::NonNull<T::Of<'static>>>>| {
            match r.get() {
                None => yield_(None),
                Some(ptr) => {
                    // SAFETY: if it is `Some`, it means **we're inside** some [`Self::with_value`]
                    // scope.
                    //
                    // And that scope is known to be smaller than either of `'r`, `'a`, since those
                    // are non-`for<>` generic lifetime params enscoping that `fn`.
                    //
                    // Thus, if we call the scope of our current `fn get_with()` call as `'fn`, we
                    // have:
                    // `exists<'a : 'fn, 'r : 'fn> typeof(ptr) = &'r T::Of<'a>`.
                    //
                    // Our caller `FnOnce` is one able to handle `for<'r, 'a> &'r T::Of<'a>` (when
                    // `Some`).
                    //
                    // So no matter our choice of `'r, 'a` when unerasing our conceptual
                    // `unsafe<'0, '1> &'0 T::Of<'1>` into `&'r T::Of<'a>`, i.e., when reïfying a
                    // concrete instance of this `unsafe<>` type via some conjured concrete
                    // lifetimes, this is going to be fine.
                    //
                    //   - (the actual choice here remains, in practice, un(der)specified. Also
                    //     called *unbounded* lifetimes. These are generally **very dangerous** when
                    //     produced by `unsafe`, as we might unify with caller-arbitrarily-picked
                    //     lifetimes of their choosing. But such a general problem/danger does not
                    //     apply here, since the caller has no lifetimes to pick, request, or
                    //     enforce or whatnot.
                    //
                    //     All they have is this universal/general/generic/abstract/you-name-it
                    //     `for<'r, 'a> …` closure signature, which entails that the onus of
                    //     type-checking is on *their* closure, which needs to be able to correctly
                    //     handle *any* choice of lifetimes on our behalf; notably, the
                    //     "true"/correct choice of `'r, 'a`.
                    //
                    //     This is because our signature is like `get_with_1()` in the following
                    //     one, rather than `get_with_2()`, which is where the unbounded lifetimes
                    //     produced by our transmute could, very problematicly, be able to unify
                    //     with *their* choice of `'x, 'y` (eg., them choosing `'x = 'y = 'static`):
                    //
                    //     ```rs
                    //     fn get_with_1(f: impl for<'x, 'y> FnOnce(Option<&'x T::Of<'y>>))
                    //     // vs.
                    //     fn get_with_2<'x, 'y>(f: impl FnOnce(Option<&'x T::Of<'y>>))
                    //     ```
                    let r: &T::Of<'_> = unsafe {
                        // We can't use `.cast()` since `T::Of<'_>` may not be `Sized`.
                        ::core::mem::transmute::<
                            ptr::NonNull<T::Of<'static>>,
                            ptr::NonNull<T::Of<'_>>,
                        >(ptr)
                        .as_ref()
                    };
                    yield_(Some(r))
                }
            }
        })
    }

    /// Same as [`Self::get_with()`], but with an artificial, dedicated
    /// `<T_Of>` generic parameter, so `rust-analyzer` on-hover info and whatnot show what this type
    /// is about, for when the layers of generics may obscure things.
    #[doc(hidden)]
    #[expect(nonstandard_style)]
    pub fn _dbg_get_with<'exists, T_Of: ?Sized, R>(
        &'static self,
        yield_: impl for<'r, 'a> FnOnce(Option<&'r T::Of<'a>>) -> R,
    ) -> R
    where
        // doc-only:
        // exists<'exists>
        T::Of<'exists>: Is<ItSelf = T_Of>,
    {
        self.get_with(yield_)
    }
}

use dbg::Is;
mod dbg {
    #![allow(unused)]

    pub trait Is {
        type ItSelf: ?Sized;
    }

    impl<T: ?Sized> Is for T {
        type ItSelf = Self;
    }

    /// Function-boundary insulated from outstanding `T: Is<Itself = U>` context (which makes Rust
    /// temporarily forget that `T: Is<ItSelf = T>` also holds, resulting in silly mismatches).
    #[inline]
    pub fn helper<T: ?Sized>(r: &<T as Is>::ItSelf) -> &T {
        r
    }
}
