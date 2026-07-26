//! Unused, this whole package is there just to work around a trybuild limitation
//! wherein it is unable to pass `--features` to its own invocation, and we want the overall
//! _test_ suite of the `iddqd` _package_ to be `serde`-tweakable (no unconditional enabling of it).
//!
//! Hence this separate _package_ to isolate an unconditional enabling of it.
