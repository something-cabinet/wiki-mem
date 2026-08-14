//! Code-edge resolution (spec item 2, FR-2.1/FR-2.2).
//!
//! Raw per-file edges (`extract_edges`) carry AST facts but no confirmed
//! targets: `imports` hold path-math candidates, `calls`/`inherits` hold only
//! callee/base names. This module resolves them against the code index and
//! refines provenance:
//!
//! - `Explicit`:  the reference resolves to exactly one defining file.
//! - `Derived`:   resolution went through a re-export/indirection chain
//!   (e.g. `use crate::foo::Bar` where `foo` re-exports `Bar`).
//! - `Ambiguous`: the reference matches multiple candidate files.
//!
//! Resolution is deterministic (sorted candidates, first-wins on ties) and
//! local (no LLM calls — NFR-2.1). It runs at query time against a
//! `CodeIndexSnapshot` so single-file edits never require a full re-index
//! (NFR-2.2).

use std::collections::{HashMap, HashSet};

use crate::models::code_edge_model::CodeEdge;
use crate::models::language_model::SupportedLanguage;
use crate::models::symbol_model::CodeIntelSymbol;
use wm_engine::models::edge_type_model::EdgeProvenance;

use super::code_index_db::CodeIndexDb;
use super::engine_service::resolve_import_candidates;

/// A code edge with resolution applied: targets confirmed against the symbol
/// index and provenance refined. Carries the 1-based `line` in `source_file`
/// where the reference appears (FR-2.3 source locations).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ResolvedCodeEdge {
    /// One of `calls`, `imports`, `inherits`.
    pub edge_type: String,
    /// File (project-relative) containing the reference.
    pub source_file: String,
    /// Enclosing symbol for calls/inherits (caller / implementing type).
    pub source_symbol: Option<String>,
    /// Confirmed target file. Empty when the reference could not be resolved.
    pub target_file: String,
    /// Callee / base name for calls/inherits; import path for imports.
    pub target_symbol: Option<String>,
    /// 1-based line of the reference in `source_file`.
    pub line: usize,
    /// Refined provenance (see module docs).
    pub provenance: EdgeProvenance,
    /// Files traversed when resolution went through re-exports (derived).
    /// Empty for direct edges.
    pub via: Vec<String>,
}

impl ResolvedCodeEdge {
    /// Node id of the source endpoint: `file#symbol` for calls/inherits,
    /// plain `file` for imports.
    pub fn source_node_id(&self) -> String {
        match &self.source_symbol {
            Some(s) => format!("{}#{}", self.source_file, s),
            None => self.source_file.clone(),
        }
    }

    /// Node id of the target endpoint: `file#symbol` when a target symbol is
    /// known, plain `file` otherwise.
    pub fn target_node_id(&self) -> String {
        match &self.target_symbol {
            Some(s) if !self.target_file.is_empty() => format!("{}#{}", self.target_file, s),
            _ => self.target_file.clone(),
        }
    }
}

/// In-memory snapshot of the code index: symbols, raw edges and the indexed
/// file set. Built either from the persisted code index DB or by walking the
/// filesystem (on-demand tools).
#[derive(Debug, Default)]
pub struct CodeIndexSnapshot {
    pub symbols: Vec<CodeIntelSymbol>,
    pub raw_edges: Vec<CodeEdge>,
    pub files: HashSet<String>,
}

impl CodeIndexSnapshot {
    /// Load the full code index from the persisted DB.
    pub fn from_db(db: &CodeIndexDb) -> Result<Self, String> {
        use super::code_index_db::EdgeQuery;
        let symbols = db.query_symbols(None, None, None, None, None, None)?;
        let raw_edges = db.query_edges(&EdgeQuery::default())?;
        let files: HashSet<String> = db.list_files()?.into_iter().collect();
        Ok(Self {
            symbols,
            raw_edges,
            files,
        })
    }

