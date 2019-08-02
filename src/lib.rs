#![doc(html_playground_url = "https://play.rust-lang.org/")]
//! This crate provide the [`with_tempdir`] macro.
//!
//! ```
//! use with_tempdir_procmacro::with_tempdir;
//!
//! #[with_tempdir(path = "/tmp/foo")]
//! #[test]
//! fn my_test(path: &Path) {
//!   let file_path = path.join("some_file.txt");
//!   let mut file = File::create(&file_path).expect("Failed to create the file");
//! }
//! ```
//!
//! Read the documentation of [`with_tempdir`] macro to know more.
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::{iter::FromIterator, path::Path};
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

#[derive(Debug, Default)]
struct Configuration<P>
where
    P: AsRef<Path>,
{
    path: Option<P>,
}

impl FromIterator<NestedMeta> for Configuration<String> {
    fn from_iter<T: IntoIterator<Item = NestedMeta>>(iter: T) -> Self {
        let mut configuration = Configuration::default();
        for attribute in iter {
            match attribute {
                NestedMeta::Meta(Meta::NameValue(ref name_value)) if name_value.ident == "path" => {
                    match &name_value.lit {
                        Lit::Str(value) => configuration.path = Some(value.value()),
                        Lit::ByteStr(value) => {
                            configuration.path = match String::from_utf8(value.value()) {
                                Ok(v) => Some(v),
                                _ => continue,
                            }
                        }
                        _ => continue,
                    };
                }
                _ => continue,
            }
        }
        configuration
    }
}

fn build_tempdir<P>(path: &Option<P>) -> proc_macro2::TokenStream
where
    P: AsRef<Path> + ToTokens,
{
    if let Some(path) = path {
        quote! {
            tempfile::Builder::new().tempdir_in(#path)
        }
    } else {
        quote! {
            tempfile::Builder::new().tempdir()
        }
    }
}

fn check_type(ty: &syn::Type) -> Option<syn::Error> {
    fn check_type_is_path(ty: &syn::Type) -> Option<syn::Error> {
        let incorrect_type = Some(syn::Error::new_spanned(
            ty,
            "First argument of the function must be of type `&std::path::Path`",
        ));
        match ty {
            syn::Type::Path(syn::TypePath {
                path: syn::Path { segments, .. },
                ..
            }) => {
                if let Some(path_segment) = segments.last() {
                    if path_segment.value().ident == "Path" {
                        None
                    } else {
                        incorrect_type
                    }
                } else {
                    incorrect_type
                }
            }
            _ => incorrect_type,
        }
    }
    match ty {
        syn::Type::Reference(syn::TypeReference {
            mutability: None,
            elem: boxed_type,
            ..
        }) => check_type_is_path(boxed_type),
        _ => Some(syn::Error::new_spanned(
            ty,
            "First argument of the function must be a reference on type `std::path::Path`",
        )),
    }
}

fn check_function_declaration(function: &ItemFn) -> Option<syn::Error> {
    if function.decl.inputs.len() == 1 {
        if let Some(pair) = function.decl.inputs.first() {
            let fn_arg = pair.value();
            return match fn_arg {
                syn::FnArg::Captured(syn::ArgCaptured { ty, .. }) => check_type(ty),
                _ => Some(syn::Error::new_spanned(
                    fn_arg,
                    "First argument of the function must be of type `&std::path::Path`",
                )),
            };
        }
    }
    Some(syn::Error::new_spanned(
        &function.ident,
        "Function must take one argument of type `&std::path::Path`",
    ))
}

fn wrap_function(function: &mut ItemFn, configuration: &Configuration<String>) -> TokenStream {
    if let Some(error) = check_function_declaration(&function) {
        return error.to_compile_error().into();
    }
    let tempdir = build_tempdir(&configuration.path);
    let function_ident = function.ident.clone();
    let function_attributes = function.attrs.clone();
    function.attrs = Vec::new();
    let wrapped = quote! {
        fn #function_ident() {
            #function
            let temp_dir = #tempdir.expect("Failed to create a temporary folder");
            #function_ident(&temp_dir.path());
        }
    };
    let wrapped: TokenStream = wrapped.into();
    let mut new_test_fn = parse_macro_input!(wrapped as ItemFn);
    new_test_fn.attrs = function_attributes;
    new_test_fn.vis = function.vis.clone();
    new_test_fn.constness = function.constness;
    new_test_fn.asyncness = function.asyncness;
    new_test_fn.unsafety = function.unsafety;
    new_test_fn.abi = function.abi.clone();
    let token_stream = quote! { #new_test_fn };
    token_stream.into()
}

/// The macro `with_tempdir` is providing you an easy way
/// to inject temporary folders in your test functions.
///
/// The macro will inject a reference on a [Path] in your function.
/// You can then write files to this newly created folder.
/// And don't worry, it's going to be cleaned up once the test is done.
///
/// The macro is using the crate [tempfile](https://crates.io/crates/tempfile)
/// behind the scene which will clean up the folder on [Drop]. If you want to
/// know more about the guarantees that [tempfile] is giving, read more on
/// documentation of the [crate](https://docs.rs/tempfile/3.1.0/tempfile/).
///
/// # How to use it
/// Below is the most simple example to use the macro.
/// ```
/// # use with_tempdir_procmacro::with_tempdir;
/// #[with_tempdir]
/// #[test]
/// fn my_test(path: &Path) {
///   let file_path = path.join("some_file.txt");
///   let mut file = File::create(&file_path).expect("Failed to create the file");
/// }
/// ```
///
/// If you want to control where the temporary directory is going to be created,
/// you can use the `path` argument of the macro
/// ```
/// # use with_tempdir_procmacro::with_tempdir;
/// #[with_tempdir(path = "/tmp/foo")]
/// #[test]
/// #[should_panic]
/// fn my_test(path: &Path) {
///   let file_path = path.join("some_file.txt");
///   let mut file = File::create(&file_path).expect("Failed to create the file");
///   assert!(false);
/// }
/// ```
///
/// And that's about it!
///
/// # Warning
/// Due to how the `#[test]` is a little specific, the order of the macros above
/// the function are important: `#[with_tempdir]` must be before `#[test]`.
///
/// For example, the following code will not compile
/// ```ignore,compile_fail
/// # use with_tempdir_procmacro::with_tempdir;
/// #[test]
/// #[with_tempdir]
/// fn my_test(path: &Path) {
///   let file_path = path.join("some_file.txt");
///   let mut file = File::create(&file_path).expect("Failed to create the file");
///   assert!(false);
/// }
/// ```
#[proc_macro_attribute]
pub fn with_tempdir(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let configuration: Configuration<_> = parse_macro_input!(attributes as AttributeArgs)
        .into_iter()
        .collect();
    let mut test_fn = parse_macro_input!(input as ItemFn);
    wrap_function(&mut test_fn, &configuration)
}
