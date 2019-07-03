extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput};

#[proc_macro_attribute]
pub fn test_with_tempdir(attributes: TokenStream, input: TokenStream) -> TokenStream {
    // let attributes = parse_macro_input!(attributes as AttributeArgs);
    let input_ts = parse_macro_input!(input as TokenStream2);
    let mut token_stream_iter = input_ts.clone().into_iter();
    if let Some(TokenTree::Ident(ident)) = token_stream_iter.next() {
        if ident == "fn" {
            if let Some(TokenTree::Ident(function_ident)) = token_stream_iter.next() {
                let wrapped = quote! {
                    #[test]
                    fn wrapped_function() {
                        #input_ts
                        let temp_folder = std::path::Path::new("/tmp/314");
                        // TODO: Create the temporary folder
                        #function_ident (&temp_folder);
                        // TODO: Remove the temporary folder
                    }
                };
                return wrapped.into();
            }
        }
    }
    // let function_name = &input_ts.ident;
    // let wrapped = quote! {
    //     #input_ts
    //     //
    //     // fn wrapped_#function_name() {
    //     //     dbg!("Bon ben c'est pas mal ici !");
    //     //     let path = Path::new("/tmp/42");
    //     //     #function_name(&path)
    //     // }
    // };
    input_ts.into()
}