    /// Build a snapshot by walking the filesystem (on-demand tools that do not
    /// rely on a persisted index). Deterministic and local — NFR-2.1.
    pub fn collect_from_fs(project_root: &std::path::Path) -> Result<Self, String> {
        use crate::services::ingest_service::is_skipped_dir;
        use walkdir::WalkDir;
        let mut snapshot = Self::default();
        for entry in WalkDir::new(project_root)
            .into_iter()
            .filter_entry(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| !is_skipped_dir(s))
                    .unwrap_or(false)
            })
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let ext = entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let lang = match SupportedLanguage::from_ext(ext) {
                Some(l) => l,
                None => continue,
            };
            if matches!(lang, SupportedLanguage::Html | SupportedLanguage::Svelte) {
                continue;
            }
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let rel_path = entry
                .path()
                .strip_prefix(project_root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .to_string();
            snapshot
                .symbols
                .extend(crate::services::engine_service::extract_symbols(
                    &content, &rel_path, ext,
                ));
            snapshot
                .raw_edges
                .extend(crate::services::engine_service::extract_edges(
                    &content, &rel_path, ext,
                ));
            snapshot.files.insert(rel_path);
        }
        Ok(snapshot)
    }
}

/// Resolve all raw edges in a snapshot against its symbol index.
pub fn resolve_code_edges(snapshot: &CodeIndexSnapshot) -> Vec<ResolvedCodeEdge> {
    let mut by_name: HashMap<&str, Vec<&CodeIntelSymbol>> = HashMap::new();
    let mut by_file: HashMap<&str, Vec<&CodeIntelSymbol>> = HashMap::new();
    for s in &snapshot.symbols {
        by_name.entry(s.name.as_str()).or_default().push(s);
        by_file.entry(s.file.as_str()).or_default().push(s);
    }
    for v in by_name.values_mut() {
        v.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    }

    let mut edges_by_file: HashMap<&str, Vec<&CodeEdge>> = HashMap::new();
    for e in &snapshot.raw_edges {
        edges_by_file
            .entry(e.source_file.as_str())
            .or_default()
            .push(e);
    }
    for v in edges_by_file.values_mut() {
        v.sort_by_key(|e| (e.line, e.edge_type.as_str()));
    }

    let files: &HashSet<String> = &snapshot.files;
    let mut resolved = Vec::new();
    for raw in &snapshot.raw_edges {
        let r = match raw.edge_type.as_str() {
            "imports" => resolve_import(raw, files, &by_file, &by_name, &edges_by_file),
            "calls" | "inherits" => resolve_symbol_edge(raw, &by_name),
            _ => None,
        };
        if let Some(r) = r {
            resolved.push(r);
        }
    }
    resolved
}

/// Resolve a `calls`/`inherits` edge: find the file(s) defining the callee /
/// base symbol.
fn resolve_symbol_edge(
    raw: &CodeEdge,
    by_name: &HashMap<&str, Vec<&CodeIntelSymbol>>,
) -> Option<ResolvedCodeEdge> {
    let callee = raw.target_symbol.as_deref()?;
    let candidates = by_name.get(callee)?;
    if candidates.is_empty() {
        return None;
    }
    let mut defining_files: Vec<&str> = candidates.iter().map(|s| s.file.as_str()).collect();
    defining_files.sort_unstable();
    defining_files.dedup();

    let provenance = if defining_files.len() > 1 {
        EdgeProvenance::Ambiguous
    } else {
        EdgeProvenance::Explicit
    };

    Some(ResolvedCodeEdge {
        edge_type: raw.edge_type.clone(),
        source_file: raw.source_file.clone(),
        source_symbol: raw.source_symbol.clone(),
        target_file: defining_files[0].to_string(),
        target_symbol: raw.target_symbol.clone(),
        line: raw.line,
        provenance,
        via: Vec::new(),
    })
}

