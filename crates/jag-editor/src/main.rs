// File: crates/jag-editor/src/main.rs
use jag_editor::run;

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run());
}
