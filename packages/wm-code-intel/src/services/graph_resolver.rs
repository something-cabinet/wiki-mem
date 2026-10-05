
use std::collections::{HashMap, HashSet};

use crate::models::code_edge_model::CodeEdge;
use crate::models::language_model::SupportedLanguage;
use crate::models::symbol_model::CodeIntelSymbol;
use wm_engine::models::edge_type_model::EdgeProvenance;

use super::code_index_db::CodeIndexDb;
use super::engine_service::resolve_import_candidates;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ResolvedCodeEdge {
    pub edge_type: String,
    pub source_file: String,
    pub source_symbol: Option<String>,
    pub target_file: String,
    pub target_symbol: Option<String>,
    pub line: usize,
    pub provenance: EdgeProvenance,
    pub via: Vec<String>,
}

impl ResolvedCodeEdge {
    pub fn source_node_id(&self) -> String {
        match &self.source_symbol {
            Some(s) => format!("{}#{}", self.source_file, s),
            None => self.source_file.clone(),
        }
    }

    pub fn target_node_id(&self) -> String {
        match &self.target_symbol {
            Some(s) if !self.target_file.is_empty() => format!("{}#{}", self.target_file, s),
            _ => self.target_file.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CodeIndexSnapshot {
    pub symbols: Vec<CodeIntelSymbol>,
    pub raw_edges: Vec<CodeEdge>,
    pub files: HashSet<String>,
    pub ts_context: Option<super::ts_config_resolver::TsResolutionContext>,
}

impl CodeIndexSnapshot {
    pub fn from_db(db: &CodeIndexDb) -> Result<Self, String> {
        use super::code_index_db::EdgeQuery;
        let symbols = db.query_symbols(None, None, None, None, None, None)?;
        let raw_edges = db.query_edges(&EdgeQuery::default())?;
        let files: HashSet<String> = db.list_files()?.into_iter().collect();
        Ok(Self {
            symbols,
            raw_edges,
            files,
            ts_context: None,
        })
    }

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
        snapshot.ts_context = Some(super::ts_config_resolver::TsResolutionContext::discover(project_root));
        Ok(snapshot)
    }
}

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
    let all_raw_refs: Vec<&CodeEdge> = snapshot.raw_edges.iter().collect();
    let mut resolved = Vec::new();
    for raw in &snapshot.raw_edges {
        let r = match raw.edge_type.as_str() {
            "imports" | "imports_deferred" => resolve_import(raw, files, &by_file, &by_name, &edges_by_file, &snapshot.ts_context),
            "calls" | "inherits" | "implements" | "references" => resolve_symbol_edge(raw, &by_name, &all_raw_refs),
            _ => None,
        };
        if let Some(r) = r {
            resolved.push(r);
        }
    }
    resolved
}

fn resolve_symbol_edge(
    raw: &CodeEdge,
    by_name: &HashMap<&str, Vec<&CodeIntelSymbol>>,
    all_edges: &[&CodeEdge],
) -> Option<ResolvedCodeEdge> {
    let callee = raw.target_symbol.as_deref()?;
    let candidates = by_name.get(callee)?;
    if candidates.is_empty() {
        return None;
    }

    let mut defining_files: Vec<&str> = candidates.iter().map(|s| s.file.as_str()).collect();
    defining_files.sort_unstable();
    defining_files.dedup();

    let receiver = raw.receiver.as_deref();

    let narrowed = match receiver {
        None => defining_files.clone(),
        Some("self") | Some("this") | Some("Self") | Some("&self") => {
            if let Some(ref enclosing) = raw.source_symbol {
                let type_files: Vec<&str> = by_name
                    .get(enclosing.as_str())
                    .map(|syms| syms.iter().map(|s| s.file.as_str()).collect())
                    .unwrap_or_default();
                if type_files.is_empty() {
                    defining_files.clone()
                } else {
                    let filtered: Vec<&str> = defining_files
                        .iter()
                        .filter(|f| type_files.contains(f))
                        .copied()
                        .collect();
                    if filtered.is_empty() {
                        defining_files.clone()
                    } else {
                        filtered
                    }
                }
            } else {
                defining_files.clone()
            }
        }
        Some(recv) => {
            if let Some(type_syms) = by_name.get(recv) {
                let type_files: Vec<&str> = type_syms.iter().map(|s| s.file.as_str()).collect();
                let filtered: Vec<&str> = defining_files
                    .iter()
                    .filter(|f| type_files.contains(f))
                    .copied()
                    .collect();
                if !filtered.is_empty() {
                    filtered
                } else {
                    infer_from_constructor(recv, raw, all_edges, by_name, &defining_files)
                }
            } else {
                infer_from_constructor(recv, raw, all_edges, by_name, &defining_files)
            }
        }
    };

    if narrowed.is_empty() {
        return None;
    }

    if narrowed.len() > 1 {
        let nearest = pick_nearest(&raw.source_file, &narrowed);
        return Some(ResolvedCodeEdge {
            edge_type: raw.edge_type.clone(),
            source_file: raw.source_file.clone(),
            source_symbol: raw.source_symbol.clone(),
            target_file: nearest.to_string(),
            target_symbol: raw.target_symbol.clone(),
            line: raw.line,
            provenance: EdgeProvenance::Explicit,
            via: Vec::new(),
        });
    }

    Some(ResolvedCodeEdge {
        edge_type: raw.edge_type.clone(),
        source_file: raw.source_file.clone(),
        source_symbol: raw.source_symbol.clone(),
        target_file: narrowed[0].to_string(),
        target_symbol: raw.target_symbol.clone(),
        line: raw.line,
        provenance: EdgeProvenance::Explicit,
        via: Vec::new(),
    })
}

fn pick_nearest<'a>(source: &str, candidates: &[&'a str]) -> &'a str {
    fn path_distance(a: &str, b: &str) -> usize {
        let a_parts: Vec<&str> = a.split('/').collect();
        let b_parts: Vec<&str> = b.split('/').collect();
        let common = a_parts
            .iter()
            .zip(b_parts.iter())
            .take_while(|(x, y)| x == y)
            .count();
        (a_parts.len() - common) + (b_parts.len() - common)
    }
    candidates
        .iter()
        .copied()
        .min_by(|a, b| {
            let da = path_distance(source, a);
            let db = path_distance(source, b);
            da.cmp(&db)
                .then_with(|| a.len().cmp(&b.len()))
                .then_with(|| a.cmp(b))
        })
        .unwrap_or(candidates[0])
}

