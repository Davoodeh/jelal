//! Create the FFI compatible Rust code.
//!
//! NOTE Read the descriptions carefully however, this may be slightly outdated.
//!
//! Using `expand` this would be much simpler, but still, this code was already written and has no
//! dependecies. Also the crate is overall small and simple, this (mostly) hardcoded approach was
//! chosen.
//!
//! See [`RustFfi::visit_file_mut`] as it is the main visitor that will construct the Rust FFI code,
//! given a file.
//!
//! Limitations of [`RustFfi`]:
//! - [`RustFfi::use_namespace`] is hardcoded to `crate` which in effect implies that all the
//!   original structs must be available from `use crate::*;` (see more in [`RustFfi::parent`]).
//! - All the items must have [`Ident`] as their idents, type or path (for previous limitations).
//! - No type is allowed to have a string match of `Self` (case-sensitive) in any ident segment
//!   ([`RustFfi::deself`]).
//! - No function or method signiture can have a parameter name ending in `mut`
//!   ([`RustFfi::pat_type_to_usage`]).
//! - All the types and fields must be `transmute` compatible as it is used on everything in the
//!   output.
//! - All the fields must be public and have `Into` and `From` implemented between the field type
//!   and the input of the inner functions.
//! - Macros are not supported so for any value to be seen by this crate, it has to be explicitly
//!   written.
//!
//! Ignored items:
//! - Only "mod style" paths (no generics) are allowed as outputs of items in an `impl` block
//!   ([`RustFfi::visit_impl_item_mut`]).
//! - All enums
//! - All functions
//! - Methods with mutable reference to primitive integers
//! - Traits that are not whitelisted
//!
//! Changes to items:
//! - All global functions in C mode will be `no_mangle` and `extern "C"`.
//! - All functions and methods will be non-const (common `wasm` limitation).
//! - All functions for non-C mode have a common prefix.
//! - Primitive referenced inputs will be converted to owned.
//! - All inputs will be replaced by their simpler equivalent if available (for example structs with
//!   one field will be replaced and at boundaries be converted using `Into` and `From` or
//!   `transmute`).
//! - All attributes will be excluded except `doc` and `repr` which will be defaulted to C feature
//!   (the rest of FFIs don't need `repr`s).
//! - All methods will have a global peer function.
//! - All `impl` const items will have a global peer const.
//! - Types that are marked have a primitive inside will be converted to the primitives with
//!   `transmute` and `into`.
//! - All trait functions will have a common prefix not to interfere with other functions with the
//!   same name.
//!
//! Special methods:
//! - methods with the same name as fields are assumed to be getters and if not
//!   defined will automatically be defined.
//! - `new` is assumed to be the default constructor if it returns Self unconditionally. This method
//!   cannot have `self` in its parameters.

use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream, Parser},
    parse_quote,
    punctuated::Punctuated,
    visit::Visit,
    visit_mut::*,
    File, FnArg, Ident, Item, ItemConst, ItemFn, Signature, Token, Type,
};

use quote::{format_ident, quote, ToTokens};

use crate::{
    resolve_type::TypeResolver,
    sift::Sift,
    util::{as_ident, remove_empty_items},
    C_FEATURE, LIB_NAME, PY_FEATURE, WASM_FEATURE,
};

/// Creates `ImplTraitWhitelist`
macro_rules! impl_trait_whitelist {
    ($i:ident $(, $is:ident)* $(,)?) => {
        /// The traits that should be parsed.
        ///
        /// When adding a variant
        #[derive(Debug, Clone, PartialEq, Eq)]
        enum ImplTraitWhitelist {
            $i,
            $($is,)*
        }

        impl ImplTraitWhitelist {
            /// All the variants in a slice.
            pub const VARIANTS: &[Self] = &[
                Self::$i,
                $(Self::$is,)*
            ];

            pub const VARIANTS_STR: &[&str] = &[
                stringify!($i),
                $(stringify!($is),)*
            ];

            /// Create a path of all the variants.
            // TODO cache or whatever, lazy static? tls?
            pub fn as_path_vec() -> Vec<syn::Path> {
                Self::VARIANTS_STR.into_iter().map(|i| syn::parse_str(i).unwrap()).collect()
            }

            /// Assuming the order is as [`Self::as_path_vec`] or [`Self::VARIANTS`], get by index.
            pub fn from_index(i: usize) -> Option<Self> {
                Self::VARIANTS.get(i).cloned() // Clone just to be like other signatures
            }
        }
    };
}

