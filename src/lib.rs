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

fn wrap_function(function: &mut ItemFn, configuration: &Configuration<String>) -> TokenStream {
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