fn infer_from_constructor<'a>(
    _recv: &str,
    raw: &CodeEdge,
    all_edges: &[&CodeEdge],
    by_name: &HashMap<&str, Vec<&CodeIntelSymbol>>,
    defining_files: &[&'a str],
) -> Vec<&'a str> {
    let enclosing = raw.source_symbol.as_deref();

    let constructor_types: Vec<&str> = all_edges
        .iter()
        .filter(|e| {
            e.source_file == raw.source_file
                && e.edge_type == "calls"
                && e.target_symbol.as_deref() == Some("new")
                && e.source_symbol.as_deref() == enclosing
                && e.receiver.is_some()
        })
        .filter_map(|e| {
            let recv = e.receiver.as_deref()?;
            if by_name.contains_key(recv) {
                Some(recv)
            } else {
                None
            }
        })
        .collect();

    if constructor_types.len() == 1 {
        let type_name = constructor_types[0];
        if let Some(type_syms) = by_name.get(type_name) {
            let type_files: Vec<&str> = type_syms.iter().map(|s| s.file.as_str()).collect();
            let filtered: Vec<&str> = defining_files
                .iter()
                .filter(|f| type_files.contains(f))
                .copied()
                .collect();
            if !filtered.is_empty() {
                return filtered;
            }
        }
    }

    defining_files.to_vec()
}