// TODO add Display and bind it to __str__ and such methods in different languages
// TODO add Eq
// TODO add Ord to comparison methods for different languages
// TODO make sift keep `derive` inputs and this struct to parse them
impl_trait_whitelist!(From, Ord);

/// Ignore lifetime and parse the input as anything with a reference prefix.
fn parse_and_mut(
    input: ParseStream,
) -> syn::Result<(Option<Token![&]>, Option<Token![mut]>, TokenStream)> {
    let and = input.parse::<Option<Token![&]>>()?;
    let _lt = input.parse::<Option<syn::Lifetime>>()?;
    let mutability = input.parse::<Option<Token![mut]>>()?;
    Ok((and, mutability, input.parse::<TokenStream>()?))
}

/// Create an FFI compatible Rust code for `WASM`, `Py`, and `C` features.
pub struct RustFfi {
    /// A cache for newly created items before the end of the visit.
    added_items: Vec<Item>,
    /// A cache for the function that introduces the module for `pyo3` before the end of the visit.
    pymodule: Option<ItemFn>,
    /// The type resolver that [`Self::dissolve`] and its related functions (recognize aliases).
    type_resolver: TypeResolver,
    /// The sift that runs on this file before and after being parsed (discards unsupported items).
    sift: Sift,
    /// Where all the original/parent items are callable from.
    ///
    /// This is used mostly in [`Self::parent`]. The imposed limitation is described there.
    use_namespace: TokenStream,
    /// Holds the last processed item's ident (the struct or `impl` block ident).
    processing_item: Ident,
}

