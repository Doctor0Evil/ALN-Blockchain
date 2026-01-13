use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// #[derive(BioscaleUpgrade)] generates a helper that wraps the type
/// into an EvidenceEnvelope-like struct (kept generic to avoid coupling).
#[proc_macro_derive(BioscaleUpgrade)]
pub fn bioscale_upgrade(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let gen = quote! {
        impl #name {
            pub fn to_envelope_json(&self) -> String {
                serde_json::to_string(self).expect("serialize upgrade")
            }
        }
    };
    gen.into()
}