/// Resolve an `imports` edge: path-math candidates filtered against the
/// indexed file set; a bounded re-export chase promotes the edge to `Derived`
/// when the imported symbol is re-exported from another file.
///
/// For Rust, item segments are dropped progressively (`crate::a::b::Item` →
/// `crate::a::b` → `crate::a`) so `use crate::engine::run` resolves to the
/// module file `src/engine.rs` even though `run` is a function, not a module.
fn resolve_import(
    raw: &CodeEdge,
    files: &HashSet<String>,
    by_file: &HashMap<&str, Vec<&CodeIntelSymbol>>,
    by_name: &HashMap<&str, Vec<&CodeIntelSymbol>>,
    edges_by_file: &HashMap<&str, Vec<&CodeEdge>>,
) -> Option<ResolvedCodeEdge> {
    let target = raw.target_symbol.as_deref()?;
    let lang = lang_from_file(&raw.source_file)?;

    // Most-specific-first import path family.
    let family: Vec<String> = match lang {
        SupportedLanguage::Rust => rust_import_prefixes(target),
        _ => vec![target.to_string()],
    };

    let mut matches: Vec<String> = Vec::new();
    let mut matched_prefix: Option<String> = None;
    for t in &family {
        if let Some(cands) = resolve_import_candidates(&raw.source_file, t, &lang) {
            for c in &cands {
                for f in files {
                    if (f == c || f.ends_with(&format!("/{}", c))) && !matches.contains(f) {
                        matches.push(f.clone());
                    }
                }
            }
        }
        if !matches.is_empty() {
            matched_prefix = Some(t.clone());
            break;
        }
    }
    matches.sort();
    if matches.is_empty() {
        return None;
    }

    // Leaf symbol to chase: the trailing item segment beyond the matched
    // prefix (Rust) or the CamelCase tail of the full path (other languages).
    let leaf: Option<String> = if let Some(prefix) = &matched_prefix {
        target
            .strip_prefix(&format!("{}::", prefix))
            .or_else(|| (prefix == target).then_some(""))
            .and_then(|rest| {
                if rest.is_empty() {
                    import_symbol_tail(target).map(|s| s.to_string())
                } else {
                    rest.split("::").next().map(|s| s.to_string())
                }
            })
    } else {
        import_symbol_tail(target).map(|s| s.to_string())
    };

    let mut provenance = EdgeProvenance::Explicit;
    let mut via: Vec<String> = Vec::new();
    let mut target_file = matches[0].clone();

    if let Some(leaf) = leaf {
        let defines_directly = by_file
            .get(matches[0].as_str())
            .map(|syms| syms.iter().any(|s| s.name == leaf))
            .unwrap_or(false);
        if !defines_directly {
            if let Some(target) = chase_reexport(
                &matches[0],
                &leaf,
                by_file,
                by_name,
                edges_by_file,
                files,
                2,
            ) {
                provenance = EdgeProvenance::Derived;
                via.push(matches[0].clone());
                target_file = target;
            }
        }
    }

    if matches.len() > 1 {
        provenance = EdgeProvenance::Ambiguous;
        via.clear();
    }

    Some(ResolvedCodeEdge {
        edge_type: raw.edge_type.clone(),
        source_file: raw.source_file.clone(),
        source_symbol: raw.source_symbol.clone(),
        target_file,
        target_symbol: raw.target_symbol.clone(),
        line: raw.line,
        provenance,
        via,
    })
}

/// Most-specific-first family of Rust import paths:
/// `crate::engine::run` → `["crate::engine::run", "crate::engine"]`.
fn rust_import_prefixes(target: &str) -> Vec<String> {
    let mut out = vec![target.to_string()];
    let mut cur = target;
    while let Some(idx) = cur.rfind("::") {
        let prefix = &cur[..idx];
        if prefix.is_empty() {
            break;
        }
        out.push(prefix.to_string());
        cur = prefix;
    }
    out
}

