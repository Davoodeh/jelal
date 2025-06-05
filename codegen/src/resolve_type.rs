//! This module helps resolve types and recognize primitives for better FFI compatibility.

use std::collections::HashMap;

use quote::ToTokens;
use syn::{visit::*, Ident, Type};

#[derive(Clone, Debug)]
pub enum ReprInt {
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,
}

impl ReprInt {
    pub fn try_from_str(s: &str) -> Option<Self> {
        Some(match s {
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u32" => Self::U32,
            "u64" => Self::U64,
            "u128" => Self::U128,
            "usize" => Self::USize,
            "i8" => Self::I8,
            "i16" => Self::I16,
            "i32" => Self::I32,
            "i64" => Self::I64,
            "i128" => Self::I128,
            "isize" => Self::ISize,
            _ => return None,
        })
    }
}

impl Into<&'static str> for ReprInt {
    fn into(self) -> &'static str {
        match self {
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
            Self::USize => "usize",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::I128 => "i128",
            Self::ISize => "isize",
        }
    }
}

impl ToTokens for ReprInt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty: Type = self.clone().into();
        ty.to_tokens(tokens);
    }
}

impl Into<Type> for ReprInt {
    fn into(self) -> Type {
        let s: &str = self.into();
        syn::parse_str(s).unwrap()
    }
}

impl Into<Ident> for ReprInt {
    fn into(self) -> Ident {
        let s: &str = self.into();
        syn::parse_str(s).unwrap()
    }
}

/// Resolves types of a module to their primitives (see [`TypeResolverError`] for limitations).
///
/// As of now, just resolves a single field `struct` and/or type aliases.
#[derive(Debug, Default, Clone)]
pub struct TypeResolver {
    /// Values that are essentially equal.
    pub aliases: HashMap<String, String>,
    /// Rest of the types that may be convertable to each other.
    pub from_into_map: HashMap<String, String>,
}

impl TypeResolver {
    /// Return the closest alias to the given type which equats to the [`Self::repr_int`] result.
    pub fn repr_alias<'a>(&'a self, k: &'a str) -> Option<&'a str> {
        let alias_k = if let Some(from_into_result) = self.from_into_map.get(k) {
            if ReprInt::try_from_str(from_into_result).is_some() {
                return Some(from_into_result);
            }

            from_into_result.as_str()
        } else {
            k
        };

        let alias = self.aliases.get(alias_k)?;

        // if an alias is valid, just return that alias instead of converting to keep readablity
        ReprInt::try_from_str(alias).map(|_| alias_k)
    }
}

impl<'ast> Visit<'ast> for TypeResolver {
    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {
        let ident = i.ident.to_string();

        self.aliases
            .insert(ident, i.ty.to_token_stream().to_string());

        visit_item_type(self, i);
    }

    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        let ident = i.ident.to_string();

        let mut fields = i.fields.iter();
        let ty = fields
            .next()
            .map(|field| field.ty.to_token_stream().to_string())
            .unwrap_or("()".into());

        if fields.next().is_some() {
            return;
        }

        self.from_into_map.insert(ident, ty);
        visit_item_struct(self, i);
    }
}
