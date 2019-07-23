extern crate alloc;
extern crate proc_macro;

use alloc::vec::IntoIter;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

#[derive(Debug, PartialEq)]
enum MacroArgument {
    Ignore,
    // TODO: Transform String into AsRef<Path>
    Path(String),
}

struct MacroArgumentIterator {
    iter_nested_meta: IntoIter<NestedMeta>,
}

impl From<IntoIter<NestedMeta>> for MacroArgumentIterator {
    fn from(iter_nested_meta: IntoIter<NestedMeta>) -> Self {
        MacroArgumentIterator { iter_nested_meta }
    }
}

impl Iterator for MacroArgumentIterator {
    type Item = MacroArgument;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_nested_meta
            .next()
            .and_then(|attribute| match attribute {
                NestedMeta::Meta(Meta::NameValue(name_value)) => {
                    if name_value.ident == "path" {
                        match name_value.lit {
                            Lit::Str(value) => Some(MacroArgument::Path(value.value())),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                NestedMeta::Meta(Meta::Word(ident)) => {
                    if ident == "ignore" {
                        Some(MacroArgument::Ignore)
                    } else {
                        None
                    }
                }
                _ => None,
            })
    }
}

// TODO: Add documentation
#[proc_macro_attribute]
pub fn test_with_tempdir(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attributes as AttributeArgs);
    let mut ignore = false;
    let mut path: Option<String> = None;
    let mut _expect_literal = false;
    for attribute in attributes {
        match attribute {
            NestedMeta::Meta(Meta::NameValue(name_value)) => {
                if name_value.ident == "path" {
                    match name_value.lit {
                        Lit::Str(value) => path = Some(value.value()),
                        _ => continue,
                    };
                }
            }
            NestedMeta::Meta(Meta::Word(ident)) => {
                if ident == "ignore" {
                    ignore = true;
                }
            }
            _ => continue,
        }
    }
    let test_macro = if ignore {
        quote! {
            #[test]
            #[ignore]
        }
    } else {
        quote! {
            #[test]
        }
    };
    // TODO: Implement parse for my test function
    let to_parse = input.clone();
    let input = parse_macro_input!(input as TokenStream2);
    let test_fn = parse_macro_input!(to_parse as ItemFn);
    let temp_dir = if let Some(path) = path {
        quote! {
            TempDir::new(#path, true)
        }
    } else {
        quote! {
            TempDir::default()
        }
    };
    // TODO: Keep the name of the original function for the wrapper and change the name of the existing function (better for test report)
    let function_ident = test_fn.ident.clone();
    let function_with_tempdir_name = format!("{}_with_tempdir", test_fn.ident);
    let function_with_tempdir_ident = Ident::new(&function_with_tempdir_name, Span::call_site());
    let wrapped = quote! {
        #test_macro
        fn #function_with_tempdir_ident() {
            use temp_testdir::TempDir;
            #input
            let temp_dir = #temp_dir;
            #function_ident(&temp_dir);
        }
    };
    return wrapped.into();
}

#[cfg(test)]
mod tests {
    use super::*;

    mod tempdir_macro_argument_iterator {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn from() {
            let ignore = NestedMeta::from(Meta::Word(Ident::new("ignore", Span::call_site())));
            let not_a_valid_arg =
                NestedMeta::from(Meta::Word(Ident::new("not_valid_arg", Span::call_site())));
            let attribute_args = vec![ignore, not_a_valid_arg];
            let mut iter = MacroArgumentIterator::from(attribute_args.into_iter());
            assert_eq!(iter.next().unwrap(), MacroArgument::Ignore);
            assert_eq!(iter.next(), None);
        }
    }
}