/// Extract a trailing symbol segment from an import path, if any
/// (`crate::foo::Bar` → `Bar`; `./utils` → `None`).
fn import_symbol_tail(target: &str) -> Option<&str> {
    let last = target.rsplit("::").next()?;
    let is_symbol = last.chars().next().is_some_and(|c| c.is_uppercase())
        && last.starts_with(|c: char| c.is_uppercase());
    if is_symbol && target.contains("::") {
        Some(last)
    } else {
        None
    }
}

/// Follow re-export chains: starting from `file`, if it does not define
/// `symbol` but imports it from another file, follow up to `depth` hops.
fn chase_reexport(
    file: &str,
    symbol: &str,
    by_file: &HashMap<&str, Vec<&CodeIntelSymbol>>,
    by_name: &HashMap<&str, Vec<&CodeIntelSymbol>>,
    edges_by_file: &HashMap<&str, Vec<&CodeEdge>>,
    files: &HashSet<String>,
    depth: usize,
) -> Option<String> {
    if depth == 0 {
        return None;
    }
    let defines = by_file
        .get(file)
        .map(|syms| syms.iter().any(|s| s.name == symbol))
        .unwrap_or(false);
    if defines {
        return Some(file.to_string());
    }
    // Symbol defined anywhere in the index but this file re-exports it.
    by_name.get(symbol)?;
    let edges = edges_by_file.get(file)?;
    for e in edges {
        if e.edge_type != "imports" {
            continue;
        }
        if let Some(leaf) = import_symbol_tail(e.target_symbol.as_deref()?) {
            if leaf != symbol {
                continue;
            }
        }
        let lang = lang_from_file(&e.source_file)?;
        let cands = resolve_import_candidates(&e.source_file, e.target_symbol.as_deref()?, &lang)?;
        for c in cands {
            let mut m: Vec<&String> = files
                .iter()
                .filter(|f| f.as_str() == c || f.ends_with(&format!("/{}", c)))
                .collect();
            m.sort();
            if let Some(next) = m.first() {
                if let Some(found) = chase_reexport(
                    next,
                    symbol,
                    by_file,
                    by_name,
                    edges_by_file,
                    files,
                    depth - 1,
                ) {
                    return Some(found);
                }
            }
        }
    }
    None
}

/// Infer the tree-sitter language for path math from a project-relative file.
fn lang_from_file(file: &str) -> Option<SupportedLanguage> {
    let ext = file.rsplit('.').next()?;
    SupportedLanguage::from_ext(ext)
}

/// Query-time index over resolved code edges. Precomputes outgoing/incoming
/// lookups by symbol node, file node and bare symbol name so `wm_graph`
/// neighbors/affected lookups are O(degree).
pub struct CodeEdgeGraph {
    pub edges: Vec<ResolvedCodeEdge>,
    files: Vec<String>,
    out_by_symbol: HashMap<(String, String), Vec<usize>>,
    in_by_symbol: HashMap<(String, String), Vec<usize>>,
    out_by_file: HashMap<String, Vec<usize>>,
    in_by_file: HashMap<String, Vec<usize>>,
    in_by_symbol_name: HashMap<String, Vec<usize>>,
    by_symbol_name: HashMap<String, Vec<usize>>,
}

