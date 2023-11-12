# upgrayedd

[*"Two Ds for a double dose of debugging."*](https://www.youtube.com/watch?v=Gf9i23KPGKk)

> [!WARNING]
> You do not need this library. Do not use this library, especially in production
> systems. It is **not safe**, *even if* you only use safe Rust with it.
> It **cannot** be made safe.

`upgrayedd` is a convenience proc macro for building function interposition
tools in Rust. When used in a shared library build (which is the only
way you should be using it), it makes `LD_PRELOAD`-style interposition
look like (relatively) idiomatic Rust.

## Usage

Interposing on an OpenSSL API:

```rust,no_run
use upgrayedd::upgrayedd;

#[upgrayedd]
fn X509_VERIFY_PARAM_set_auth_level(param: *mut std::ffi::c_void, level: std::ffi::c_int) {
    eprintln!("before!");
    unsafe { upgrayedd(param, level) };
    eprintln!("after!");
}
```

Rewriting parameters:

```rust
use upgrayedd::upgrayedd;

#[upgrayedd]
fn frobulate(size: libc::size_t) {
    unsafe { upgrayedd(size + 42) };
}
```

Stubbing functions out:

```rust
use upgrayedd::upgrayedd;

#[upgrayedd]
fn verify_cert(cert: *mut std::ffi::c_void) -> libc::c_int {
    // nothing to see here
    1
}
```

See the [docs] for more usage examples.

[docs]: https://docs.rs/upgrayedd

## Limitations

* Fundamentally unsafe. Again: please do not use this for anything serious.
* Hooking "basic" routines (like `malloc` and `free`) may or may not work,
  depending on what you put in your hook. In particular, if you accidentally
  recurse while holding a lock (which is easy to do when hooking `malloc`
  and calling `println!`), you'll probably end up deadlocking.
* Bad things will definitely happen if you try to pass Rust types into
  wrapped C functions, which `upgrayedd` will happily let you do.
* Bad things will definitely happen if you supply the wrong function
  signature, which `upgrayedd` will happily let you do.
* Only C functions can currently be wrapped; C++ would probably work with small
  tweaks.
* Currently Linux only. Other OSes would probably work with small tweaks.
