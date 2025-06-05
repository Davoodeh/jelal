#![doc = include_str!("../README.md")]
//!
//! Derives in this crate are not expected to be insensitive to the order of fields. Revise the
//! documentation before usage.

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

/// Match a data struct with at least one field or error.
fn match_data_struct(derive_input: &DeriveInput) -> syn::Result<&syn::DataStruct> {
    let syn::Data::Struct(data_struct) = &derive_input.data else {
        return Err(syn::Error::new(
            derive_input.span(),
            "only structs are supported for this derive macro",
        ));
    };

    if data_struct.fields.len() == 0 {
        return Err(syn::Error::new(
            derive_input.span(),
            "use PartialOrd and Ord derives for fieldless types",
        ));
    }

    Ok(data_struct)
}

/// Automatically order fields based on values in order of definition.
///
/// Set `use_cmp` to use the const-time `cmp` of the field instead of using a primitive compare
/// which is only available for primitive types.
///
/// By using the `use_cmp` on the struct name, all the fields are assumed to have `cmp` const
/// function. Use it on fields to only mark those.
#[proc_macro_derive(ConstFieldOrder, attributes(use_cmp))]
pub fn const_ord_field_order(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let data_struct = match match_data_struct(&input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into_token_stream().into(),
    };

    let has_use_cmp_attr = |attrs: &Vec<syn::Attribute>| {
        attrs
            .iter()
            .find(|i| i.meta.path().is_ident("use_cmp"))
            .is_some()
    };

    let global_uses_cmp = has_use_cmp_attr(&input.attrs);

    let fields = &data_struct.fields;

    let ident = &input.ident;
    let var = format_ident!("other");
    let ordering = quote! { ::core::cmp::Ordering };

    let members = fields.members().collect::<Vec<_>>();

    let body = fields.iter().zip(members.into_iter()).rev().fold(
        proc_macro2::TokenStream::default(),
        |acc, (field, member)| {
            let uses_cmp = global_uses_cmp || has_use_cmp_attr(&field.attrs);
            let ty = &field.ty;

            let acc = match (uses_cmp, acc.is_empty()) {
                (true, true) | (false, false) => acc, // deepest object or the most recent primitive
                (true, false) => quote! { .then(#acc) }, // not-primitive and the most recent
                (false, true) => quote! { #ordering::Equal }, // deepest primitive
            };

            if uses_cmp {
                quote! { #ty::cmp(&self.#member, &#var.#member) #acc }
            } else {
                quote! {
                    if self.#member < #var.#member {
                        #ordering::Less
                    } else if self.#member > #var.#member {
                        #ordering::Greater
                    } else {
                        #acc
                    }
                }
            }
        },
    );

    quote! {
        #[automatically_derived]
        impl #ident {
            /// Const-context definition of [`Ord::cmp`].
            pub const fn cmp(&self, #var: &Self) -> #ordering {
                #body
            }
        }

        #[automatically_derived]
        impl ::core::cmp::PartialOrd for #ident {
            fn partial_cmp(&self, #var: &Self) -> Option<#ordering> {
                Some(Ord::cmp(self, #var))
            }
        }
    }
    .into()
}
