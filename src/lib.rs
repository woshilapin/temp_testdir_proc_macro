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
    new_test_fn.constness = function.constness.clone();
    new_test_fn.asyncness = function.asyncness.clone();
    new_test_fn.unsafety = function.unsafety.clone();
    new_test_fn.abi = function.abi.clone();
    let token_stream = quote! { #new_test_fn };
    token_stream.into()
}

// TODO: Add documentation
#[proc_macro_attribute]
pub fn with_tempdir(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let configuration: Configuration<_> = parse_macro_input!(attributes as AttributeArgs)
        .into_iter()
        .collect();
    let mut test_fn = parse_macro_input!(input as ItemFn);
    let token_stream = wrap_function(&mut test_fn, &configuration);
    token_stream.into()
}
