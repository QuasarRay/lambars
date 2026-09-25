//! Proc-macro and proc-macro2 metaverification infrastructure for Lambars.

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, DeriveInput, Expr, Ident, ItemFn, LitStr, Signature, Token, Visibility,
    parse::Parse, parse::ParseStream, parse_macro_input,
};

struct FormalSpecArgs {
    spec_id: LitStr,
    spec_name: LitStr,
}
impl Parse for FormalSpecArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let spec_id = input.parse()?;
        input.parse::<Token![,]>()?;
        let spec_name = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("expected exactly two string literals"));
        }
        Ok(Self { spec_id, spec_name })
    }
}

/// Associates one ordinary-Rust semantic adapter with a canonical formal spec.
#[proc_macro_attribute]
pub fn formal_spec(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as FormalSpecArgs);
    let function = parse_macro_input!(item as ItemFn);
    let ident = &function.sig.ident;
    let upper = ident.to_string().to_ascii_uppercase();
    let id_const = format_ident!("__FORMAL_SPEC_ID_{upper}");
    let name_const = format_ident!("__FORMAL_SPEC_NAME_{upper}");
    let spec_id = args.spec_id;
    let spec_name = args.spec_name;
    quote! {
        #function
        #[doc(hidden)]
        const #id_const: &str = #spec_id;
        #[doc(hidden)]
        const #name_const: &str = #spec_name;
    }
    .into()
}

/// Derives `lambars_spec::FormalModel`.
///
/// Optional override: `#[formal_model_name("ModelName")]`.
#[proc_macro_derive(FormalModel, attributes(formal_model_name))]
pub fn derive_formal_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let name = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("formal_model_name"))
        .and_then(|attr| attr.parse_args::<LitStr>().ok())
        .map_or_else(|| ident.to_string(), |lit| lit.value());
    quote! {
        impl ::lambars_spec::FormalModel for #ident {
            const FORMAL_MODEL_NAME: &'static str = #name;
        }
    }
    .into()
}

/// Derives `lambars_spec::ProofDelegate`.
///
/// Optional target: `#[proof_delegate_target("inner")]`.
#[proc_macro_derive(ProofDelegate, attributes(proof_delegate_target))]
pub fn derive_proof_delegate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let target = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("proof_delegate_target"))
        .and_then(|attr| attr.parse_args::<LitStr>().ok())
        .map_or_else(|| "self".to_owned(), |lit| lit.value());
    quote! {
        impl ::lambars_spec::ProofDelegate for #ident {
            const PROOF_DELEGATE_TARGET: &'static str = #target;
        }
    }
    .into()
}

struct DelegateInput {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    signature: Signature,
    body: Expr,
}
impl Parse for DelegateInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let signature = input.parse()?;
        input.parse::<Token![=>]>()?;
        let body = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self { attrs, visibility, signature, body })
    }
}

/// Delegate-function proc macro.
///
/// Syntax: `formal_delegate!(pub fn model(x: T) -> U => target(x));`.
#[proc_macro]
pub fn formal_delegate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DelegateInput);
    let attrs = input.attrs;
    let visibility = input.visibility;
    let signature = input.signature;
    let body = input.body;
    quote! {
        #(#attrs)*
        #visibility #signature {
            #body
        }
    }
    .into()
}

/// Generates Kani- and Verus-named ordinary-Rust delegates from one adapter.
///
/// Verifier-specific crates decorate these delegates with backend attributes,
/// avoiding duplicated semantic adapter bodies.
#[proc_macro_attribute]
pub fn dual_backend_adapter(_args: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let ident = &function.sig.ident;
    let kani_ident = format_ident!("{ident}__kani_adapter");
    let verus_ident = format_ident!("{ident}__verus_adapter");
    let inputs: Vec<Ident> = function
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(typed) => match typed.pat.as_ref() {
                syn::Pat::Ident(pat) => Some(pat.ident.clone()),
                _ => None,
            },
            syn::FnArg::Receiver(_) => None,
        })
        .collect();
    if inputs.len() != function.sig.inputs.len() {
        return syn::Error::new_spanned(
            &function.sig,
            "dual_backend_adapter requires free functions with identifier arguments",
        )
        .to_compile_error()
        .into();
    }
    let mut kani_sig = function.sig.clone();
    kani_sig.ident = kani_ident;
    let mut verus_sig = function.sig.clone();
    verus_sig.ident = verus_ident;
    let call = if function.sig.asyncness.is_some() {
        quote!(#ident(#(#inputs),*).await)
    } else {
        quote!(#ident(#(#inputs),*))
    };
    quote! {
        #function
        #[doc(hidden)]
        #kani_sig { #call }
        #[doc(hidden)]
        #verus_sig { #call }
    }
    .into()
}
