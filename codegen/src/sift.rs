//! [`Sift`] and filter the unsupported code with this visit.

use quote::ToTokens;
use syn::{visit_mut::*, Attribute, Ident, ImplItem, Item, Signature, Token};

use crate::util::{as_ident, is_generics_empty, is_ident, is_simple_type, remove_empty_items};

/// Remove unacceptable code unfit for FFI.
///
/// See [`VisitMut`] methods below for how the selection is made. Each method (as in its docs),
/// holds information about what it sifts and selects:
///
/// - [`Self::visit_file_mut`]: [`Item`] - global and `impl` items
/// - [`Self::visit_attributes_mut`]: [`Attribute`] - `#[]` and `#![]` macros
pub struct Sift {
    /// Holds the list of acceptable struct idents.
    pub structs_whitelist: Vec<Ident>,
    /// Holds the list of trait implementations that are acceptable.
    ///
    /// If given without the final generics, it will still match.
    pub impl_trait_whitelist: Vec<syn::Path>,
}

impl Sift {
    /// Searches [`Self::impl_trait_whitelist`] and returns the item corresponding to trait.
    pub fn is_trait_in_whitelist<'a, 'b>(
        &'a self,
        trait_: &'b Option<(Option<Token![!]>, syn::Path, Token![for])>,
    ) -> Option<(usize, &'a syn::Path)> {
        match trait_ {
            Some((None, path, _)) => {
                let mut sans_end_arg = path.clone();
                sans_end_arg
                    .segments
                    .last_mut()
                    .map(|i| i.arguments = syn::PathArguments::None);

                let sans_end_arg = sans_end_arg.into_token_stream().to_string();
                let path = path.to_token_stream().to_string();

                self.impl_trait_whitelist.iter().enumerate().find(|(_, i)| {
                    let i = i.to_token_stream().to_string();
                    i == path || i == sans_end_arg
                })
            }
            _ => None,
        }
    }

    /// Pass the trait if it matches completely or without the final generics.
    ///
    /// As an example, `From` in the whitelist should match `From<X>` and `From` and `From<Y>` but
    /// `From<X>` in the whitelist only matches `From<X>`.
    pub fn is_acceptable_trait(
        &self,
        trait_: &Option<(Option<Token![!]>, syn::Path, Token![for])>,
    ) -> bool {
        trait_.is_none() || self.is_trait_in_whitelist(&trait_).is_some()
    }

    /// Accept structures if the ident is in list and not generic.
    ///
    /// A struct is only acceptable when is whitelisted and has no generics.  Generics are either
    /// not supported or hard to work around.
    pub fn is_acceptable_struct(&self, ident: &Ident, generics: &syn::Generics) -> bool {
        self.structs_whitelist.contains(&ident) && is_generics_empty(&generics)
    }

    /// Accept the signature only if all inputs are ident pattern & not generic.
    ///
    /// Enforcing ident patterns (i.e. `var: ty`) helps parsing, changing and validating the inputs
    /// and also helps with generating usage (i.e. `var` as an argument usage).
    pub fn is_acceptable_sig(sig: &Signature) -> bool {
        let generics_empty_ident_inputs = sig.inputs.iter().all(|input| match input {
            syn::FnArg::Receiver(_) => true,
            syn::FnArg::Typed(pat_type) => is_ident(&pat_type.pat),
        });

        let generics_empty_output = match &sig.output {
            syn::ReturnType::Default => true,
            syn::ReturnType::Type(_, ty) => is_simple_type(ty),
        };

        is_generics_empty(&sig.generics)
            && sig.variadic.is_none()
            && generics_empty_ident_inputs
            && generics_empty_output
    }
}

impl VisitMut for Sift {
    /// Sift global items.
    ///
    /// Except the items described below and under the conditions specified, the rest are dropped:
    /// - [`Item::Type`] and [`Item::Const`] are unconditionally selected.
    /// - [`Item::Struct`] is only acceptable if [`Self::is_acceptable_struct`] accepts.
    /// - [`Item::Impl`] is only acceptable if its struct is acceptable and its type is an `Ident`.
    ///   Trait implementations are only allowed if [`Self::is_acceptable_trait`] passes, again,
    ///   only if its type is an `Ident`.  Except the [`ImplItem`] described below and under the
    ///   conditions specified, the rest are dropped:
    ///   - [`ImplItem::Type`] and [`ImplItem::Const`] are unconditionally selected.
    ///   - [`ImplItem::Fn`] is only acceptable if its signature is acceptable (see
    ///     [`Self::is_acceptable_sig`]).
    ///   - Every item from an accepted trait implementation.
    fn visit_file_mut(&mut self, i: &mut syn::File) {
        // TODO split to this and Sift::empty_items
        i.items.retain_mut(|i| match i {
            Item::Type(_) => true,
            Item::Const(_) => true,
            Item::Impl(v) => {
                if let Some(ident) = as_ident(&v.self_ty) {
                    if v.trait_.is_none() {
                        // remove invalid items if not a trait implementation
                        v.items.retain(|i| match i {
                            ImplItem::Const(_) => true,
                            ImplItem::Type(_) => true,
                            ImplItem::Fn(v) => Self::is_acceptable_sig(&v.sig),
                            _ => false,
                        });
                        self.is_acceptable_struct(&ident, &v.generics)
                    } else {
                        self.is_acceptable_trait(&v.trait_)
                    }
                } else {
                    false
                }
            }
            Item::Struct(v) => self.is_acceptable_struct(&v.ident, &v.generics),
            _ => false,
        });
        remove_empty_items(&mut i.items);

        visit_file_mut(self, i);
    }

    /// Select `deprecated`, `doc` and `repr` attributes and only that.
    fn visit_attributes_mut(&mut self, i: &mut Vec<Attribute>) {
        const SIFT: &[&str] = &["doc", "repr", "deprecated"];
        i.retain_mut(|attr| {
            let keep = SIFT.iter().any(|i| attr.path().is_ident(i));
            if keep {
                visit_attribute_mut(self, attr);
            }
            keep
        });
    }
}
