//! # `upgrayedd`
//!
//! [*"Two Ds for a double dose of debugging."*](https://www.youtube.com/watch?v=Gf9i23KPGKk)
//!
//! WARNING: You do not need this library. Do not use this library, especially
//! in production systems. It is **not safe**, *even if* you only use safe
//! Rust with it. It **cannot** be made safe.
//!
//! `upgrayedd` is a convenience proc macro for building function interposition
//! tools in Rust. When used in a shared library build (which is the only
//! way you should be using it), it makes `LD_PRELOAD`-style interposition
//! look like (relatively) idiomatic Rust.
//!
//! See the [macro docs](`macro@upgrayedd`) for more details.

/// The `#[upgrayedd]` attribute macro.
///
/// This macro can be used to simultaneously define a *target* (C) function
/// to wrap *and* the handler that will wrap it.
///
/// ## Basic usage
///
/// A simple example:
///
/// ```no_run
/// use upgrayedd::upgrayedd;
///
/// #[upgrayedd]
/// fn X509_VERIFY_PARAM_set_auth_level(param: *mut std::ffi::c_void, level: std::ffi::c_int) {
///     eprintln!("before!");
///     unsafe { upgrayedd(param, level) };
///     eprintln!("after!");
/// }
/// ```
///
/// In this case, `upgrayedd` is targeting OpenSSL's [`X509_VERIFY_PARAM_set_auth_level`] API,
/// and the wrapper does some printing before and after calling the underlying wrapped
/// function, which is exposed through the injected `upgrayedd` local variable.
///
/// ## Advanced usage
///
/// There isn't that much else to do. However, a few extra things to know:
///
/// You can change the local binding from `upgrayedd` to whatever you please
/// by passing a different binding in via `#[upgrayedd(my_binding)]`. For example:
///
/// ```no_run
/// use upgrayedd::upgrayedd;
///
/// #[upgrayedd(real_frobulate)]
/// fn frobulate(size: std::ffi::c_int) -> *mut std::ffi::c_void {
///     eprintln!("frobulating {size} bytes");
///
///     unsafe { real_frobulate(size) }
/// }
/// ```
///
/// You can, of course, modify parameters before passing them into the wrapped function:
///
/// ```no_run
/// use upgrayedd::upgrayedd;
///
/// #[upgrayedd(real_frobulate)]
/// fn frobulate(size: std::ffi::c_int) -> *mut std::ffi::c_void {
///     unsafe { real_frobulate(size + 42) }
/// }
/// ```
///
/// Or choose not to call the wrapped function at all:
///
/// ```no_run
/// use upgrayedd::upgrayedd;
///
/// #[upgrayedd]
/// fn check_password(passwd: *const std::ffi::c_char) -> std::ffi::c_int {
///     1
/// }
/// ```
///
/// ## Safety
///
/// Despite exposing a safe Rust wrapper function, `upgrayedd`'s behavior is **fundamentally unsafe
/// and unsound**. This is by design: dynamic function interposition *intentionally* violates
/// program invariants, especially if used to modify parameters in between function calls. It
/// also violates Rust's runtime invariants.
///
/// [`X509_VERIFY_PARAM_set_auth_level`]: https://www.openssl.org/docs/man1.1.1/man3/X509_VERIFY_PARAM_set_auth_level.html
pub use upgrayedd_macros::upgrayedd;

#[doc(hidden)]
pub use ::libc;
