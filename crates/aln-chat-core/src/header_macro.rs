use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, LitStr};

/// Macro to construct a compile‑time guarded ALN chat block header.
///
/// Guarantees:
/// - DID and Bostrom are bound to concrete consts.
/// - EvidenceBundle has exactly 10 tags at construction site.
/// - Parent hash is wired explicitly (or None for genesis).
#[proc_macro]
pub fn aln_block_header_v1(input: TokenStream) -> TokenStream {
    // Expected syntax:
    // aln_block_header_v1!(
    //     block_ident,
    //     HOST_DID_CONST,
    //     BOSTROM_ADDR_CONST,
    //     "env-label",
    //     "domain-label",
    //     evidence_bundle_expr,
    //     parent_hash_expr
    // );

    let args = parse_macro_input!(input as syn::ExprTuple);
    let block_ident = match &args.elems[0] {
        syn::Expr::Path(p) => p.path.get_ident().cloned().unwrap(),
        _ => panic!("First arg must be an identifier for the block name"),
    };

    let host_did_ident = match &args.elems[1] {
        syn::Expr::Path(p) => p.path.get_ident().cloned().unwrap(),
        _ => panic!("Second arg must be a HostDid const identifier"),
    };

    let bostrom_ident = match &args.elems[2] {
        syn::Expr::Path(p) => p.path.get_ident().cloned().unwrap(),
        _ => panic!("Third arg must be a BostromAddress const identifier"),
    };

    let env_label_lit = match &args.elems[3] {
        syn::Expr::Lit(l) => match &l.lit {
            syn::Lit::Str(s) => s.clone(),
            _ => panic!("Fourth arg must be a string literal env_label"),
        },
        _ => panic!("Fourth arg must be a string literal env_label"),
    };

    let domain_lit = match &args.elems[4] {
        syn::Expr::Lit(l) => match &l.lit {
            syn::Lit::Str(s) => s.clone(),
            _ => panic!("Fifth arg must be a string literal domain"),
        },
        _ => panic!("Fifth arg must be a string literal domain"),
    };

    let evidence_expr = &args.elems[5];
    let parent_hash_expr = &args.elems[6];

    let env_label: LitStr = env_label_lit;
    let domain: LitStr = domain_lit;

    // At compile time, we can only partially check evidence cardinality; a
    // dedicated const fn in bioscale_evidence enforces len == 10.
    let expanded = quote! {
        const #block_ident: &::aln_chat_core::AlnChatBlockHeader = {
            use aln_chat_core::AlnChatBlockHeader;
            use bioscale_evidence::assert_evidence_len_10;

            const EVID: ::bioscale_evidence::EvidenceBundle = #evidence_expr;
            const _: () = assert_evidence_len_10(&EVID);

            &AlnChatBlockHeader {
                host_did: #host_did_ident,
                bostrom: #bostrom_ident,
                env_label: #env_label,
                domain: #domain,
                evidence: EVID,
                parent_hash_hex: #parent_hash_expr,
            }
        };
    };

    TokenStream::from(expanded)
}

/// Header subset used by the macro; expanded into full `AlnChatBlock` at runtime.
#[derive(Debug, Clone)]
pub struct AlnChatBlockHeader {
    pub host_did: HostDid,
    pub bostrom: BostromAddress,
    pub env_label: &'static str,
    pub domain: &'static str,
    pub evidence: EvidenceBundle,
    pub parent_hash_hex: Option<&'static str>,
}