impl CodeEdgeGraph {
    pub fn build(edges: Vec<ResolvedCodeEdge>) -> Self {
        let mut file_set: HashSet<&str> = HashSet::new();
        for e in &edges {
            file_set.insert(e.source_file.as_str());
            if !e.target_file.is_empty() {
                file_set.insert(e.target_file.as_str());
            }
        }
        let mut files: Vec<String> = file_set.into_iter().map(|s| s.to_string()).collect();
        files.sort();

        let mut g = CodeEdgeGraph {
            edges,
            files,
            out_by_symbol: HashMap::new(),
            in_by_symbol: HashMap::new(),
            out_by_file: HashMap::new(),
            in_by_file: HashMap::new(),
            in_by_symbol_name: HashMap::new(),
            by_symbol_name: HashMap::new(),
        };
        for (i, e) in g.edges.iter().enumerate() {
            if let Some(s) = &e.source_symbol {
                g.out_by_symbol
                    .entry((e.source_file.clone(), s.clone()))
                    .or_default()
                    .push(i);
                g.by_symbol_name.entry(s.clone()).or_default().push(i);
            }
            g.out_by_file
                .entry(e.source_file.clone())
                .or_default()
                .push(i);
            if !e.target_file.is_empty() {
                g.in_by_file
                    .entry(e.target_file.clone())
                    .or_default()
                    .push(i);
                if let Some(s) = &e.target_symbol {
                    g.in_by_symbol
                        .entry((e.target_file.clone(), s.clone()))
                        .or_default()
                        .push(i);
                    g.in_by_symbol_name.entry(s.clone()).or_default().push(i);
                    g.by_symbol_name.entry(s.clone()).or_default().push(i);
                }
            }
        }
        g
    }

    pub fn files(&self) -> &[String] {
        &self.files
    }

    pub fn has_file(&self, file: &str) -> bool {
        self.files.iter().any(|f| f == file)
    }

    pub fn has_symbol(&self, file: &str, symbol: &str) -> bool {
        self.out_by_symbol
            .contains_key(&(file.to_string(), symbol.to_string()))
            || self
                .in_by_symbol
                .contains_key(&(file.to_string(), symbol.to_string()))
    }

