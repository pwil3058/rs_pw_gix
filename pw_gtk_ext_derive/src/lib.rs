// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(WClone)]
pub fn wrapper_derive(input: TokenStream) -> TokenStream {
    let parsed_input: syn::DeriveInput = syn::parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    let (impl_generics, ty_generics, where_clause) = parsed_input.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics Clone for #struct_name #ty_generics #where_clause {
            fn clone(&self) -> Self {
                Self(Rc::clone(&self.0))
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}