fn resolve_import(
    raw: &CodeEdge,
    files: &HashSet<String>,
    by_file: &HashMap<&str, Vec<&CodeIntelSymbol>>,
    by_name: &HashMap<&str, Vec<&CodeIntelSymbol>>,
    edges_by_file: &HashMap<&str, Vec<&CodeEdge>>,
    ts_context: &Option<super::ts_config_resolver::TsResolutionContext>,
) -> Option<ResolvedCodeEdge> {
    let target = raw.target_symbol.as_deref()?;
    let lang = lang_from_file(&raw.source_file)?;

    if matches!(lang, SupportedLanguage::TypeScript | SupportedLanguage::Tsx) {
        if let Some(ctx) = ts_context {
            let is_alias_candidate = !target.starts_with('.');
            if is_alias_candidate {
                if let Some(candidates) = ctx.resolve_specifier(&raw.source_file, target) {
                    let mut ts_matches: Vec<String> = Vec::new();
                    for c in &candidates {
                        for f in files {
                            if (f == c || f.ends_with(&format!("/{}", c))) && !ts_matches.contains(f) {
                                ts_matches.push(f.clone());
                            }
                        }
                    }
                    ts_matches.sort();
                    if !ts_matches.is_empty() {
                        let target_file = pick_nearest(&raw.source_file, &ts_matches.iter().map(|s| s.as_str()).collect::<Vec<_>>()).to_string();
                        return Some(ResolvedCodeEdge {
                            edge_type: raw.edge_type.clone(),
                            source_file: raw.source_file.clone(),
                            source_symbol: raw.source_symbol.clone(),
                            target_file,
                            target_symbol: raw.target_symbol.clone(),
                            line: raw.line,
                            provenance: EdgeProvenance::Explicit,
                            via: Vec::new(),
                        });
                    }
                }
            }
        }
    }

    let most_specific_first_prefixes: Vec<String> = match lang {
        SupportedLanguage::Rust => rust_import_prefixes(target),
        _ => vec![target.to_string()],
    };

    let mut matches: Vec<String> = Vec::new();
    let mut matched_prefix: Option<String> = None;
    for t in &most_specific_first_prefixes {
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

    let trailing_symbol: Option<String> = if let Some(prefix) = &matched_prefix {
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

    if let Some(trailing_symbol) = trailing_symbol {
        let defines_directly = by_file
            .get(matches[0].as_str())
            .map(|syms| syms.iter().any(|s| s.name == trailing_symbol))
            .unwrap_or(false);
        if !defines_directly {
            if let Some(target) = chase_reexport(
                &matches[0],
                &trailing_symbol,
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
        target_file = pick_nearest(&raw.source_file, &matches.iter().map(|s| s.as_str()).collect::<Vec<_>>()).to_string();
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

fn lang_from_file(file: &str) -> Option<SupportedLanguage> {
    let ext = file.rsplit('.').next()?;
    SupportedLanguage::from_ext(ext)
}

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

    pub fn outgoing_from_symbol(&self, file: &str, symbol: &str) -> Vec<&ResolvedCodeEdge> {
        self.out_by_symbol
            .get(&(file.to_string(), symbol.to_string()))
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    pub fn incoming_to_symbol(&self, file: &str, symbol: &str) -> Vec<&ResolvedCodeEdge> {
        self.in_by_symbol
            .get(&(file.to_string(), symbol.to_string()))
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    pub fn outgoing_from_file(&self, file: &str) -> Vec<&ResolvedCodeEdge> {
        self.out_by_file
            .get(file)
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    pub fn incoming_to_file(&self, file: &str) -> Vec<&ResolvedCodeEdge> {
        self.in_by_file
            .get(file)
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    pub fn incoming_to_symbol_name(&self, name: &str) -> Vec<&ResolvedCodeEdge> {
        self.in_by_symbol_name
            .get(name)
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    pub fn edges_for_symbol_name(&self, name: &str) -> Vec<&ResolvedCodeEdge> {
        self.by_symbol_name
            .get(name)
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    pub fn edges_of_type(&self, edge_type: &str) -> Vec<&ResolvedCodeEdge> {
        self.edges
            .iter()
            .filter(|e| e.edge_type == edge_type)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeNodeRef {
    File(String),
    Symbol { file: String, symbol: String },
    SymbolName(String),
}

impl CodeNodeRef {
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

pub fn detect_import_cycles(edges: &[ResolvedCodeEdge]) -> Vec<Vec<String>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut all_files: HashSet<&str> = HashSet::new();

    for e in edges {
        if e.edge_type != "imports" {
            continue;
        }
        if e.target_file.is_empty() {
            continue;
        }
        all_files.insert(e.source_file.as_str());
        all_files.insert(e.target_file.as_str());
        adj.entry(e.source_file.as_str())
            .or_default()
            .push(e.target_file.as_str());
    }

    let mut state = TarjanState {
        index_counter: 0,
        stack: Vec::new(),
        on_stack: HashSet::new(),
        indices: HashMap::new(),
        lowlinks: HashMap::new(),
        cycles: Vec::new(),
    };

    struct TarjanState<'a> {
        index_counter: usize,
        stack: Vec<&'a str>,
        on_stack: HashSet<&'a str>,
        indices: HashMap<&'a str, usize>,
        lowlinks: HashMap<&'a str, usize>,
        cycles: Vec<Vec<String>>,
    }

    impl<'a> TarjanState<'a> {
        fn strongconnect(&mut self, node: &'a str, adj: &HashMap<&'a str, Vec<&'a str>>) {
            self.indices.insert(node, self.index_counter);
            self.lowlinks.insert(node, self.index_counter);
            self.index_counter += 1;
            self.stack.push(node);
            self.on_stack.insert(node);

            if let Some(neighbors) = adj.get(node) {
                for &neighbor in neighbors {
                    if !self.indices.contains_key(neighbor) {
                        self.strongconnect(neighbor, adj);
                        let nl = *self.lowlinks.get(neighbor).unwrap_or(&0);
                        let entry = self.lowlinks.entry(node).or_insert(0);
                        if nl < *entry {
                            *entry = nl;
                        }
                    } else if self.on_stack.contains(neighbor) {
                        let ni = *self.indices.get(neighbor).unwrap_or(&0);
                        let entry = self.lowlinks.entry(node).or_insert(0);
                        if ni < *entry {
                            *entry = ni;
                        }
                    }
                }
            }

            if self.lowlinks.get(node) == self.indices.get(node) {
                let mut component: Vec<String> = Vec::new();
                while let Some(w) = self.stack.pop() {
                    self.on_stack.remove(w);
                    component.push(w.to_string());
                    if w == node {
                        break;
                    }
                }
                if component.len() > 1 {
                    component.reverse();
                    let first = component[0].clone();
                    component.push(first);
                    self.cycles.push(component);
                }
            }
        }
    }

    let mut sorted_files: Vec<&str> = all_files.into_iter().collect();
    sorted_files.sort();

    for &file in &sorted_files {
        if !state.indices.contains_key(file) {
            state.strongconnect(file, &adj);
        }
    }

    state.cycles.sort();
    state.cycles
}

pub fn is_deferred_import(edge: &ResolvedCodeEdge) -> bool {
    edge.edge_type == "imports_deferred"
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
            receiver: None,
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
                    ts_context: None,
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
                    ts_context: None,
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
                    ts_context: None,
                };
        let resolved = resolve_code_edges(&snapshot);
        assert_eq!(resolved.len(), 1);
        let e = &resolved[0];
        assert_eq!(e.provenance, EdgeProvenance::Explicit);
        assert_eq!(
            e.target_file, "src/a.rs",
            "path-distance picks src/a.rs (nearer to src/main.rs than src/b.rs — same distance, shorter path)"
        );
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
                    ts_context: None,
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
                    ts_context: None,
                };
        let resolved = resolve_code_edges(&snapshot);
        assert_eq!(resolved.len(), 1);
        let e = &resolved[0];
        assert_eq!(e.target_file, "src/utils.ts");
        assert_eq!(e.provenance, EdgeProvenance::Explicit);
    }

    #[test]
    fn import_through_reexport_is_derived() {
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
                    ts_context: None,
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
    fn ambiguous_import_picks_path_nearest() {
        let snapshot = CodeIndexSnapshot {
                    symbols: vec![
                        sym("src/a.ts", "x", "function", 1),
                        sym("lib/a.ts", "x", "function", 1),
                    ],
                    raw_edges: vec![raw("imports", "src/main.ts", None, Some("a"), 2)],
                    files: ["src/a.ts".into(), "lib/a.ts".into(), "src/main.ts".into()]
                        .into_iter()
                        .collect(),
                    ts_context: None,
                };
        let resolved = resolve_code_edges(&snapshot);
        assert_eq!(resolved.len(), 1);
        let e = &resolved[0];
        assert_eq!(e.provenance, EdgeProvenance::Explicit);
        assert_eq!(
            e.target_file, "src/a.ts",
            "path-distance picks src/a.ts (same dir as src/main.ts)"
        );
    }
}
