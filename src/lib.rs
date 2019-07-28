extern crate proc_macro;

// TODO: Add fully qualified names everywhere
use proc_macro::TokenStream;
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
                        Lit::ByteStr(value) => {
                            configuration.path = match String::from_utf8(value.value()) {
                                Ok(v) => Some(v),
                                _ => continue,
                            }
                        }
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
    let temp_dir = if let Some(path) = configuration.path {
        quote! {
            Builder::new().tempdir_in(#path)
        }
    } else {
        quote! {
            Builder::new().tempdir()
        }
    };
    let mut test_fn = parse_macro_input!(input as ItemFn);
    // Wrapping function will keep the existing function name
    // Existing function will be renamed with a 'wrapped_' prefix
    let fn_ident = test_fn.ident.clone();
    let attributes = test_fn.attrs.clone();
    test_fn.attrs = Vec::new();
    let wrapped = quote! {
        #(#attributes)*
        fn #fn_ident() {
            use tempfile::Builder;
            #test_fn
            let temp_dir = #temp_dir.expect("Failed to create a temporary folder");
            #fn_ident(&temp_dir.path());
        }
    };
    return wrapped.into();
}
