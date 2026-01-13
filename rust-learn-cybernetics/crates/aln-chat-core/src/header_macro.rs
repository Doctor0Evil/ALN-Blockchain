// rust-learn-cybernetics/crates/aln-chat-core/src/header_macro.rs
/// Macro for a simple header literal.
///
/// This is a declarative macro instead of a proc-macro for simplicity.
#[macro_export]
macro_rules! aln_block_header_v1 {
    ($session:expr) => {{
        $crate::block::AlnChatBlockHeader {
            id: uuid::Uuid::new_v4(),
            parent_id: None,
            session_id: $session.to_string(),
        }
    }};
}
