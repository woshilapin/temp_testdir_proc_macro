extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Lit, Meta, NestedMeta};

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
    let input = parse_macro_input!(input as TokenStream2);
    let mut token_stream_iter = input.clone().into_iter();
    if let Some(TokenTree::Ident(ident)) = token_stream_iter.next() {
        if ident == "fn" {
            if let Some(TokenTree::Ident(function_ident)) = token_stream_iter.next() {
                // TODO: Keep the name of the original function for the wrapper and change the name of the existing function (better for test report)
                let function_with_tempdir_name = format!("{}_with_tempdir", function_ident);
                let function_with_tempdir_ident =
                    Ident::new(&function_with_tempdir_name, Span::call_site());
                let temp_dir = if let Some(path) = path {
                    quote! {
                        TempDir::new(#path, true)
                    }
                } else {
                    quote! {
                        TempDir::default()
                    }
                };
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
        }
    }
    input.into()
}
