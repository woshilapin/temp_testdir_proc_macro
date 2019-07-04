extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::parse_macro_input;

// TODO: Add documentation
#[proc_macro_attribute]
// TODO: Add a possinility to ignore test with #[test_with_tempdir(ignore)]
// TODO: Add a possibility to specify and keep the folder with #[temp_testdir(path = "/tmp/my_folder")]
pub fn test_with_tempdir(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    // TODO: Implement parse for my test function
    let input = parse_macro_input!(input as TokenStream2);
    let mut token_stream_iter = input.clone().into_iter();
    if let Some(TokenTree::Ident(ident)) = token_stream_iter.next() {
        if ident == "fn" {
            if let Some(TokenTree::Ident(function_ident)) = token_stream_iter.next() {
                let function_with_tempdir_name = format!("{}_with_tempdir", function_ident);
                let function_with_tempdir_ident =
                    Ident::new(&function_with_tempdir_name, Span::call_site());
                let wrapped = quote! {
                    #[test]
                    fn #function_with_tempdir_ident() {
                        use temp_testdir::TempDir;
                        #input
                        let temp_dir = TempDir::default();
                        #function_ident(&temp_dir);
                    }
                };
                return wrapped.into();
            }
        }
    }
    input.into()
}
