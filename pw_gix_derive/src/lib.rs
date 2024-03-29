// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[proc_macro_derive(PWO)]
pub fn pwo_derive(input: TokenStream) -> TokenStream {
    let parsed_input: syn::DeriveInput = syn::parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    match parsed_input.data {
        syn::Data::Struct(s) => {
            if let syn::Fields::Named(fields) = s.fields {
                if let Some(field) = fields.named.first() {
                    if let Some(ref ff_id) = field.ident {
                        if let syn::Type::Path(ref ff_ty) = field.ty {
                            let (impl_generics, ty_generics, where_clause) =
                                parsed_input.generics.split_for_impl();
                            let tokens = quote! {
                                impl #impl_generics PackableWidgetObject for #struct_name #ty_generics #where_clause {
                                    type PWT = #ff_ty;

                                    fn pwo(&self) -> &Self::PWT {
                                        &self.#ff_id
                                    }
                                }
                            };
                            return proc_macro::TokenStream::from(tokens);
                        } else {
                            let tokens = quote_spanned! {
                                field.ty.span()=> compile_error!("'PWO': unexpected type")
                            };
                            return proc_macro::TokenStream::from(tokens);
                        }
                    }
                }
            }
        }
        _ => {
            let tokens = quote_spanned! {
                struct_name.span()=> compile_error!("'PWO' is only derivable for structs")
            };
            return proc_macro::TokenStream::from(tokens);
        }
    }
    let tokens = quote_spanned! {
        struct_name.span()=> compile_error!("'PWO' requires at least one named field")
    };
    proc_macro::TokenStream::from(tokens)
}

#[proc_macro_derive(PWO2)]
pub fn pwo2_derive(input: TokenStream) -> TokenStream {
    let parsed_input: syn::DeriveInput = syn::parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    match parsed_input.data {
        syn::Data::Struct(s) => {
            if let syn::Fields::Named(fields) = s.fields {
                if let Some(field) = fields.named.first() {
                    if let Some(ref ff_id) = field.ident {
                        if let syn::Type::Path(ref ff_ty) = field.ty {
                            let (impl_generics, ty_generics, where_clause) =
                                parsed_input.generics.split_for_impl();
                            let tokens = quote! {
                                impl #impl_generics PackableWidgetObject2 for #struct_name #ty_generics #where_clause {
                                    type PWT = #ff_ty;

                                    fn pwo(&self) -> &Self::PWT {
                                        &self.#ff_id
                                    }
                                }
                            };
                            return proc_macro::TokenStream::from(tokens);
                        } else {
                            let tokens = quote_spanned! {
                                field.ty.span()=> compile_error!("'PWO': unexpected type")
                            };
                            return proc_macro::TokenStream::from(tokens);
                        }
                    }
                }
            }
        }
        _ => {
            let tokens = quote_spanned! {
                struct_name.span()=> compile_error!("'PWO' is only derivable for structs")
            };
            return proc_macro::TokenStream::from(tokens);
        }
    }
    let tokens = quote_spanned! {
        struct_name.span()=> compile_error!("'PWO' requires at least one named field")
    };
    proc_macro::TokenStream::from(tokens)
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
