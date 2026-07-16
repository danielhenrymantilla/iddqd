use core::fmt;

/// Wrapper for which [`fmt::Debug`] is implemented as [`fmt::Display`]ing the inner value.
///
/// Notably, for <code>\&[str]</code>s, this removes the quotes.
pub(crate) struct ImplDebugFromDisplay<T: fmt::Display>(pub(crate) T);

impl<T: fmt::Display> fmt::Debug for ImplDebugFromDisplay<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the Display formatter to write the string without quotes.
        fmt::Display::fmt(&self.0, f)
    }
}
