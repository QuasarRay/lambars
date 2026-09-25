use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, DeriveInput, ItemFn, LitStr, Token};

struct VerificationArgs {
    id: LitStr,
}

impl Parse for VerificationArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let key: syn::Ident = input.parse()?;
        if key != "id" {
            return Err(syn::Error::new(key.span(), "expected `id`"));
        }
        input.parse::<Token![=]>()?;
        Ok(Self { id: input.parse()? })
    }
}

/// Attribute macro attaching one canonical specification ID to a predicate.
#[proc_macro_attribute]
pub fn verification_case(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as VerificationArgs);
    let function = parse_macro_input!(input as ItemFn);
    let id = args.id;
    let name = &function.sig.ident;
    let marker = format_ident!(
        "__LAM_BARS_SPEC_ID_{}",
        name.to_string().to_uppercase()
    );
    quote! {
        #function

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const #marker: &str = #id;
    }
    .into()
}

/// Derive macro registering a type as a reusable verification model.
#[proc_macro_derive(VerificationModel)]
pub fn derive_verification_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    quote! {
        impl ::lambars_verification::VerificationModel for #ident {
            const TYPE_NAME: &'static str = stringify!(#ident);
        }
    }
    .into()
}

struct DualVerifyInput {
    name: syn::Ident,
    _comma1: Token![,],
    id: LitStr,
    _comma2: Token![,],
    body: syn::Block,
}

impl Parse for DualVerifyInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _comma1: input.parse()?,
            id: input.parse()?,
            _comma2: input.parse()?,
            body: input.parse()?,
        })
    }
}

/// Function-like proc macro generating a runtime regression plus a Kani harness
/// from one ordinary-Rust semantic predicate.
///
/// Verus consumes the same predicate through the verifier-specific adapter
/// package; this keeps verifier syntax out of the production crate.
#[proc_macro]
pub fn dual_verify(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DualVerifyInput);
    let name = input.name;
    let id = input.id;
    let body = input.body;
    let kani_name = format_ident!("kani_{}", name);
    let rust_name = format_ident!("runtime_{}", name);

    quote! {
        #[allow(dead_code)]
        fn #name() -> bool #body

        #[test]
        fn #rust_name() {
            assert!(
                #name(),
                "specification {} failed in runtime regression",
                #id
            );
        }

        #[cfg(kani)]
        #[kani::proof]
        fn #kani_name() {
            assert!(
                #name(),
                "specification {} failed under Kani",
                #id
            );
        }
    }
    .into()
}