impl RustFfi {
    /// Create standalone functions from this method and add them to the list (no visit).
    fn push_method_fns(&mut self, method: &syn::ImplItemFn) {
        let mut sig = method.sig.clone();
        let (args, conversions) = self.inputs_to_args_stmts(&mut sig, false);

        sig.ident = format_ident!(
            "{}_{}",
            self.processing_item.to_string().to_ascii_lowercase(),
            method.sig.ident
        );
        // deself and dissolve since not in a method and is called in isolation
        if let syn::ReturnType::Type(_, ty) = &mut sig.output {
            *ty = Box::new(self.deself_dissolve(ty));
        }

        let ident = &method.sig.ident;
        let self_ty = &self.processing_item;
        let fn_item = syn::ItemFn {
            attrs: method.attrs.clone(),
            vis: method.vis.clone(),
            sig,
            block: parse_quote! {
                {
                    #(#conversions)*
                    #self_ty::#ident(#args).into() // no transmute allowed, maybe dissolved
                }
            },
        };

        {
            // C version
            let mut fn_item = fn_item.clone();
            fn_item.sig.ident = format_ident!("{}", fn_item.sig.ident);
            fn_item.sig.abi = parse_quote! { extern "C" };
            fn_item
                .attrs
                .push(parse_quote! { #[cfg(feature = #C_FEATURE)] });
            fn_item.attrs.push(parse_quote! { #[unsafe(no_mangle)] });
            self.added_items.push(Item::Fn(fn_item));
        }
        {
            // rest version
            let mut fn_item = fn_item.clone();

            fn_item.vis = syn::Visibility::Public(Default::default()); // wasm_bindgen only: public

            // a prefix is suggested to distinguish since signitures vary moreover, wasm does not
            // accept {class_name}_{method_name} formatted functions when exist in impl blocks
            //
            // wasm-bindgen complains about functions named like {lower_class_name}_{method_name} if
            // the method also exists. Then there is also #138762 with "C" ABI in and how it
            // changes when used in wasm32-unknown-unknown.  So the decision was to disable the
            // compatibility.
            fn_item.sig.ident = format_ident!("_{}", fn_item.sig.ident);
            fn_item.attrs.append(&mut parse_quote! {
                #[cfg_attr(feature = #PY_FEATURE, pyfunction)]
                #[cfg_attr(feature = #WASM_FEATURE, wasm_bindgen)]
            });
            self.pymodule_push(&fn_item.sig.ident, true);
            self.added_items.push(Item::Fn(fn_item));
        }
    }

    /// Add an item to the pymodule (initialize if not already).
    fn pymodule_push(&mut self, ident: &Ident, is_fn: bool) {
        let pymodule = self.pymodule.get_or_insert_with(|| {
            parse_quote! {
                #[cfg(feature = #PY_FEATURE)]
                #[pymodule(name = #LIB_NAME)]
                fn __pymodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
                    Ok(())
                }
            }
        });

        let stmt = if is_fn {
            parse_quote! { m.add_function(wrap_pyfunction!(#ident, m)?)?; }
        } else {
            parse_quote! { m.add_class::<#ident>()?; }
        };

        pymodule.block.stmts.insert(0, stmt);
    }

    /// Create a new instance.
    pub fn new(structs_whitelist: Vec<Ident>) -> Self {
        Self {
            type_resolver: Default::default(),
            added_items: Default::default(),
            pymodule: Default::default(),
            processing_item: format_ident!("_placeholder_"),
            use_namespace: quote! { crate }, // TODO read from args
            sift: Sift {
                structs_whitelist,
                impl_trait_whitelist: ImplTraitWhitelist::as_path_vec(),
            },
        }
    }

    /// Return the original inclusion path for this [`Self::processing_item`].
    ///
    /// For now, just prefixes [`Self::processing_item`] with the input of [`Self::use_namespace`]
    /// which in effect implies that this program expects every item to be accessable (`use`able)
    /// from the given namespace.
    fn parent(&self) -> TokenStream {
        self.parent_of(&self.processing_item)
    }

    /// Return the original item for the given ident from the namespaces (see [`Self::parent`]).
    // TODO make this return a Path (or whatever fits the datatype and usage better)
    fn parent_of(&self, ident: &Ident) -> TokenStream {
        let ns = &self.use_namespace;
        quote! { #ns::#ident }
    }

    /// Replace `Self` in the given `ty` with the given `replacement`.
    fn deself_with(ty: &Type, replacement: &Type) -> Type {
        let deselfed = ty
            .to_token_stream()
            .to_string()
            .replace("Self", &replacement.to_token_stream().to_string());
        syn::parse_str(&deselfed).expect("expected a type with no `Self` in its name")
    }

    /// Do dissolve and deself in succession.
    fn deself(&self, ty: &Type) -> Type {
        let this = syn::parse2(self.processing_item.to_token_stream()).unwrap();
        Self::deself_with(ty, &this)
    }

    /// Replace `Self` with this type.
    ///
    /// TODO HACK this is now just a simple string replacement for `Self` which is error prone for
    /// types that have `Self` in their name like `ThisSelfWrapperStruct`.
    fn deself_dissolve(&self, ty: &Type) -> Type {
        let this = self.processing_item.to_string();
        let this_dissolved = self.dissolve(&this).unwrap_or(&this);
        let deselfed = Self::deself_with(ty, &syn::parse_str(this_dissolved).unwrap())
            .to_token_stream()
            .to_string();

        if let Some(prime) = self.dissolve_as_type(&deselfed) {
            return prime;
        }

        syn::parse_str(&deselfed).expect("expected a type with no `Self` in its name")
    }

    /// Return the primitive FFI equivalent for this type (if available).
    fn dissolve<'a>(&'a self, s: &'a str) -> Option<&'a str> {
        self.type_resolver.repr_alias(&s)
    }

    fn dissolve_as_type(&self, s: &str) -> Option<Type> {
        self.dissolve(s).map(|s| syn::parse_str(s).unwrap())
    }

    /// Assuming pattern is from [`syn::FnArg::Typed`], convert it to an expression (hopefully).
    ///
    /// TODO current method makes a string replacement of `mut ` hence any parameter that ends with
    /// `mut` will cause problem.
    fn pat_type_to_usage(&self, pat_type: &syn::PatType) -> TokenStream {
        let (_, _, usage) = parse_and_mut
            .parse2(pat_type.pat.to_token_stream())
            .unwrap();
        let (and, mut mutability, ty) =
            parse_and_mut.parse2(pat_type.ty.to_token_stream()).unwrap();
        let mut clone = and.map(|_| quote! { .clone() });

        // if it is a primtive, remove the pointer stuff for simpler FFI usage
        if self.dissolve(&ty.to_token_stream().to_string()).is_some() {
            mutability = None;
            clone = None;
        }

        quote! { #and #mutability #usage #clone .into() }
    }

    /// Get `& mut deself_owned deself_dissolved` about an input if not `self`.
    fn process_input(
        &self,
        input: &FnArg,
    ) -> Option<(Option<Token![&]>, Option<Token![mut]>, Type, Option<Type>)> {
        let FnArg::Typed(pat_type) = input else {
            return None;
        };
        let deself = self.deself(&pat_type.ty);

        // Convert the pat_type to its primitive (deref) and add a stmt if conversion was needed
        let (and, mutability, deself_owned_tokens) =
            parse_and_mut.parse2(deself.to_token_stream()).unwrap();
        let deself_dissolved = self
            .dissolve(&deself_owned_tokens.to_string())
            .map(|i| syn::parse_str::<Type>(i).unwrap());
        let deself_owned = Type::parse.parse2(deself_owned_tokens).unwrap();

        Some((and, mutability, deself_owned, deself_dissolved))
    }

    /// Check if the input to a function is FFI workable or not.
    ///
    /// This does also `deself` the type.
    ///
    /// This primarily checks if the value is a mutable integer pointer or not.  Since other than
    /// objects, no `RefAbi` is defined for integer pointers.  The visitor implemented will convert
    /// normal immutable pointers to owned instances. However, since `&mut` is not lightly defined
    /// and also tuple returns are not easily done cross FFI, these functions rejected.
    ///
    /// Also see [`Sift::is_acceptable_sig`]. The functionality there cannot check for types since
    /// no `TypeResolver` is utilized in that struct.
    fn is_acceptable_input(&self, input: &FnArg) -> bool {
        self.process_input(input)
            .map(|(and, mut_, _, dissolved)| and.is_none() || mut_.is_none() || dissolved.is_none())
            .unwrap_or(true)
    }

    /// Dissolve a function and add statements reflecting the new changes.
    ///
    /// If returns Err, a mutable reference to a primitive is used in inputs hence not usable in
    /// FFI. Check inputs and do not pass to this function if [`Self::is_acceptable_input`] returns
    /// false for any.
    fn inputs_to_args_stmts(
        &self,
        sig: &mut Signature,
        is_method: bool,
    ) -> (Punctuated<syn::Expr, Token![,]>, Vec<syn::Stmt>) {
        // convert self to this for not-methods
        if let Some(first) = sig.inputs.first_mut() {
            match first {
                FnArg::Receiver(v) if !is_method => {
                    let ty = &v.ty;
                    *first = parse_quote! { this: #ty };
                }
                _ => {}
            };
        }

        let mut argv = Punctuated::new();
        let mut stmts = vec![];

        // gather usage and usage-convertor statement for self
        if let Some(receiver) = sig.receiver() {
            let parent = self.parent();
            let (prefix, postfix) = match &receiver.reference {
                Some(_) => {
                    let mutability = &receiver.mutability;
                    (quote! { & #mutability }, quote! { .clone() })
                }
                None => Default::default(),
            };
            stmts.push(parse_quote! { let this = self; });
            // this provides "breathing room" for other functions to just rely
            // on `this` instead of "self" whenver receiver was available.
            stmts.push(parse_quote! { let this: #prefix #parent = #prefix this #postfix .into(); });
            argv.push(parse_quote! { this });
        }

        // gather usage and usage-convertor statement for inputs (not self)
        for input in sig.inputs.iter_mut() {
            let Some((and, mut_, deselfed_owned, dissolved)) = self.process_input(&input) else {
                continue;
            };
            let FnArg::Typed(pat_type) = input else {
                continue;
            };

            // NOTE at this point no `&mut PrimitiveInt` must be an argument invalid items must have
            //      been sifted before calling this.  The codegen will not fail but will produce
            //      probably unsupported code.

            // Update with the deselfed or deselfed and dissolved version
            let is_dissolved = dissolved.is_some();
            pat_type.ty = Box::new(match dissolved {
                Some(dissolved) => dissolved,
                None => parse_quote! { #and #mut_ #deselfed_owned },
            });

            let usage = self.pat_type_to_usage(&pat_type);
            if and.is_some() && is_dissolved {
                let pat = &pat_type.pat;
                stmts.push(parse_quote! { let #pat: #deselfed_owned = #usage; });
                argv.push(parse_quote! { &#pat });
            } else {
                argv.push(parse_quote! { #usage });
            }
        }

        (argv, stmts)
    }

    /// Assuming a whitelisted trait is the given `impl`, process it.
    fn process_whitelisted_trait(
        &mut self,
        impl_trait: &mut syn::ItemImpl,
        whitelisted_for: &ImplTraitWhitelist,
    ) {
        // If is a impl for a primitive value, ignore because cannot do it without the trait or
        // generics (for now at least)
        let is_impl_for_primitive = self
            .dissolve(&impl_trait.self_ty.to_token_stream().to_string())
            .is_some();
        let is_generic_impl = !impl_trait.generics.to_token_stream().to_string().is_empty();

        // if ran into this and not a match, should be sifted
        let mut impl_trait = {
            let mut taken = parse_quote! { impl ToBeRemoved {} };
            std::mem::swap(impl_trait, &mut taken);
            taken
        };

        if is_impl_for_primitive || is_generic_impl {
            return;
        }

        match whitelisted_for {
            // From is an impl with only one item. Also, primitives and generics are hard to process
            ImplTraitWhitelist::From if impl_trait.items.len() == 1 => {
                // TODO this has many issues... Replace with a sophisticated algo
                // if could not match a name or whatever, just remove the item and continue
                // From must have one item, no generics and not implemented for a primitive
                if let Some(syn::ImplItem::Fn(fun)) = impl_trait.items.first_mut() {
                    if let Some(from_arg) = impl_trait
                        .trait_
                        .take()
                        .map(|(_, path, _)| {
                            match path.segments.last() {
                                // no need to check ident or length of segs since sift did
                                Some(syn::PathSegment {
                                    arguments: syn::PathArguments::AngleBracketed(generics),
                                    ..
                                }) if generics.args.len() == 1 => {
                                    syn::parse2::<Ident>(generics.args.to_token_stream()).ok()
                                }
                                _ => None,
                            }
                        })
                        .flatten()
                    {
                        let arg_parent = self.parent_of(&from_arg);
                        if let Some(FnArg::Typed(pat_type)) = fun.sig.inputs.first_mut() {
                            // let args = last_path_seg.arguments.
                            // may fail if path is not a straight forward one
                            let ident = format_ident!(
                                "ext_from_{}",
                                from_arg.to_string().to_ascii_lowercase()
                            );
                            fun.sig.ident = ident;

                            fun.attrs.push(parse_quote! {
                                #[doc = " FFI version of a `From` trait implementation"]
                            });
                            pat_type.pat = parse_quote! { value };
                            let parent = self.parent();
                            fun.block = parse_quote! {
                                {
                                    #parent::from(#arg_parent::from(value)).into()
                                }
                            };

                            fun.vis = syn::Visibility::Public(Default::default());

                            self.push_method_fns(&fun);

                            // Create a static compatible for py
                            // TODO put this in a separate method so it can be used in the other place
                            let old_ident = fun.sig.ident.to_string();
                            let mut py_fun = fun.clone();
                            let mut py = impl_trait.clone();
                            py.attrs.append(&mut parse_quote! {
                                #[cfg(feature = #PY_FEATURE)]
                                #[pymethods]
                            });
                            impl_trait.attrs.push(parse_quote! {
                                #[cfg_attr(feature = #WASM_FEATURE, wasm_bindgen)]
                            });
                            py_fun.sig.ident = format_ident!("__py_only_{}", old_ident);
                            py_fun.attrs.append(&mut parse_quote! {
                                #[cfg(feature = #PY_FEATURE)]
                                #[staticmethod]
                                #[pyo3(name = #old_ident)]
                            });
                            py.items = vec![syn::ImplItem::Fn(py_fun)];
                            self.added_items.push(Item::Impl(py));

                            self.added_items.push(Item::Impl(impl_trait));
                        }
                    }
                }
            }
            ImplTraitWhitelist::Ord if impl_trait.items.len() == 1 => {
                if let Some(syn::ImplItem::Fn(fun)) = impl_trait.items.first_mut() {
                    if let Some(FnArg::Typed(pat_type)) = fun.sig.inputs.last_mut() {
                        // Especially wasm, cannot do auto-transparent so have to make explicit `i8`
                        fun.sig.output =
                            syn::ReturnType::Type(Default::default(), parse_quote! { i8 });
                        pat_type.pat = parse_quote! { other };
                        impl_trait.trait_ = None;
                        fun.sig.ident = format_ident!("ext_cmp");

                        let parent = self.parent();

                        fun.block = parse_quote! {
                            {
                                #parent::from(self.clone()).cmp(&#parent::from(other.clone())) as i8
                            }
                        };

                        fun.vis = syn::Visibility::Public(Default::default());

                        self.push_method_fns(&fun);

                        impl_trait.attrs.append(&mut parse_quote! {
                            #[cfg_attr(feature = #WASM_FEATURE, wasm_bindgen)]
                            #[cfg_attr(feature = #PY_FEATURE, pymethods)]
                        });

                        self.added_items.push(Item::Impl(impl_trait));
                    }
                }
            }
            _ => {}
        }
    }

    /// Given a valid `deprecated` attribute, returns a string explaining its situation.
    ///
    /// If not a valid deprecated attribute, returns None.
    fn deprecated_to_doc(deprecated_meta: &syn::Meta) -> Option<String> {
        if !deprecated_meta.path().is_ident("deprecated") {
            return None;
        }

        // add a deprecated note for every deprecated item and remove the deprecated attribute
        //
        // Since this only tackles the deprecated issues as of now, the whole block is dedicated
        // to it
        // extract a lit_str from a value or not
        let lit_str_value = |expr: &syn::Expr| match &expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) => Some(lit_str.value()),
            _ => None,
        };

        let note = match &deprecated_meta {
            syn::Meta::NameValue(kv) => lit_str_value(&kv.value),
            syn::Meta::List(meta_list) => meta_list
                .parse_args_with(Punctuated::<syn::MetaNameValue, Token![,]>::parse_terminated)
                .ok()
                .and_then(|name_values| {
                    let note = name_values
                        .iter()
                        .find(|kv| kv.path.is_ident("note"))
                        .and_then(|kv| lit_str_value(&kv.value));
                    let since = name_values
                        .iter()
                        .find(|kv| kv.path.is_ident("since"))
                        .and_then(|kv| lit_str_value(&kv.value))
                        .map(|i| format!("since `{}`", i));

                    // join or whatever at hand
                    match (note, since) {
                        (Some(note), Some(since)) => {
                            Some(format!("{} ({})", note, since.to_ascii_lowercase()))
                        }
                        (a, b) => a.or(b),
                    }
                }),
            _ => None,
        };

        Some(format!(
            " Deprecated: {}.",
            note.as_deref()
                .unwrap_or("may break in the future versions")
        ))
    }
}

impl VisitMut for RustFfi {
    fn visit_file_mut(&mut self, i: &mut File) {
        self.sift.visit_file_mut(i);
        self.type_resolver.visit_file(i);

        visit_file_mut(self, i);
        remove_empty_items(&mut i.items);

        if let Some(pymodule) = std::mem::take(&mut self.pymodule) {
            i.items.push(Item::Fn(pymodule));
        }

        let mut added_items = std::mem::take(&mut self.added_items);
        remove_empty_items(&mut added_items);
        i.items.append(&mut added_items);
    }

    fn visit_attributes_mut(&mut self, i: &mut Vec<syn::Attribute>) {
        // loop over all items and visit_attribute_mut, except each element may add another
        let mut attr_index = 0;
        while attr_index < i.len() {
            if let Some(deprecated_doc) = Self::deprecated_to_doc(&i[attr_index].meta) {
                // Effectively removes all the docs in FFI by coming the first
                // TODO hence it's important to implement a `collapse-docs`.
                i.insert(0, parse_quote! { #[doc = #deprecated_doc] });
                self.visit_attribute_mut(&mut i[0]);
                i.insert(1, parse_quote! { #[doc = ""] });
                self.visit_attribute_mut(&mut i[1]);
                attr_index += 2;
            }

            self.visit_attribute_mut(&mut i[attr_index]);

            attr_index += 1;
        }
    }

    fn visit_attribute_mut(&mut self, i: &mut syn::Attribute) {
        if i.path().is_ident("repr") {
            // keep the repr for C mode
            let meta = &i.meta;
            i.meta = parse_quote!(cfg_attr(feature = #C_FEATURE, #meta));
        }

        visit_attribute_mut(self, i);
    }

    fn visit_item_struct_mut(&mut self, i: &mut syn::ItemStruct) {
        self.processing_item = i.ident.clone();

        // add a default repr (see also `visit_attribute*`)
        if i.attrs
            .iter_mut()
            .find(|i| i.path().is_ident("repr"))
            .is_none()
        {
            i.attrs.push(parse_quote! { #[repr(C)] });
        }

        // as of now, all attributes are sifted automatically so this is safe and won't invoke
        // "Clone already derived" error
        i.attrs.append(&mut parse_quote! {
            #[cfg_attr(feature = #WASM_FEATURE, wasm_bindgen)]
            #[cfg_attr(feature = #PY_FEATURE, pyclass)]
            #[derive(Clone)]
        });

        visit_item_struct_mut(self, i);

        self.pymodule_push(&i.ident, false);

        let parent = self.parent();
        let members = i.fields.members().collect::<Vec<_>>();
        let ident = &i.ident;
        self.added_items.push(Item::Impl(parse_quote! {
            impl From<#ident> for #parent {
                fn from(value: #ident) -> Self {
                    Self {
                        #(#members: value.#members.into(), )*
                    }
                }
            }
        }));
        self.added_items.push(Item::Impl(parse_quote! {
            impl From<#parent> for #ident {
                fn from(value: #parent) -> Self {
                    Self {
                        #(#members: value.#members.into(),)*
                    }
                }
            }
        }));

        // Implement required dissolve from and intos
        if let Some(dissolved) = self.dissolve_as_type(&i.ident.to_string()) {
            self.added_items.push(Item::Impl(parse_quote! {
                impl From<#dissolved> for #ident {
                    fn from(value: #dissolved) -> Self {
                        #parent::from(value).into()
                    }
                }
            }));
            self.added_items.push(Item::Impl(parse_quote! {
                impl Into<#dissolved> for #ident {
                    fn into(self) -> #dissolved {
                        #parent::from(self).into()
                    }
                }
            }));
        }
    }

    fn visit_item_impl_mut(&mut self, i: &mut syn::ItemImpl) {
        let Some(ident) = as_ident(&i.self_ty) else {
            return;
            // unsifted non-ident `impl.self_ty`: should not happen but still
        };
        self.processing_item = ident;

        // if it's a trait item, do it like this, and else follow the normal procedure
        if let Some(whitelisted) = self
            .sift
            .is_trait_in_whitelist(&i.trait_)
            .map(|(i, _)| ImplTraitWhitelist::from_index(i))
            .flatten()
        {
            self.process_whitelisted_trait(i, &whitelisted);
        }

        // drop invalid functions
        i.items.retain(|i| match i {
            syn::ImplItem::Fn(f) => f.sig.inputs.iter().all(|i| self.is_acceptable_input(i)),
            _ => true,
        });

        visit_item_impl_mut(self, i);

        let self_ty_str = self.processing_item.to_string();

        // this is probably better to be done with extract_if but 1.87
        fn take_items(
            items: &mut Vec<syn::ImplItem>,
            matching: impl Fn(&syn::ImplItem) -> bool,
        ) -> Vec<syn::ImplItem> {
            let mut taken = vec![];
            let mut remains = vec![];

            for i in std::mem::take(items) {
                if matching(&i) {
                    taken.push(i)
                } else {
                    remains.push(i)
                }
            }

            *items = remains;
            taken
        }

        // split and sift items
        let consts = take_items(&mut i.items, |i| matches!(i, syn::ImplItem::Const(_)));
        let mut statics = take_items(&mut i.items, |i| {
            matches!(i,
                syn::ImplItem::Fn(v)
                    if v.sig.receiver().is_none()
                        || v.sig.receiver().is_some_and(|i| i.reference.is_none())
            )
        });

        let i_attrs = &i.attrs;

        // consts
        self.added_items.push(Item::Impl(syn::ItemImpl {
            items: consts,
            attrs: parse_quote! {
                #(#i_attrs)*
                #[cfg_attr(feature = #PY_FEATURE, pymethods)]
            },
            ..i.clone()
        }));

        // statics
        let mut constructor = take_items(&mut statics, |i| {
            matches!(i,
            syn::ImplItem::Fn(syn::ImplItemFn {
                sig:
                    syn::Signature {
                        ident,
                        output: syn::ReturnType::Type(_, ty),
                        ..
                    },
                ..
            }) if ident == "new" && ty.to_token_stream().to_string() == self_ty_str)
        });
        let mut non_py = syn::ItemImpl {
            items: statics,
            ..i.clone()
        };
        let mut py = non_py.clone();
        non_py.attrs.push(parse_quote! {
            #[cfg_attr(feature = #WASM_FEATURE, wasm_bindgen)]
        });
        // there is this lack of feature in pyo3 that doesn't allow conditional (behind cfg_attr)
        // `pymethods` to have inner attributes like `staticmethod` so a `cfg` style duplication is
        // required
        py.attrs.append(&mut parse_quote! {
            #[cfg(feature = #PY_FEATURE)]
            #[pymethods]
        });
        for item in py.items.iter_mut() {
            let syn::ImplItem::Fn(f) = item else {
                continue;
            };
            let old_ident = f.sig.ident.to_string();
            f.vis = syn::Visibility::Inherited;
            f.sig.ident = format_ident!("__py_only_{}", old_ident);
            f.attrs.append(&mut parse_quote! {
                #[cfg(feature = #PY_FEATURE)]
                #[pyo3(name = #old_ident)]
            });

            // already sure that this is not a reference since its in `statics`
            if let Some(FnArg::Receiver(receiver)) = f.sig.inputs.first_mut() {
                *receiver = parse_quote! { &self }; // not static anymore
                let clone = parse_quote! { let this: Self = this.clone(); };
                f.block.stmts.insert(1, clone); // 0: let `this`
            } else {
                f.attrs.push(parse_quote! { #[staticmethod] });
            }
        }
        if let Some(syn::ImplItem::Fn(f)) = constructor.first_mut() {
            non_py.items.push(parse_quote! {
                #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
                #f
            });

            f.sig.ident = format_ident!("__py_only_{}", f.sig.ident);
            py.items.push(parse_quote! {
                #[cfg(feature = #PY_FEATURE)]
                #[new]
                #f
            });
        }
        self.added_items.push(Item::Impl(non_py));
        self.added_items.push(Item::Impl(py));

        // rest
        i.attrs.append(&mut parse_quote! {
            #[cfg_attr(feature = #PY_FEATURE, pymethods)]
            #[cfg_attr(feature = #WASM_FEATURE, wasm_bindgen)]
        });
    }

    fn visit_impl_item_const_mut(&mut self, i: &mut syn::ImplItemConst) {
        let parent = self.parent();
        let ident = &i.ident;
        let ty_deself = self.deself_dissolve(&i.ty);

        // if no change took place after removing `Self`, just use value, else transmute the value
        if i.ty.to_token_stream().to_string() == ty_deself.to_token_stream().to_string() {
            i.expr = parse_quote! { #parent::#ident };
        } else {
            // TODO comment on what it does: Self { x: y } is deselfed and rewritten
            // for cbindgen to include as it won't include transmutation of structs
            match &mut i.expr {
                syn::Expr::Struct(v) if v.qself.is_none() && v.path.is_ident("Self") => {
                    v.path = parse_quote! { #ty_deself };
                    for field in v.fields.iter_mut() {
                        let Ok((_, rest)) = Parser::parse2(
                            |input: ParseStream| {
                                Ok((
                                    input.parse::<Token![Self]>()?,
                                    input.parse::<TokenStream>()?,
                                ))
                            },
                            field.expr.to_token_stream(),
                        ) else {
                            continue;
                        };
                        field.expr = parse_quote! { #parent #rest };
                    }
                }
                v => *v = parse_quote! { unsafe { ::core::mem::transmute(#parent::#ident) } },
            }
        }

        visit_impl_item_const_mut(self, i);

        // Make a global duplicate of this item out of the impl scope
        // TODO Enable for languages
        let const_ident = format_ident!(
            "{}_{}",
            self.processing_item.to_string().to_ascii_uppercase(),
            i.ident
        );
        let const_ident_str = const_ident.to_string();
        // TODO add these to other languages since right now there are not much of a use for them
        self.added_items.push(Item::Const(ItemConst {
            attrs: i.attrs.clone(),
            vis: i.vis.clone(),
            const_token: i.const_token.clone(),
            ident: const_ident.clone(),
            generics: i.generics.clone(),
            colon_token: i.colon_token.clone(),
            ty: Box::new(ty_deself.clone()),
            eq_token: i.eq_token.clone(),
            expr: Box::new(i.expr.clone()),
            semi_token: i.semi_token.clone(),
        }));
        // C statics since `const` won't be linked
        let mut item_static = syn::ItemStatic {
            attrs: i.attrs.clone(),
            vis: i.vis.clone(),
            static_token: Default::default(),
            mutability: syn::StaticMutability::None,
            ident: format_ident!("_{}", const_ident),
            colon_token: Default::default(),
            ty: Box::new(ty_deself),
            eq_token: i.eq_token.clone(),
            expr: Box::new(parse_quote! { #const_ident }),
            semi_token: i.semi_token.clone(),
        };
        item_static
            .attrs
            .push(parse_quote! { #[unsafe(export_name = #const_ident_str)] });
        self.added_items.push(Item::Static(item_static));
    }

    fn visit_impl_item_fn_mut(&mut self, i: &mut syn::ImplItemFn) {
        let (args, conversions) = self.inputs_to_args_stmts(&mut i.sig, true);

        // WASM does not accept any `const fn`
        // ??? will this matter for FFI? should this be behind cfg flags?
        i.sig.constness = None;

        // deself the output
        if let syn::ReturnType::Type(_, ty) = &mut i.sig.output {
            // No need to dissolve since this ruins the functionality of chain method calling in
            // methods. Hence the deself only
            *ty = Box::new(self.deself(ty));
        }

        let parent = self.parent();
        let ident = &i.sig.ident;

        // the methods declared here are trusted so if the results is invalid, that's on author.
        // Hence the transmute
        i.block = parse_quote! {
            {
                #(#conversions)*
                unsafe { ::core::mem::transmute(#parent::#ident(#args)) }
            }
        };

        visit_impl_item_fn_mut(self, i);

        self.push_method_fns(&i);
    }

    fn visit_field_mut(&mut self, i: &mut syn::Field) {
        i.vis = syn::Visibility::Inherited;
        visit_field_mut(self, i);
    }
}
