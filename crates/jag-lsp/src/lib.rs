// File: crates/jag-lsp/src/lib.rs
pub mod index;
pub mod server;
pub use index::{SymbolIndex, SharedSymbolIndex};
pub use server::{JagLanguageServer, start_lsp_server};
