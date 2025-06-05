//! Holds extra functions that are more general purpose than a module's.

use quote::ToTokens;
use syn::{
    parse::{Parse, Parser},
    Ident, Item,
};

/// Determine if the given generics is empty or not.
pub fn is_generics_empty(generics: &syn::Generics) -> bool {
    generics.lt_token.is_none() && generics.params.is_empty() && generics.where_clause.is_none()
}

/// Return true if this input can be parsed as an ident.
pub fn is_ident<I>(i: I) -> bool
where
    I: ToTokens,
{
    as_ident(i).is_some()
}

/// Convert a type to an ident if possible.
pub fn as_ident<I>(i: I) -> Option<Ident>
where
    I: ToTokens,
{
    Ident::parse.parse2(i.to_token_stream()).ok()
}

/// A simple path type with no `<>` at its end section.
pub fn is_simple_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(path_type) => path_type
            .path
            .segments
            .last()
            .is_some_and(|i| !matches!(i.arguments, syn::PathArguments::AngleBracketed(_))),
        _ => false,
    }
}

/// Remove empty items from the list of items.
pub fn remove_empty_items(items: &mut Vec<Item>) {
    items.retain(|i| match i {
        Item::Impl(v) => v.trait_.is_some() || !v.items.is_empty(),
        Item::Verbatim(v) => !v.is_empty(),
        _ => true,
    });
}
