//! Macro internals for `upgrayedd`.
//!
//! Don't use this crate directly. You want the `upgrayedd` crate instead.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Comma, FnArg, Ident,
    LitByteStr, LitStr,
};

// Converts a FnArg sequence into a like sequence, but only with identifiers.
// For example, `foo: u8, bar: u64` becomes just `foo, bar`.
//
// Adapted and simplified from: https://stackoverflow.com/a/71482073
fn transform_params(params: Punctuated<FnArg, Comma>) -> Punctuated<Ident, Comma> {
    let idents = params.iter().filter_map(|param| {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                return Some(pat_ident.ident);
            }
        }
        None
    });

    let mut punctuated: Punctuated<syn::Ident, Comma> = Punctuated::new();
    idents.for_each(|ident| punctuated.push(ident));

    punctuated
}

/// Use `upgrayedd::upgrayedd` instead.
#[proc_macro_attribute]
pub fn upgrayedd(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as syn::ItemFn);

    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = func;

    // This is purely for misuse-resistance reasons: these wrapper functions
    // assume that the underlying function pointer has been initialized,
    // which may not be true if they're called directly from Rust rather
    // than indirectly due to a shared library call. We can't completely
    // stop the user from shooting themselves in the foot, but we can
    // at least indicate to them that they're doing something wrong by
    // trying to expose the wrapper somewhere else.
    if !matches!(vis, syn::Visibility::Inherited) {
        return syn::Error::new(vis.span(), "upgrayedd-ed functions much be private")
            .to_compile_error()
            .into();
    }

    let stmts = &block.stmts;

    let syn::Signature {
        constness: _,
        asyncness: _,
        unsafety: _,
        abi: _,
        fn_token: _,
        ident,
        generics: _,
        paren_token: _,
        inputs,
        variadic: _,
        output,
    } = sig.clone();

    let inner_var =
        parse_macro_input!(attr as Option<Ident>).unwrap_or(Ident::new("upgrayedd", ident.span()));

    // A string literal for the function being wrapped, e.g. `"read"`
    let real_c_name_lit = LitStr::new(&ident.to_string(), ident.span());

    // The same but a null-terminated bytes literal, e.g. `b"read\0"`
    let real_c_name_bytes = {
        let real_c_name_lit_bytes = ident.to_string().into_bytes();
        LitByteStr::new(&real_c_name_lit_bytes, ident.span())
    };

    // The same but a null-terminated bytes literal, e.g. `b"read\0"`
    let real_c_name_bytes_nulled = {
        let mut real_c_name_lit_bytes = ident.to_string().into_bytes();
        real_c_name_lit_bytes.push(0);
        LitByteStr::new(&real_c_name_lit_bytes, ident.span())
    };

    // The Rust-side "inner" wrapper ident for the target function, e.g. `__upgrayedd_inner_wrapper_read`
    let inner_wrapper = Ident::new(&format!("__upgrayedd_inner_wrapper_{ident}"), ident.span());

    // The Rust-side target ident for the target function, e.g. `__upgrayedd_target_read`
    //
    // This is really the name of the global that holds the function pointer to the target.
    let target = Ident::new(&format!("__upgrayedd_target_{ident}"), ident.span());

    // The Rust-side target constructor ident for the target function, e.g. `_upgrayedd_target_ctor_read`
    //
    // This is nested in a few places, but at the top-level is a static function pointer to
    // a function that, when called (during early program initialization), sets the value
    // of `__upgrayedd_target_read` to the actual function pointer being targeted for wrapping.
    // let target_ctor = Ident::new(&format!("__upgrayedd_target_ctor_{ident}"), target.span());

    let args = transform_params(inputs.clone());

    let gen = quote! {
        static mut #target: Option<unsafe extern "C" fn(#inputs) #output> = None;

        // #[used]
        // #[allow(non_upper_case_globals)]
        // #[doc(hidden)]
        // #[link_section = ".init_array"]
        // static #target_ctor: unsafe extern "C" fn() -> usize = {
        //     #[allow(non_snake_case)]
        //     #[link_section = ".text.startup"]
        //     unsafe extern "C" fn #target_ctor() -> usize {
        //         // NOTE: transmuting from a nullable function pointer to `Option<fn _>` is sound, per:
        //         // https://rust-lang.github.io/unsafe-code-guidelines/layout/function-pointers.html#representation
        //         #target = std::mem::transmute(::libc::dlsym(
        //             ::libc::RTLD_NEXT,
        //             std::mem::transmute(#real_c_name_bytes_nulled.as_ptr()),
        //         ));

        //         0
        //     }
        //     #target_ctor
        // };

        #[no_mangle]
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[export_name = #real_c_name_lit]
        pub unsafe extern "C" fn #inner_wrapper(#inputs) #output {
            if #target.is_none() {
                #target = std::mem::transmute(::libc::dlsym(
                    ::libc::RTLD_NEXT,
                    std::mem::transmute(#real_c_name_bytes_nulled.as_ptr()),
                ));
            }

            // This should only happen if, somehow, our wrapper gets called
            // with no underlying target.
            if #target.is_none() {
                // NOTE: We can't reliably panic here, since we might be hooking
                // something like malloc (in which case we'd regress infinitely,
                // since the panic handler calls malloc).
                // Instead, we dump a basic message to stderr and abort directly.
                let msg = b"barf: upgrayedd tried to hook something that broke rust's runtime: ";
                ::libc::write(::libc::STDERR_FILENO, msg.as_ptr() as *const ::libc::c_void, msg.len());
                ::libc::write(::libc::STDERR_FILENO, #real_c_name_bytes.as_ptr() as *const ::libc::c_void, #real_c_name_bytes.len());
                ::libc::write(::libc::STDERR_FILENO, b"\n".as_ptr() as *const ::libc::c_void, 1);

                std::process::abort();
            }

            #ident(#args)
        }

        #[allow(non_snake_case)]
        #(#attrs)* #vis #sig {
            #[allow(unused_variables)]
            let #inner_var = unsafe { #target.unwrap_unchecked() };

            #(#stmts)*
        }
    };

    gen.into()
}
