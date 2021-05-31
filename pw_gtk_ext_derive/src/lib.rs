// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};

#[proc_macro_derive(PWO)]
pub fn pwo_derive(input: TokenStream) -> TokenStream {
    let parsed_input: syn::DeriveInput = syn::parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    let error_tokens = quote_spanned! {
        struct_name.span()=> compile_error!("'PWO' derive failed")
    };
    match parsed_input.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(fields) => match fields.named.first() {
                Some(field) => match field.ident {
                    Some(ref ff_id) => {
                        let (impl_generics, ty_generics, where_clause) =
                            parsed_input.generics.split_for_impl();
                        let tokens = quote! {
                            impl #impl_generics PackableWidgetObject for #struct_name #ty_generics #where_clause {
                                fn pwo(&self) -> gtk::Widget {
                                    self.#ff_id.clone().dynamic_cast::<gtk::Widget>().unwrap()
                                }
                            }
                        };
                        proc_macro::TokenStream::from(tokens)
                    }
                    _ => proc_macro::TokenStream::from(error_tokens),
                },
                _ => proc_macro::TokenStream::from(error_tokens),
            },
            syn::Fields::Unnamed(_fields) => {
                let (impl_generics, ty_generics, where_clause) =
                    parsed_input.generics.split_for_impl();
                let tokens = quote! {
                    impl #impl_generics PackableWidgetObject for #struct_name #ty_generics #where_clause {
                        fn pwo(&self) -> gtk::Widget {
                            self.0.pwo()
                        }
                    }
                };
                proc_macro::TokenStream::from(tokens)
            }
            _ => proc_macro::TokenStream::from(error_tokens),
        },
        _ => proc_macro::TokenStream::from(error_tokens),
    }
}

#[proc_macro_derive(Wrapper)]
pub fn wrapper_derive(input: TokenStream) -> TokenStream {
    let parsed_input: syn::DeriveInput = syn::parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    let (impl_generics, ty_generics, where_clause) = parsed_input.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics TopGtkWindow for #struct_name #ty_generics #where_clause {
            fn get_toplevel_gtk_window(&self) -> Option<gtk::Window> {
                if let Some(widget) = self.pwo().get_toplevel() {
                    if widget.is_toplevel() {
                        if let Ok(window) = widget.dynamic_cast::<gtk::Window>() {
                            return Some(window)
                        }
                    }
                };
                None
            }
        }

        impl #impl_generics DialogUser for #struct_name #ty_generics #where_clause {}

        impl #impl_generics WidgetWrapper for #struct_name #ty_generics #where_clause {}
    };
    proc_macro::TokenStream::from(tokens)
}

#[proc_macro_derive(WClone)]
pub fn wclone_derive(input: TokenStream) -> TokenStream {
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
