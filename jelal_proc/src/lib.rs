#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Abi, Attribute, Ident, ImplItem, ImplItemFn, ItemFn, ItemImpl, LitStr, Meta, Token,
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

/// Modifiers for a function (see [`fn_attr`] for more information).
///
/// As of now, since only one parameter is allowed, this is an enum, later, maybe not.
enum FnAttr {
    Extern(LitStr),
    Unextern,
    Const,
    Unconst,
}

impl FnAttr {
    /// Apply the modifications on the given function.
    pub fn apply_to(&self, fun: &mut ItemFn) {
        match self {
            FnAttr::Extern(abi) => {
                fun.sig.abi = Some(Abi {
                    extern_token: Default::default(),
                    name: Some(abi.clone()),
                })
            }
            FnAttr::Unextern => fun.sig.abi = None,
            FnAttr::Const => fun.sig.constness = Some(Default::default()),
            FnAttr::Unconst => fun.sig.constness = None,
        }
    }
}

impl Parse for FnAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![extern]) {
            input.parse::<Token![extern]>().unwrap();
            return Ok(Self::Extern(input.parse::<LitStr>()?));
        }
        if lookahead.peek(Token![const]) {
            input.parse::<Token![const]>().unwrap();
            return Ok(Self::Const);
        }
        if lookahead.peek(Ident) {
            match input.parse::<Ident>().unwrap().to_string().as_str() {
                "unconst" => return Ok(Self::Unconst),
                "unextern" => return Ok(Self::Unextern),
                _ => {}
            }
        }

        Err(lookahead.error())
    }
}

/// Change a function signiture indirectly.
///
/// This helps modify a function signiture for specific interfaces conditionally. The allowed inputs
/// are as follow:
///
/// #### Extern (ABI)
///
/// ##### Adding an `extern`
///
/// ```rust,ignore
/// #[cfg_attr(criterion, fn_attr(extern "ABI"))] // where "ABI" can be "C" for example
/// fn f() {}
/// // turns to:
/// extern "ABI" fn f() {}
/// ```
///
/// ##### Removing an `extern`
///
/// ```rust,ignore
/// #[cfg_attr(criterion, fn_attr(unextern))]
/// extern "ABI" fn f() {}
/// // turns to:
/// fn f() {}
/// ```
///
/// #### Constness
///
/// ##### Marking as `const`
///
/// ```rust,ignore
/// #[cfg_attr(criterion, fn_attr(const))]
/// fn f() {}
/// // turns to:
/// const fn f() {}
/// ```
///
/// ##### Unmarking as `const`
///
/// ```rust,ignore
/// #[cfg_attr(criterion, fn_attr(unconst))]
/// const fn f() {}
/// // turns to:
/// fn f() {}
/// ```
#[proc_macro_attribute]
pub fn fn_attr(args: TokenStream, tokens: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as FnAttr);
    let mut fun = parse_macro_input!(tokens as ItemFn);

    // TODO document and change the docstring of the function explaining the possible changes.
    args.apply_to(&mut fun);

    fun.into_token_stream().into()
}

/// Checks if any of the two given criterions are enabled at once.
///
/// Contrary to what Cargo suggests about features, this helps to enable mutually exlcusive
/// features.
#[proc_macro]
pub fn forbid_mutual_feature(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<LitStr, Token![,]>::parse_terminated);

    // make all(item, any(any_other_items,...)), for each item in the parsed list
    let criteria = args.iter().map(|i| {
        let any_other = args
            .iter()
            .filter(|j| i.to_token_stream().to_string() != j.to_token_stream().to_string());
        quote! { all(feature = #i, any(#(feature = #any_other,)*)) }
    });

    let message = format!(
        "only one of the features specified in the following list is allowed at a time: {:?}",
        args.iter().map(|i| i.value()).collect::<Vec<_>>()
    );

    quote! {
        #[cfg(any(#(#criteria,)*))]
        compile_error!(#message);

    }
    .into()
}