    /// Outgoing edges from a symbol node (calls/inherits).
    pub fn outgoing_from_symbol(&self, file: &str, symbol: &str) -> Vec<&ResolvedCodeEdge> {
        self.out_by_symbol
            .get(&(file.to_string(), symbol.to_string()))
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Incoming edges to a symbol node (callers / implementers).
    pub fn incoming_to_symbol(&self, file: &str, symbol: &str) -> Vec<&ResolvedCodeEdge> {
        self.in_by_symbol
            .get(&(file.to_string(), symbol.to_string()))
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Outgoing edges from a file node (imports + calls/inherits from its symbols).
    pub fn outgoing_from_file(&self, file: &str) -> Vec<&ResolvedCodeEdge> {
        self.out_by_file
            .get(file)
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Incoming edges to a file node (importers + callers of its symbols).
    pub fn incoming_to_file(&self, file: &str) -> Vec<&ResolvedCodeEdge> {
        self.in_by_file
            .get(file)
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Incoming edges whose target symbol matches `name` across all files.
    pub fn incoming_to_symbol_name(&self, name: &str) -> Vec<&ResolvedCodeEdge> {
        self.in_by_symbol_name
            .get(name)
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// All edges (either endpoint) whose symbol is `name`, across all files.
    pub fn edges_for_symbol_name(&self, name: &str) -> Vec<&ResolvedCodeEdge> {
        self.by_symbol_name
            .get(name)
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Edges of a given type (used for `wm_code.deps` edge filters).
    pub fn edges_of_type(&self, edge_type: &str) -> Vec<&ResolvedCodeEdge> {
        self.edges
            .iter()
            .filter(|e| e.edge_type == edge_type)
            .collect()
    }
}

/// A reference to a code node as passed by a user (CLI/MCP `node`/`id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeNodeRef {
    File(String),
    Symbol { file: String, symbol: String },
    SymbolName(String),
}

impl CodeNodeRef {
    /// Parse a node id against the known indexed files.
    pub fn parse(id: &str, graph: &CodeEdgeGraph) -> CodeNodeRef {
        if let Some((file, symbol)) = id.split_once('#') {
            if !file.is_empty() && !symbol.is_empty() {
                return CodeNodeRef::Symbol {
                    file: file.to_string(),
                    symbol: symbol.to_string(),
                };
            }
        }
        if graph.has_file(id) {
            return CodeNodeRef::File(id.to_string());
        }
        CodeNodeRef::SymbolName(id.to_string())
    }

    pub fn node_id(&self) -> String {
        match self {
            CodeNodeRef::File(f) => f.clone(),
            CodeNodeRef::Symbol { file, symbol } => format!("{}#{}", file, symbol),
            CodeNodeRef::SymbolName(n) => n.clone(),
        }
    }

    pub fn title(&self) -> String {
        match self {
            CodeNodeRef::File(f) => f.rsplit('/').next().unwrap_or(f).to_string(),
            CodeNodeRef::Symbol { symbol, .. } => symbol.clone(),
            CodeNodeRef::SymbolName(n) => n.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::code_edge_model::CodeEdge;

    fn sym(file: &str, name: &str, kind: &str, line: usize) -> CodeIntelSymbol {
        CodeIntelSymbol {
            file: file.to_string(),
            name: name.to_string(),
            kind: kind.to_string(),
            line,
            column: 0,
            snippet: String::new(),
            language: "rust".to_string(),
        }
    }

    fn raw(
        edge_type: &str,
        source_file: &str,
        source_symbol: Option<&str>,
        target_symbol: Option<&str>,
        line: usize,
    ) -> CodeEdge {
        CodeEdge {
            edge_type: edge_type.to_string(),
            source_file: source_file.to_string(),
            source_symbol: source_symbol.map(|s| s.to_string()),
            target_file: String::new(),
            target_symbol: target_symbol.map(|s| s.to_string()),
            line,
            provenance: EdgeProvenance::Explicit,
        }
    }

    #[test]
    fn resolves_cross_file_call_explicit() {
        let snapshot = CodeIndexSnapshot {
            symbols: vec![
                sym("src/lib.rs", "helper", "function", 1),
                sym("src/main.rs", "caller", "function", 1),
            ],
            raw_edges: vec![raw(
                "calls",
                "src/main.rs",
                Some("caller"),
                Some("helper"),
                4,
            )],
            files: ["src/lib.rs".into(), "src/main.rs".into()]
                .into_iter()
                .collect(),
        };
        let resolved = resolve_code_edges(&snapshot);
        assert_eq!(resolved.len(), 1, "call to a known symbol resolves");
        let e = &resolved[0];
        assert_eq!(e.edge_type, "calls");
        assert_eq!(e.source_file, "src/main.rs");
        assert_eq!(e.source_symbol.as_deref(), Some("caller"));
        assert_eq!(e.target_file, "src/lib.rs");
        assert_eq!(e.target_symbol.as_deref(), Some("helper"));
        assert_eq!(e.line, 4);
        assert_eq!(e.provenance, EdgeProvenance::Explicit);
    }

    #[test]
    fn drops_call_to_unknown_symbol() {
        let snapshot = CodeIndexSnapshot {
            symbols: vec![sym("src/lib.rs", "helper", "function", 1)],
            raw_edges: vec![raw(
                "calls",
                "src/main.rs",
                Some("caller"),
                Some("println"),
                4,
            )],
            files: ["src/lib.rs".into(), "src/main.rs".into()]
                .into_iter()
                .collect(),
        };
        let resolved = resolve_code_edges(&snapshot);
        assert!(resolved.is_empty(), "unknown callee edges are dropped");
    }

    #[test]
    fn ambiguous_call_when_symbol_defined_in_two_files() {
        let snapshot = CodeIndexSnapshot {
            symbols: vec![
                sym("src/a.rs", "run", "function", 1),
                sym("src/b.rs", "run", "function", 1),
            ],
            raw_edges: vec![raw("calls", "src/main.rs", Some("caller"), Some("run"), 4)],
            files: ["src/a.rs".into(), "src/b.rs".into(), "src/main.rs".into()]
                .into_iter()
                .collect(),
        };
        let resolved = resolve_code_edges(&snapshot);
        assert_eq!(resolved.len(), 1);
        let e = &resolved[0];
        assert_eq!(e.provenance, EdgeProvenance::Ambiguous);
        assert!(e.target_file == "src/a.rs" || e.target_file == "src/b.rs");
    }

    #[test]
    fn rust_import_resolves_with_src_prefix() {
        let snapshot = CodeIndexSnapshot {
            symbols: vec![
                sym("src/foo.rs", "Bar", "struct", 1),
                sym("src/main.rs", "main", "function", 1),
            ],
            raw_edges: vec![raw(
                "imports",
                "src/main.rs",
                None,
                Some("crate::foo::Bar"),
                3,
            )],
            files: ["src/foo.rs".into(), "src/main.rs".into()]
                .into_iter()
                .collect(),
        };
        let resolved = resolve_code_edges(&snapshot);
        assert_eq!(resolved.len(), 1, "crate import with src/ prefix resolves");
        let e = &resolved[0];
        assert_eq!(e.target_file, "src/foo.rs");
        assert_eq!(e.provenance, EdgeProvenance::Explicit);
    }

    #[test]
    fn ts_relative_import_resolves() {
        let snapshot = CodeIndexSnapshot {
            symbols: vec![
                sym("src/utils.ts", "helper", "function", 1),
                sym("src/main.ts", "main", "function", 1),
            ],
            raw_edges: vec![raw("imports", "src/main.ts", None, Some("./utils"), 2)],
            files: ["src/utils.ts".into(), "src/main.ts".into()]
                .into_iter()
                .collect(),
        };
        let resolved = resolve_code_edges(&snapshot);
        assert_eq!(resolved.len(), 1);
        let e = &resolved[0];
        assert_eq!(e.target_file, "src/utils.ts");
        assert_eq!(e.provenance, EdgeProvenance::Explicit);
    }

    #[test]
    fn import_through_reexport_is_derived() {
        // src/main.rs: use crate::foo::Bar
        // src/foo.rs:  pub use crate::bar::Bar;   (re-export)
        // src/bar.rs:  pub struct Bar;
        let snapshot = CodeIndexSnapshot {
            symbols: vec![
                sym("src/bar.rs", "Bar", "struct", 1),
                sym("src/main.rs", "main", "function", 1),
            ],
            raw_edges: vec![
                raw("imports", "src/main.rs", None, Some("crate::foo::Bar"), 2),
                raw("imports", "src/foo.rs", None, Some("crate::bar::Bar"), 1),
            ],
            files: [
                "src/bar.rs".into(),
                "src/foo.rs".into(),
                "src/main.rs".into(),
            ]
            .into_iter()
            .collect(),
        };
        let resolved = resolve_code_edges(&snapshot);
        let imp = resolved
            .iter()
            .find(|e| e.source_file == "src/main.rs")
            .expect("main.rs import resolves");
        assert_eq!(imp.target_file, "src/bar.rs", "chased to the defining file");
        assert_eq!(imp.provenance, EdgeProvenance::Derived);
        assert_eq!(imp.via, vec!["src/foo.rs".to_string()]);
    }

    #[test]
    fn ambiguous_import_when_two_files_match() {
        // Bare specifier `import { x } from 'a'` where both src/a.ts and
        // lib/a.ts exist as candidate files.
        let snapshot = CodeIndexSnapshot {
            symbols: vec![
                sym("src/a.ts", "x", "function", 1),
                sym("lib/a.ts", "x", "function", 1),
            ],
            raw_edges: vec![raw("imports", "src/main.ts", None, Some("a"), 2)],
            files: ["src/a.ts".into(), "lib/a.ts".into(), "src/main.ts".into()]
                .into_iter()
                .collect(),
        };
        let resolved = resolve_code_edges(&snapshot);
        assert_eq!(resolved.len(), 1);
        let e = &resolved[0];
        assert_eq!(e.provenance, EdgeProvenance::Ambiguous);
    }
}
