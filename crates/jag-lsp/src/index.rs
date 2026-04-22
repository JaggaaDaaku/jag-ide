// File: crates/jag-lsp/src/index.rs
use std::collections::HashMap;
use tower_lsp::lsp_types::*;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub container_name: Option<String>,
}

#[derive(Default)]
pub struct SymbolIndex {
    // Map: file_uri -> symbols in that file
    files: HashMap<String, Vec<Symbol>>,
    // Map: symbol_name -> all locations (for workspace symbols)
    global_index: HashMap<String, Vec<Location>>,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add or update symbols for a file (called on document change)
    pub fn update_file(&mut self, uri: &str, symbols: Vec<Symbol>) {
        // Remove old entries for this file from global index
        if let Some(old_symbols) = self.files.get(uri) {
            for sym in old_symbols {
                if let Some(locations) = self.global_index.get_mut(&sym.name) {
                    locations.retain(|loc| loc.uri.as_str() != uri);
                }
            }
        }
        
        // Add new symbols
        for sym in &symbols {
            self.global_index
                .entry(sym.name.clone())
                .or_default()
                .push(sym.location.clone());
        }
        
        self.files.insert(uri.to_string(), symbols);
    }
    
    /// Get symbols for a specific file
    pub fn get_file_symbols(&self, uri: &str) -> Option<&Vec<Symbol>> {
        self.files.get(uri)
    }
    
    /// Find symbol by name across workspace
    pub fn find_symbol(&self, name: &str) -> Option<&Vec<Location>> {
        self.global_index.get(name)
    }
    
    /// Find symbol at position (for go-to-definition)
    pub fn find_at_position(&self, uri: &str, position: Position) -> Option<&Symbol> {
        self.files.get(uri)?.iter().find(|sym| {
            let range = &sym.location.range;
            position >= range.start && position <= range.end
        })
    }
}

// Thread-safe wrapper for async access
pub type SharedSymbolIndex = Arc<RwLock<SymbolIndex>>;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_symbol(name: &str, uri: &str, start_line: u32, end_line: u32) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind: SymbolKind::FUNCTION,
            location: Location {
                uri: Url::parse(uri).unwrap(),
                range: Range {
                    start: Position { line: start_line, character: 0 },
                    end: Position { line: end_line, character: 20 },
                },
            },
            container_name: None,
        }
    }

    #[test]
    fn test_update_and_get_file_symbols() {
        let mut index = SymbolIndex::new();
        let uri = "file:///test.rs";
        let sym1 = create_mock_symbol("test_fn", uri, 0, 1);
        let sym2 = create_mock_symbol("another_fn", uri, 5, 10);
        
        let symbols = vec![sym1, sym2];
        index.update_file(uri, symbols);
        
        let retrieved = index.get_file_symbols(uri).unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].name, "test_fn");
    }

    #[test]
    fn test_find_symbol_global() {
        let mut index = SymbolIndex::new();
        let uri1 = "file:///test1.rs";
        let uri2 = "file:///test2.rs";
        
        let sym1 = create_mock_symbol("shared_fn", uri1, 0, 1);
        let sym2 = create_mock_symbol("shared_fn", uri2, 5, 10);
        
        index.update_file(uri1, vec![sym1]);
        index.update_file(uri2, vec![sym2]);
        
        let locations = index.find_symbol("shared_fn").unwrap();
        assert_eq!(locations.len(), 2);
    }
    
    #[test]
    fn test_find_at_position() {
        let mut index = SymbolIndex::new();
        let uri = "file:///test.rs";
        let sym = create_mock_symbol("target_fn", uri, 10, 15);
        
        index.update_file(uri, vec![sym]);
        
        // Exact match start
        assert!(index.find_at_position(uri, Position { line: 10, character: 0 }).is_some());
        // Middle
        assert!(index.find_at_position(uri, Position { line: 12, character: 5 }).is_some());
        // Outside before
        assert!(index.find_at_position(uri, Position { line: 9, character: 0 }).is_none());
        // Outside after
        assert!(index.find_at_position(uri, Position { line: 16, character: 0 }).is_none());
    }
}
