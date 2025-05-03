//! Create bindgens for functions and else.
//!
//! This is a temporary fix for issues in PyO3. Read the README and the descriptions on
//! [`py_attr()`].

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::ParseStream, parse_macro_input, Attribute, ImplItem, ImplItemFn, ItemImpl, Meta, Token,
};

/// Create bindgens for conditional pyo3.
///
/// PyO3 generally does not support attributes under conditional clauses
/// (<https://github.com/PyO3/pyo3/issues/780>). Simply, when the header macro of a block like
/// `pymethods` is conditional (via `cfg_attr`), the nested macros (like `new`) will misbehave.
///
/// As of now, this only supports usage for `impl` blocks and only with `pymethods` input similar to
/// the example below:
///
/// ```rust,ignore
/// #[pymethods]
/// impl X {
///     #[new]
///     pub fn new() -> Self { todo!() }
/// }
///
/// // can be conditionally written like so all the functions will have the given attribute:
///
/// #[cfg_attr(criterion, py_attr(pymethods, new))]
/// impl X {
///     pub fn new() -> Self { todo!() }
/// }
/// ```
#[proc_macro_attribute]
pub fn py_attr(args: TokenStream, tokens: TokenStream) -> TokenStream {
    let arg_parser = |input: ParseStream| {
        let parent = input.parse::<Meta>()?;
        input.parse::<Token![,]>()?;
        let meta = input.parse::<Meta>()?;
        Ok((parent, meta))
    };

    let (parent, meta) = parse_macro_input!(args with arg_parser);

    if parent.path().is_ident("pymethods") {
        let mut item_impl = parse_macro_input!(tokens as ItemImpl);
        for i in item_impl.items.iter_mut() {
            match i {
                ImplItem::Fn(ImplItemFn { attrs, .. }) => attrs.push(Attribute {
                    pound_token: Default::default(),
                    style: syn::AttrStyle::Outer,
                    bracket_token: syn::token::Bracket::default(),
                    meta: meta.clone(),
                }),
                _ => continue,
            };
        }
        quote! {
            #[#parent]
            #item_impl
        }
        .into()
    } else {
        panic!("unsupported parent value");
    }
}
