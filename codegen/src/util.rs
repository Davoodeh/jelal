//! Holds extra functions that are more general purpose than a module's.

use std::io::Write;

use quote::ToTokens;
use syn::{
    parse::{Parse, Parser},
    parse_quote, Ident, Item,
};

use crate::FILES_PREFIX;

/// Prefixes the given path so it will be in the jelal sources.
pub fn prefixed_path(path: &str) -> String {
    format!("{}{}", FILES_PREFIX, path)
}

/// Write the content to the path and create the directory if not there.
pub fn write_output<S: AsRef<std::ffi::OsStr> + ?Sized>(
    path: &S,
    content: impl ToString,
) -> Result<(), std::io::Error> {
    let path = std::path::Path::new(&path);

    // mkdir -p $(basedir $path)
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // echo $src > $path
    std::fs::File::create(path).and_then(|mut i| i.write_all(content.to_string().as_bytes()))
}

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

/// Given an expression, return if the value is literal string.
pub fn lit_str_expr(expr: &syn::Expr) -> Option<&syn::LitStr> {
    match &expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => Some(lit_str),
        _ => None,
    }
}

/// Returns `name = literal_str` if `name` matches the given `ident`.
pub fn name_value_str<'attr, 'ident>(
    attr: &'attr syn::Attribute,
    ident: &'ident str,
) -> Option<&'attr syn::LitStr> {
    match attr.meta.require_name_value() {
        Ok(kv) if attr.path().is_ident(ident) => lit_str_expr(&kv.value),
        _ => None,
    }
}

/// Collapse literal string `doc` attributes regardless of their position.
///
/// This merges each `doc` with its previous if both are literal string. Using `expand`-like
/// commands most `doc` values should be str and have a higher chance of total merge.
//
// Link to the reference, cluster all the docs together and merge the strings
// TODO break to many lines to make for a tidier output
pub fn collapse_docs(attrs: &mut Vec<syn::Attribute>) {
    let mut previous: Option<(String, usize)> = None; // cache the string and its position in attrs

    let mut current_index = 0;
    while current_index < attrs.len() {
        match (name_value_str(&attrs[current_index], "doc"), &mut previous) {
            // if there are any previous_attr (literal string docs), concat with them, else move on
            (Some(current_doc), Some((previous_doc, previous_index))) => {
                // unwraps work since the previous was selected knowing it matches this.
                previous_doc.push('\n');
                previous_doc.push_str(&current_doc.value());

                // concat with the previous attribute
                attrs[*previous_index] = parse_quote! { #[doc = #previous_doc] };

                // upon concatination, remove the value since it's already in the previous
                attrs.remove(current_index);

                // this is safe since the first (0, cause of underflow) cannot come here since a
                // "previous" is required
                current_index -= 1;
            }
            // keep this as the "previous" for the next iteration if an str doc
            (Some(current_doc), None) => previous = Some((current_doc.value(), current_index)),
            // if this doc is not a literal string, then the next values must not add to its
            // previous since the "succession chain" of literals is broken.
            (None, _) => previous = None,
        }

        current_index += 1;
    }
}
