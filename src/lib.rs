extern crate alloc;
extern crate proc_macro;

use alloc::vec::IntoIter;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
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
    let temp_dir = if let Some(path) = path {
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
