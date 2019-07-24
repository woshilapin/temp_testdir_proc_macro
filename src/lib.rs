extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::{iter::FromIterator, path::Path};
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

#[derive(Debug, Default)]
struct Configuration<P>
where
    P: AsRef<Path>,
{
    ignore: bool,
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
                        Lit::ByteStr(value) => configuration.path = match String::from_utf8(value.value()) {
                            Ok(v) => Some(v),
                            _ => continue,
                        },
                        _ => continue,
                    };
                }
                NestedMeta::Meta(Meta::Word(ref ident)) if ident == "ignore" => {
                    configuration.ignore = true;
                }
                _ => continue,
            }
        }
        configuration
    }
}

// TODO: Add documentation
#[proc_macro_attribute]
pub fn test_with_tempdir(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attributes as AttributeArgs);
    let mut _expect_literal = false;
    let configuration: Configuration<_> = attributes.into_iter().collect();
    let test_macro = if configuration.ignore {
        quote! {
            #[test]
            #[ignore]
        }
    } else {
        quote! {
            #[test]
        }
    };
    let temp_dir = if let Some(path) = configuration.path {
        quote! {
            TempDir::new(#path, true)
        }
    } else {
        quote! {
            TempDir::default()
        }
    };
    let mut test_fn = parse_macro_input!(input as ItemFn);
    // Wrapping function will keep the existing function name
    // Existing function will be renamed with a 'wrapped_' prefix
    let test_function_ident = test_fn.ident.clone();
    let wrapped_function_name = format!("wrapped_{}", test_function_ident);
    let wrapped_function_ident = Ident::new(&wrapped_function_name, Span::call_site());
    test_fn.ident = wrapped_function_ident.clone();
    let wrapped = quote! {
        #test_macro
        fn #test_function_ident() {
            use temp_testdir::TempDir;
            #test_fn
            let temp_dir = #temp_dir;
            #wrapped_function_ident(&temp_dir);
        }
    };
    return wrapped.into();
}
