use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::models::code_edge_model::CodeEdge;
use crate::models::dep_model::CodeIntelDep;
use crate::models::symbol_model::CodeIntelSymbol;
use crate::services::graph_resolver::ResolvedCodeEdge;
use turso::params_from_iter;
use turso::Value;
use wm_engine::models::edge_type_model::EdgeProvenance;

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

const UPSERT_FILE_SQL: &str = "INSERT INTO code_files (path, sha256, mtime, language)
     VALUES (?1, ?2, ?3, ?4)
     ON CONFLICT(path) DO UPDATE SET
         sha256 = excluded.sha256,
         mtime = excluded.mtime,
         language = excluded.language";
const DELETE_SYMBOLS_SQL: &str = "DELETE FROM code_symbols WHERE file = ?1";
const INSERT_SYMBOL_SQL: &str =
    "INSERT INTO code_symbols (file, name, kind, line, column, snippet, language)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";
const DELETE_DEPS_SQL: &str = "DELETE FROM code_deps WHERE file = ?1";
const INSERT_DEP_SQL: &str = "INSERT INTO code_deps (file, target, line, kind, language)
     VALUES (?1, ?2, ?3, ?4, ?5)";
const DELETE_EDGES_SQL: &str = "DELETE FROM code_edges WHERE source_file = ?1";
const INSERT_EDGE_SQL: &str = "INSERT INTO code_edges
     (source_file, source_symbol, edge_type, target_file, target_symbol, receiver, line, provenance, language)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";
const DELETE_RESOLVED_EDGES_SQL: &str = "DELETE FROM resolved_edges";
const INSERT_RESOLVED_EDGE_SQL: &str = "INSERT INTO resolved_edges
     (source_file, source_symbol, edge_type, target_file, target_symbol, line, provenance, via)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";
const DELETE_FILE_SQL: &str = "DELETE FROM code_files WHERE path = ?1";

type SymbolParams<'a> = (&'a str, &'a str, &'a str, i64, i64, &'a str, &'a str);
type DepParams<'a> = (&'a str, &'a str, i64, &'a str, &'a str);
type EdgeParams<'a> = (
    &'a str,
    Option<&'a str>,
    &'a str,
    &'a str,
    Option<&'a str>,
    Option<&'a str>,
    i64,
    &'a str,
    &'a str,
);
type ResolvedParams<'a> = (
    &'a str,
    Option<&'a str>,
    &'a str,
    &'a str,
    Option<&'a str>,
    i64,
    &'a str,
    &'a str,
);

#[derive(Debug, Clone)]
pub struct FileData {
    pub path: String,
    pub sha256: String,
    pub mtime: i64,
    pub language: String,
    pub symbols: Vec<CodeIntelSymbol>,
    pub deps: Vec<CodeIntelDep>,
    pub edges: Vec<CodeEdge>,
}

struct InnerDb {
    conn: turso::Connection,
}

pub struct CodeIndexDb {
    db: Arc<Mutex<InnerDb>>,
}

async fn open_db(path: &str) -> Result<turso::Connection, String> {
    let db = turso::Builder::new_local(path)
        .build()
        .await
        .map_err(|e| e.to_string())?;
    let conn = db.connect().map_err(|e| e.to_string())?;
    conn.busy_timeout(BUSY_TIMEOUT).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS code_files (
            path TEXT PRIMARY KEY,
            sha256 TEXT NOT NULL,
            mtime INTEGER NOT NULL,
            language TEXT NOT NULL
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS code_symbols (
            file TEXT NOT NULL, name TEXT NOT NULL, kind TEXT NOT NULL,
            line INTEGER NOT NULL, column INTEGER NOT NULL,
            snippet TEXT NOT NULL, language TEXT NOT NULL
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_symbols_name ON code_symbols(name)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_symbols_kind ON code_symbols(kind)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS code_deps (
            file TEXT NOT NULL, target TEXT NOT NULL,
            line INTEGER NOT NULL, kind TEXT NOT NULL,
            language TEXT NOT NULL DEFAULT ''
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_deps_target ON code_deps(target)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_symbols_file ON code_symbols(file)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_deps_file ON code_deps(file)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_deps_lang ON code_deps(language)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS code_edges (
            source_file TEXT NOT NULL,
            source_symbol TEXT,
            edge_type TEXT NOT NULL,
            target_file TEXT NOT NULL,
            target_symbol TEXT,
            receiver TEXT,
            line INTEGER NOT NULL,
            provenance TEXT NOT NULL DEFAULT 'explicit',
            language TEXT NOT NULL DEFAULT ''
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute("ALTER TABLE code_edges ADD COLUMN receiver TEXT", ())
        .await
        .ok();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_edges_source ON code_edges(source_file)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_edges_target ON code_edges(target_file)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_edges_type ON code_edges(edge_type)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS resolved_edges (
            source_file TEXT NOT NULL,
            source_symbol TEXT,
            edge_type TEXT NOT NULL,
            target_file TEXT NOT NULL,
            target_symbol TEXT,
            line INTEGER NOT NULL,
            provenance TEXT NOT NULL DEFAULT 'explicit',
            via TEXT NOT NULL DEFAULT ''
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resolved_source ON resolved_edges(source_file)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resolved_target ON resolved_edges(target_file)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resolved_type ON resolved_edges(edge_type)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(conn)
}

async fn load_file_hashes_impl(
    conn: &turso::Connection,
) -> Result<HashMap<String, (String, i64)>, String> {
    let mut result = HashMap::new();
    let mut rows = conn
        .query("SELECT path, sha256, mtime FROM code_files", ())
        .await
        .map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let path: String = row.get(0).map_err(|e| e.to_string())?;
        let sha256: String = row.get(1).map_err(|e| e.to_string())?;
        let mtime: i64 = row.get(2).map_err(|e| e.to_string())?;
        result.insert(path, (sha256, mtime));
    }
    Ok(result)
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn to_i64(value: usize, what: &str) -> Result<i64, String> {
    value
        .try_into()
        .map_err(|_| format!("{} overflow: {}", what, value))
}

fn symbol_params<'a>(
    file_path: &'a str,
    symbol: &'a CodeIntelSymbol,
) -> Result<SymbolParams<'a>, String> {
    let line = to_i64(symbol.line, "symbol line")?;
    let column = to_i64(symbol.column, "symbol column")?;
    Ok((
        file_path,
        symbol.name.as_str(),
        symbol.kind.as_str(),
        line,
        column,
        symbol.snippet.as_str(),
        symbol.language.as_str(),
    ))
}

fn dep_params<'a>(file: &'a FileData, dep: &'a CodeIntelDep) -> Result<DepParams<'a>, String> {
    let line = to_i64(dep.line, "dep line")?;
    Ok((
        file.path.as_str(),
        dep.target.as_str(),
        line,
        dep.kind.as_str(),
        file.language.as_str(),
    ))
}

fn edge_params<'a>(file: &'a FileData, edge: &'a CodeEdge) -> Result<EdgeParams<'a>, String> {
    let line = to_i64(edge.line, "edge line")?;
    Ok((
        file.path.as_str(),
        edge.source_symbol.as_deref(),
        edge.edge_type.as_str(),
        edge.target_file.as_str(),
        edge.target_symbol.as_deref(),
        edge.receiver.as_deref(),
        line,
        edge.provenance.as_str(),
        file.language.as_str(),
    ))
}

fn resolved_params<'a>(
    edge: &'a ResolvedCodeEdge,
    via: &'a str,
) -> Result<ResolvedParams<'a>, String> {
    let line = to_i64(edge.line, "resolved edge line")?;
    Ok((
        edge.source_file.as_str(),
        edge.source_symbol.as_deref(),
        edge.edge_type.as_str(),
        edge.target_file.as_str(),
        edge.target_symbol.as_deref(),
        line,
        edge.provenance.as_str(),
        via,
    ))
}

async fn insert_symbol(
    conn: &turso::Connection,
    file_path: &str,
    symbol: &CodeIntelSymbol,
) -> Result<(), String> {
    conn.execute(INSERT_SYMBOL_SQL, symbol_params(file_path, symbol)?)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn insert_dep(
    conn: &turso::Connection,
    file: &FileData,
    dep: &CodeIntelDep,
) -> Result<(), String> {
    conn.execute(INSERT_DEP_SQL, dep_params(file, dep)?)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn insert_edge(
    conn: &turso::Connection,
    file: &FileData,
    edge: &CodeEdge,
) -> Result<(), String> {
    conn.execute(INSERT_EDGE_SQL, edge_params(file, edge)?)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn replace_file_symbols(conn: &turso::Connection, file: &FileData) -> Result<(), String> {
    conn.execute(DELETE_SYMBOLS_SQL, [file.path.as_str()])
        .await
        .map_err(|e| e.to_string())?;
    for symbol in &file.symbols {
        insert_symbol(conn, &file.path, symbol).await?;
    }
    Ok(())
}

async fn replace_file_deps(conn: &turso::Connection, file: &FileData) -> Result<(), String> {
    conn.execute(DELETE_DEPS_SQL, [file.path.as_str()])
        .await
        .map_err(|e| e.to_string())?;
    for dep in &file.deps {
        insert_dep(conn, file, dep).await?;
    }
    Ok(())
}

async fn replace_file_edges(conn: &turso::Connection, file: &FileData) -> Result<(), String> {
    conn.execute(DELETE_EDGES_SQL, [file.path.as_str()])
        .await
        .map_err(|e| e.to_string())?;
    for edge in &file.edges {
        insert_edge(conn, file, edge).await?;
    }
    Ok(())
}

async fn upsert_file(conn: &turso::Connection, file: &FileData) -> Result<(), String> {
    conn.execute(
        UPSERT_FILE_SQL,
        (
            file.path.as_str(),
            file.sha256.as_str(),
            file.mtime,
            file.language.as_str(),
        ),
    )
    .await
    .map_err(|e| e.to_string())?;
    replace_file_symbols(conn, file).await?;
    replace_file_deps(conn, file).await?;
    replace_file_edges(conn, file).await
}

async fn upsert_files_impl(conn: &turso::Connection, files: &[FileData]) -> Result<(), String> {
    for file in files {
        upsert_file(conn, file).await?;
    }
    Ok(())
}

async fn query_symbols_impl(
    conn: &turso::Connection,
    name: Option<&str>,
    kind: Option<&str>,
    file: Option<&str>,
    path: Option<&str>,
    language: Option<&str>,
    max_results: Option<usize>,
) -> Result<Vec<CodeIntelSymbol>, String> {
    let mut sql =
        "SELECT file, name, kind, line, column, snippet, language FROM code_symbols WHERE 1=1"
            .to_string();
    let mut params: Vec<Value> = Vec::new();

    if name.is_some() {
        sql.push_str(" AND name LIKE '%' || ? || '%' ESCAPE '\\'");
    }
    if kind.is_some() {
        sql.push_str(" AND kind = ?");
    }
    if file.is_some() {
        sql.push_str(" AND (file = ? OR file LIKE '%/' || ?)");
    }
    if path.is_some() {
        sql.push_str(" AND file LIKE ? || '/%' ESCAPE '\\'");
    }
    if language.is_some() {
        sql.push_str(" AND language = ?");
    }
    if let Some(limit) = max_results {
        sql.push_str(&format!(" LIMIT {}", limit));
    }

    if let Some(n) = name {
        params.push(Value::from(escape_like(n)));
    }
    if let Some(k) = kind {
        params.push(Value::from(k.to_string()));
    }
    if let Some(f) = file {
        let escaped = escape_like(f);
        params.push(Value::from(escaped.clone()));
        params.push(Value::from(escaped));
    }
    if let Some(p) = path {
        params.push(Value::from(escape_like(p)));
    }
    if let Some(l) = language {
        params.push(Value::from(l.to_string()));
    }

    let mut rows = conn
        .query(&sql, params_from_iter(params))
        .await
        .map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let file: String = row.get(0).map_err(|e| e.to_string())?;
        let name: String = row.get(1).map_err(|e| e.to_string())?;
        let kind: String = row.get(2).map_err(|e| e.to_string())?;
        let line: i64 = row.get(3).map_err(|e| e.to_string())?;
        let column: i64 = row.get(4).map_err(|e| e.to_string())?;
        let snippet: String = row.get(5).map_err(|e| e.to_string())?;
        let language: String = row.get(6).map_err(|e| e.to_string())?;
        results.push(CodeIntelSymbol {
            file,
            name,
            kind,
            line: usize::try_from(line).map_err(|_| format!("negative line value: {}", line))?,
            column: usize::try_from(column)
                .map_err(|_| format!("negative column value: {}", column))?,
            snippet,
            language,
        });
    }
    Ok(results)
}

async fn query_deps_impl(
    conn: &turso::Connection,
    file: Option<&str>,
    target: Option<&str>,
    reverse: bool,
    depth: Option<usize>,
    language: Option<&str>,
) -> Result<Vec<serde_json::Value>, String> {
    let depth = depth.unwrap_or(1);
    if depth > 1 {
        return Err("Recursive dependency depth not yet supported (depth=1 only)".into());
    }

    let mut sql = String::new();
    let mut params: Vec<Value> = Vec::new();

    if reverse {
        let t = match target {
            Some(t) => t,
            None => return Ok(Vec::new()),
        };
        sql.push_str(
            "SELECT DISTINCT file FROM code_deps WHERE target LIKE '%' || ? || '%' ESCAPE '\\'",
        );
        params.push(Value::from(escape_like(t)));

        if let Some(l) = language {
            sql.push_str(" AND language = ?");
            params.push(Value::from(l.to_string()));
        }

        let mut rows = conn
            .query(&sql, params_from_iter(params))
            .await
            .map_err(|e| e.to_string())?;
        let mut results: Vec<serde_json::Value> = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            let _f: String = row.get(0).map_err(|e| e.to_string())?;
            results.push(serde_json::json!({"file": _f}));
        }
        return Ok(results);
    }

    let file_filter = match file {
        Some(f) => f,
        None => return Ok(Vec::new()),
    };
    sql.push_str(
        "SELECT file, target, line, kind FROM code_deps WHERE (file = ? OR file LIKE '%/' || ?)",
    );
    let escaped_file = escape_like(file_filter);
    params.push(Value::from(escaped_file.clone()));
    params.push(Value::from(escaped_file));

    if let Some(t) = target {
        sql.push_str(" AND target LIKE '%' || ? || '%' ESCAPE '\\'");
        params.push(Value::from(escape_like(t)));
    }

    if let Some(l) = language {
        sql.push_str(" AND language = ?");
        params.push(Value::from(l.to_string()));
    }

    let mut rows = conn
        .query(&sql, params_from_iter(params))
        .await
        .map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let f: String = row.get(0).map_err(|e| e.to_string())?;
        let t: String = row.get(1).map_err(|e| e.to_string())?;
        let line: i64 = row.get(2).map_err(|e| e.to_string())?;
        let kind: String = row.get(3).map_err(|e| e.to_string())?;
        results.push(serde_json::json!({
            "file": f,
            "target": t,
            "line": line,
            "kind": kind,
        }));
    }
    Ok(results)
}

async fn get_file_count_and_max_mtime_impl(
    conn: &turso::Connection,
) -> Result<(usize, i64), String> {
    let mut rows = conn
        .query(
            "SELECT COUNT(*), COALESCE(MAX(mtime), 0) FROM code_files",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let count: i64 = row.get(0).map_err(|e| e.to_string())?;
        let mtime: i64 = row.get(1).map_err(|e| e.to_string())?;
        Ok((
            usize::try_from(count).map_err(|_| format!("negative count: {}", count))?,
            mtime,
        ))
    } else {
        Ok((0, 0))
    }
}

async fn count_symbols_impl(conn: &turso::Connection) -> Result<usize, String> {
    let mut rows = conn
        .query("SELECT COUNT(*) FROM code_symbols", ())
        .await
        .map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let count: i64 = row.get(0).map_err(|e| e.to_string())?;
        Ok(usize::try_from(count).map_err(|_| format!("negative count: {}", count))?)
    } else {
        Ok(0)
    }
}

async fn count_deps_impl(conn: &turso::Connection) -> Result<usize, String> {
    let mut rows = conn
        .query("SELECT COUNT(*) FROM code_deps", ())
        .await
        .map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let count: i64 = row.get(0).map_err(|e| e.to_string())?;
        Ok(usize::try_from(count).map_err(|_| format!("negative count: {}", count))?)
    } else {
        Ok(0)
    }
}

async fn count_edges_impl(conn: &turso::Connection) -> Result<usize, String> {
    let mut rows = conn
        .query("SELECT COUNT(*) FROM code_edges", ())
        .await
        .map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let count: i64 = row.get(0).map_err(|e| e.to_string())?;
        Ok(usize::try_from(count).map_err(|_| format!("negative count: {}", count))?)
    } else {
        Ok(0)
    }
}

async fn list_files_impl(conn: &turso::Connection) -> Result<Vec<String>, String> {
    let mut rows = conn
        .query("SELECT path FROM code_files", ())
        .await
        .map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let path: String = row.get(0).map_err(|e| e.to_string())?;
        paths.push(path);
    }
    Ok(paths)
}

#[derive(Debug, Clone, Default)]
pub struct EdgeQuery {
    pub source_file: Option<String>,
    pub source_symbol: Option<String>,
    pub edge_type: Option<String>,
    pub target_file: Option<String>,
    pub provenance: Option<String>,
    pub reverse: bool,
    pub language: Option<String>,
}

impl EdgeQuery {
    pub fn forward(source_file: Option<&str>) -> Self {
        Self {
            source_file: source_file.map(|s| s.to_string()),
            ..Default::default()
        }
    }
}

async fn query_edges_impl(
    conn: &turso::Connection,
    q: &EdgeQuery,
) -> Result<Vec<CodeEdge>, String> {
    let mut sql =
        "SELECT source_file, source_symbol, edge_type, target_file, target_symbol, line, provenance, receiver FROM code_edges WHERE 1=1"
            .to_string();
    let mut params: Vec<Value> = Vec::new();

    if q.reverse {
        if let Some(tf) = &q.target_file {
            sql.push_str(" AND target_file = ?");
            params.push(Value::from(tf.clone()));
        } else {
            return Ok(Vec::new());
        }
        if let Some(ts) = &q.source_symbol {
            sql.push_str(" AND target_symbol = ?");
            params.push(Value::from(ts.clone()));
        }
    } else {
        if let Some(sf) = &q.source_file {
            sql.push_str(" AND (source_file = ? OR source_file LIKE '%/' || ?)");
            let escaped = escape_like(sf);
            params.push(Value::from(escaped.clone()));
            params.push(Value::from(escaped));
        }
        if let Some(ss) = &q.source_symbol {
            sql.push_str(" AND source_symbol = ?");
            params.push(Value::from(ss.clone()));
        }
    }
    if let Some(et) = &q.edge_type {
        sql.push_str(" AND edge_type = ?");
        params.push(Value::from(et.clone()));
    }
    if let Some(p) = &q.provenance {
        sql.push_str(" AND provenance = ?");
        params.push(Value::from(p.clone()));
    }
    if let Some(l) = &q.language {
        sql.push_str(" AND language = ?");
        params.push(Value::from(l.clone()));
    }

    let mut rows = conn
        .query(&sql, params_from_iter(params))
        .await
        .map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let source_file: String = row.get(0).map_err(|e| e.to_string())?;
        let source_symbol: Option<String> = row.get(1).map_err(|e| e.to_string())?;
        let edge_type: String = row.get(2).map_err(|e| e.to_string())?;
        let target_file: String = row.get(3).map_err(|e| e.to_string())?;
        let target_symbol: Option<String> = row.get(4).map_err(|e| e.to_string())?;
        let line: i64 = row.get(5).map_err(|e| e.to_string())?;
        let provenance: String = row.get(6).map_err(|e| e.to_string())?;
        let receiver: Option<String> = row.get(7).ok();
        let provenance = match provenance.as_str() {
            "derived" => crate::models::code_edge_model::EdgeProvenance::Derived,
            "ambiguous" => crate::models::code_edge_model::EdgeProvenance::Ambiguous,
            _ => crate::models::code_edge_model::EdgeProvenance::Explicit,
        };
        results.push(CodeEdge {
            edge_type,
            source_file,
            source_symbol,
            target_file,
            target_symbol,
            receiver,
            line: usize::try_from(line).map_err(|_| format!("negative line value: {}", line))?,
            provenance,
        });
    }
    Ok(results)
}

async fn stale_paths_impl(
    conn: &turso::Connection,
    known_paths: &[String],
) -> Result<Vec<String>, String> {
    let known: HashSet<&str> = known_paths.iter().map(String::as_str).collect();
    let db_paths = list_files_impl(conn).await?;
    Ok(db_paths
        .into_iter()
        .filter(|path| !known.contains(path.as_str()))
        .collect())
}

async fn delete_file_rows(conn: &turso::Connection, path: &str) -> Result<(), String> {
    conn.execute(DELETE_FILE_SQL, [path])
        .await
        .map_err(|e| e.to_string())?;
    conn.execute(DELETE_SYMBOLS_SQL, [path])
        .await
        .map_err(|e| e.to_string())?;
    conn.execute(DELETE_DEPS_SQL, [path])
        .await
        .map_err(|e| e.to_string())?;
    conn.execute(DELETE_EDGES_SQL, [path])
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn delete_stale_files_impl(
    conn: &turso::Connection,
    known_paths: &[String],
) -> Result<usize, String> {
    let stale = stale_paths_impl(conn, known_paths).await?;
    for path in &stale {
        delete_file_rows(conn, path).await?;
    }
    Ok(stale.len())
}

async fn apply_base_write_impl(
    conn: &turso::Connection,
    files: &[FileData],
    known_paths: &[String],
) -> Result<(), String> {
    upsert_files_impl(conn, files).await?;
    delete_stale_files_impl(conn, known_paths).await?;
    conn.execute(DELETE_RESOLVED_EDGES_SQL, ())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn run_async<F, T>(f: F) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    match tokio::runtime::Handle::try_current() {
        Ok(_) => tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(f)),
        Err(_) => {
            let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
            rt.block_on(f)
        }
    }
}

async fn rollback_tx(conn: &turso::Connection) {
    let _ = conn.execute("ROLLBACK", ()).await;
}

async fn commit_tx<T>(conn: &turso::Connection, value: T) -> Result<T, String> {
    match conn.execute("COMMIT", ()).await {
        Ok(_) => Ok(value),
        Err(error) => {
            rollback_tx(conn).await;
            Err(error.to_string())
        }
    }
}

async fn with_write_tx<T, F, Fut>(conn: &turso::Connection, operation: F) -> Result<T, String>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    conn.execute("BEGIN IMMEDIATE", ())
        .await
        .map_err(|e| e.to_string())?;
    match operation().await {
        Ok(value) => commit_tx(conn, value).await,
        Err(error) => {
            rollback_tx(conn).await;
            Err(error)
        }
    }
}

async fn insert_resolved_edge(
    conn: &turso::Connection,
    edge: &ResolvedCodeEdge,
) -> Result<(), String> {
    let via = edge.via.join(",");
    conn.execute(INSERT_RESOLVED_EDGE_SQL, resolved_params(edge, &via)?)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn replace_resolved_edges_impl(
    conn: &turso::Connection,
    edges: &[ResolvedCodeEdge],
) -> Result<(), String> {
    conn.execute(DELETE_RESOLVED_EDGES_SQL, ())
        .await
        .map_err(|e| e.to_string())?;
    for edge in edges {
        insert_resolved_edge(conn, edge).await?;
    }
    Ok(())
}

async fn load_resolved_edges_impl(
    conn: &turso::Connection,
) -> Result<Vec<ResolvedCodeEdge>, String> {
    let mut rows = conn
        .query(
            "SELECT source_file, source_symbol, edge_type, target_file, target_symbol, line, provenance, via FROM resolved_edges",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let source_file: String = row.get(0).map_err(|e| e.to_string())?;
        let source_symbol: Option<String> = row.get(1).map_err(|e| e.to_string())?;
        let edge_type: String = row.get(2).map_err(|e| e.to_string())?;
        let target_file: String = row.get(3).map_err(|e| e.to_string())?;
        let target_symbol: Option<String> = row.get(4).map_err(|e| e.to_string())?;
        let line: i64 = row.get(5).map_err(|e| e.to_string())?;
        let provenance_str: String = row.get(6).map_err(|e| e.to_string())?;
        let via_str: String = row.get(7).map_err(|e| e.to_string())?;

        let provenance = match provenance_str.as_str() {
            "derived" => EdgeProvenance::Derived,
            "ambiguous" => EdgeProvenance::Ambiguous,
            _ => EdgeProvenance::Explicit,
        };
        let via: Vec<String> = via_str
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        results.push(ResolvedCodeEdge {
            edge_type,
            source_file,
            source_symbol,
            target_file,
            target_symbol,
            line: usize::try_from(line).map_err(|_| format!("negative line value: {}", line))?,
            provenance,
            via,
        });
    }
    Ok(results)
}

async fn has_resolved_edges_impl(conn: &turso::Connection) -> Result<bool, String> {
    let mut rows = conn
        .query("SELECT COUNT(*) FROM resolved_edges", ())
        .await
        .map_err(|e| e.to_string())?;
    let Some(row) = rows.next().await.map_err(|e| e.to_string())? else {
        return Ok(false);
    };
    let count: i64 = row.get(0).map_err(|e| e.to_string())?;
    Ok(count > 0)
}

async fn count_resolved_edges_impl(conn: &turso::Connection) -> Result<usize, String> {
    let mut rows = conn
        .query("SELECT COUNT(*) FROM resolved_edges", ())
        .await
        .map_err(|e| e.to_string())?;
    let Some(row) = rows.next().await.map_err(|e| e.to_string())? else {
        return Ok(0);
    };
    let count: i64 = row.get(0).map_err(|e| e.to_string())?;
    usize::try_from(count).map_err(|_| format!("negative count: {}", count))
}

impl CodeIndexDb {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let path_str = path.to_str().ok_or("invalid path")?.to_string();
        let conn = run_async(open_db(&path_str))
            .map_err(|error| crate::describe_lock_error(&path, error))?;
        Ok(Self {
            db: Arc::new(Mutex::new(InnerDb { conn })),
        })
    }

    pub fn load_file_hashes(&self) -> Result<HashMap<String, (String, i64)>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(load_file_hashes_impl(&db.conn))
    }

    pub fn bulk_upsert_files(&self, files: &[FileData]) -> Result<(), String> {
        let files = files.to_vec();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let result = run_async(with_write_tx(&db.conn, || async {
            upsert_files_impl(&db.conn, &files).await
        }));
        debug_assert!(db.conn.is_autocommit().unwrap_or(false));
        result
    }

    pub fn apply_base_write(
        &self,
        files: &[FileData],
        known_paths: &[String],
    ) -> Result<(), String> {
        let files = files.to_vec();
        let known = known_paths.to_vec();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let result = run_async(with_write_tx(&db.conn, || async {
            apply_base_write_impl(&db.conn, &files, &known).await
        }));
        debug_assert!(db.conn.is_autocommit().unwrap_or(false));
        result
    }

    pub fn query_symbols(
        &self,
        name: Option<&str>,
        kind: Option<&str>,
        file: Option<&str>,
        path: Option<&str>,
        language: Option<&str>,
        max_results: Option<usize>,
    ) -> Result<Vec<CodeIntelSymbol>, String> {
        let name = name.map(|s| s.to_string());
        let kind = kind.map(|s| s.to_string());
        let file = file.map(|s| s.to_string());
        let path = path.map(|s| s.to_string());
        let language = language.map(|s| s.to_string());
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(query_symbols_impl(
            &db.conn,
            name.as_deref(),
            kind.as_deref(),
            file.as_deref(),
            path.as_deref(),
            language.as_deref(),
            max_results,
        ))
    }

    pub fn query_deps(
        &self,
        file: Option<&str>,
        target: Option<&str>,
        reverse: bool,
        depth: Option<usize>,
        language: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let file = file.map(|s| s.to_string());
        let target = target.map(|s| s.to_string());
        let language = language.map(|s| s.to_string());
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(query_deps_impl(
            &db.conn,
            file.as_deref(),
            target.as_deref(),
            reverse,
            depth,
            language.as_deref(),
        ))
    }

    pub fn get_file_count_and_max_mtime(&self) -> Result<(usize, i64), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(get_file_count_and_max_mtime_impl(&db.conn))
    }

    pub fn count_symbols(&self) -> Result<usize, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(count_symbols_impl(&db.conn))
    }

    pub fn count_deps(&self) -> Result<usize, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(count_deps_impl(&db.conn))
    }

    pub fn count_edges(&self) -> Result<usize, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(count_edges_impl(&db.conn))
    }

    pub fn list_files(&self) -> Result<Vec<String>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(list_files_impl(&db.conn))
    }

    pub fn query_edges(&self, q: &EdgeQuery) -> Result<Vec<CodeEdge>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(query_edges_impl(&db.conn, q))
    }

    pub fn delete_stale_files(&self, known_paths: &[String]) -> Result<usize, String> {
        let known = known_paths.to_vec();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let result = run_async(with_write_tx(&db.conn, || async {
            delete_stale_files_impl(&db.conn, &known).await
        }));
        debug_assert!(db.conn.is_autocommit().unwrap_or(false));
        result
    }

    pub fn replace_resolved_edges(&self, edges: &[ResolvedCodeEdge]) -> Result<(), String> {
        let edges = edges.to_vec();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let result = run_async(with_write_tx(&db.conn, || async {
            replace_resolved_edges_impl(&db.conn, &edges).await
        }));
        debug_assert!(db.conn.is_autocommit().unwrap_or(false));
        result
    }

    pub fn load_resolved_edges(&self) -> Result<Vec<ResolvedCodeEdge>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(load_resolved_edges_impl(&db.conn))
    }

    pub fn has_resolved_edges(&self) -> Result<bool, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(has_resolved_edges_impl(&db.conn))
    }

    pub fn count_resolved_edges(&self) -> Result<usize, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(count_resolved_edges_impl(&db.conn))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_file(path: &str, name: &str, mtime: i64) -> FileData {
        FileData {
            path: path.to_string(),
            sha256: format!("sha-{}", name),
            mtime,
            language: "rust".to_string(),
            symbols: vec![CodeIntelSymbol {
                file: path.to_string(),
                name: name.to_string(),
                kind: "function".to_string(),
                line: 1,
                column: 0,
                snippet: format!("fn {}() {{}}", name),
                language: "rust".to_string(),
            }],
            deps: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn sample_edge(source: &str) -> ResolvedCodeEdge {
        ResolvedCodeEdge {
            edge_type: "calls".to_string(),
            source_file: source.to_string(),
            source_symbol: Some("caller".to_string()),
            target_file: "src/target.rs".to_string(),
            target_symbol: Some("target".to_string()),
            line: 1,
            provenance: EdgeProvenance::Explicit,
            via: Vec::new(),
        }
    }

    fn autocommit(db: &CodeIndexDb) -> bool {
        let inner = db.db.lock().expect("db lock");
        inner.conn.is_autocommit().expect("autocommit state")
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn every_write_path_returns_to_autocommit() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        db.bulk_upsert_files(&[sample_file("src/a.rs", "a", 1)])
            .expect("upsert");
        assert!(autocommit(&db), "bulk upsert must commit");
        db.delete_stale_files(&["src/a.rs".to_string()])
            .expect("delete stale");
        assert!(autocommit(&db), "stale delete must commit");
        db.apply_base_write(
            &[sample_file("src/b.rs", "b", 2)],
            &["src/b.rs".to_string()],
        )
        .expect("base write");
        assert!(autocommit(&db), "base write must commit");
        db.replace_resolved_edges(&[sample_edge("src/b.rs")])
            .expect("replace edges");
        assert!(autocommit(&db), "edge replace must commit");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn failed_base_write_restores_autocommit_and_prior_rows() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        db.bulk_upsert_files(&[sample_file("src/a.rs", "a", 1)])
            .expect("seed base");
        db.replace_resolved_edges(&[sample_edge("src/a.rs")])
            .expect("seed edges");
        let mut broken = sample_file("src/b.rs", "b", 2);
        broken.symbols[0].line = usize::MAX;
        let result = db.apply_base_write(
            &[sample_file("src/c.rs", "c", 3), broken],
            &["src/a.rs".to_string(), "src/c.rs".to_string()],
        );
        assert!(result.is_err(), "overflowing symbol must fail the write");
        assert!(autocommit(&db), "failed write must roll back");
        let mut files = db.list_files().expect("list");
        files.sort();
        assert_eq!(
            files,
            vec!["src/a.rs".to_string()],
            "prior base rows survive"
        );
        assert_eq!(
            db.count_resolved_edges().expect("count"),
            1,
            "prior resolved edges survive a rolled-back base write"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn base_write_commits_new_rows_and_clears_resolved_edges() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        db.bulk_upsert_files(&[sample_file("src/a.rs", "a", 1)])
            .expect("seed base");
        db.replace_resolved_edges(&[sample_edge("src/a.rs")])
            .expect("seed edges");
        db.apply_base_write(
            &[sample_file("src/b.rs", "b", 2)],
            &["src/a.rs".to_string(), "src/b.rs".to_string()],
        )
        .expect("base write");
        db.apply_base_write(&[], &["src/a.rs".to_string(), "src/b.rs".to_string()])
            .expect("base write without changes");
        assert_eq!(
            db.get_file_count_and_max_mtime().expect("count").0,
            2,
            "both base rows are committed"
        );
        assert_eq!(
            db.count_resolved_edges().expect("count"),
            0,
            "base commit invalidates resolved edges atomically"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_open_and_query_empty() {
        let path = PathBuf::from(":memory:");
        let db = CodeIndexDb::open(path).expect("should open :memory: db");

        let symbols = db
            .query_symbols(None, None, None, None, None, None)
            .expect("query empty symbols");
        assert!(symbols.is_empty(), "empty db should return no symbols");

        let deps = db
            .query_deps(None, None, false, None, None)
            .expect("query empty deps");
        assert!(deps.is_empty(), "empty db should return no deps");

        let (count, _mtime) = db.get_file_count_and_max_mtime().expect("get file count");
        assert_eq!(count, 0, "empty db should have 0 files");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_bulk_upsert_and_query_symbols() {
        let path = PathBuf::from(":memory:");
        let db = CodeIndexDb::open(path).expect("open");

        let files = vec![
            FileData {
                path: "src/main.rs".into(),
                sha256: "abc".into(),
                mtime: 1000,
                language: "rust".into(),
                symbols: vec![
                    CodeIntelSymbol {
                        file: "src/main.rs".into(),
                        name: "main".into(),
                        kind: "function".into(),
                        line: 1,
                        column: 0,
                        snippet: "fn main() {}".into(),
                        language: "rust".into(),
                    },
                    CodeIntelSymbol {
                        file: "src/main.rs".into(),
                        name: "Helper".into(),
                        kind: "struct".into(),
                        line: 5,
                        column: 0,
                        snippet: "struct Helper;".into(),
                        language: "rust".into(),
                    },
                ],
                deps: vec![CodeIntelDep {
                    target: "std::io".into(),
                    line: 1,
                    kind: "use".into(),
                }],
                edges: vec![],
            },
            FileData {
                path: "src/lib.rs".into(),
                sha256: "def".into(),
                mtime: 2000,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/lib.rs".into(),
                    name: "add".into(),
                    kind: "function".into(),
                    line: 10,
                    column: 0,
                    snippet: "pub fn add() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
        ];

        db.bulk_upsert_files(&files).expect("bulk upsert");

        let all = db
            .query_symbols(None, None, None, None, None, None)
            .expect("query all");
        assert_eq!(all.len(), 3, "should have 3 symbols total");

        let named = db
            .query_symbols(Some("main"), None, None, None, None, None)
            .expect("query by name");
        assert_eq!(named.len(), 1, "should find 1 symbol matching 'main'");
        assert_eq!(named[0].name, "main");

        let structs = db
            .query_symbols(None, Some("struct"), None, None, None, None)
            .expect("query by kind");
        assert_eq!(structs.len(), 1, "should find 1 struct");
        assert_eq!(structs[0].name, "Helper");

        let main_syms = db
            .query_symbols(None, None, Some("src/main.rs"), None, None, None)
            .expect("query by file");
        assert_eq!(main_syms.len(), 2, "src/main.rs has 2 symbols");

        let src_syms = db
            .query_symbols(None, None, None, Some("src"), None, None)
            .expect("query by path");
        assert_eq!(src_syms.len(), 3, "src/ has 3 symbols");

        let rust_syms = db
            .query_symbols(None, None, None, None, Some("rust"), None)
            .expect("query by language");
        assert_eq!(rust_syms.len(), 3, "rust has 3 symbols");

        let (count, mtime) = db.get_file_count_and_max_mtime().expect("count");
        assert_eq!(count, 2, "should have 2 files");
        assert_eq!(mtime, 2000, "max mtime should be 2000");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_deps() {
        let path = PathBuf::from(":memory:");
        let db = CodeIndexDb::open(path).expect("open");

        let files = vec![FileData {
            path: "src/main.rs".into(),
            sha256: "abc".into(),
            mtime: 1000,
            language: "rust".into(),
            symbols: vec![],
            deps: vec![
                CodeIntelDep {
                    target: "std::io".into(),
                    line: 1,
                    kind: "use".into(),
                },
                CodeIntelDep {
                    target: "crate::helper".into(),
                    line: 2,
                    kind: "use".into(),
                },
            ],
            edges: vec![],
        }];

        db.bulk_upsert_files(&files).expect("bulk upsert");

        let deps = db
            .query_deps(Some("src/main.rs"), None, false, None, None)
            .expect("query deps");
        assert_eq!(deps.len(), 2, "should have 2 deps");

        let filtered = db
            .query_deps(Some("src/main.rs"), Some("std"), false, None, None)
            .expect("query deps filtered");
        assert_eq!(filtered.len(), 1, "should have 1 dep matching 'std'");

        let reverse = db
            .query_deps(None, Some("std"), true, None, None)
            .expect("reverse query");
        assert_eq!(reverse.len(), 1, "1 file depends on std");

        let err = db
            .query_deps(Some("src/main.rs"), None, false, Some(2), None)
            .expect_err("depth > 1 should error");
        assert!(
            err.contains("not yet supported"),
            "error should mention depth limit: {}",
            err
        );

        let lang_deps = db
            .query_deps(Some("src/main.rs"), None, false, None, Some("rust"))
            .expect("query deps by language");
        assert_eq!(lang_deps.len(), 2, "rust deps count");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_staleness_detection() {
        let path = PathBuf::from(":memory:");
        let db = CodeIndexDb::open(path).expect("open");

        let files = vec![
            FileData {
                path: "src/main.rs".into(),
                sha256: "abc".into(),
                mtime: 1000,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![],
                edges: vec![],
            },
            FileData {
                path: "src/lib.rs".into(),
                sha256: "def".into(),
                mtime: 2000,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![],
                edges: vec![],
            },
        ];

        db.bulk_upsert_files(&files).expect("bulk upsert");
        assert_eq!(
            db.get_file_count_and_max_mtime().unwrap().0,
            2,
            "2 files after upsert"
        );

        let known = vec!["src/main.rs".into()];
        db.delete_stale_files(&known).expect("delete stale");
        assert_eq!(
            db.get_file_count_and_max_mtime().unwrap().0,
            1,
            "only 1 file should remain"
        );

        let remaining = db
            .query_symbols(None, None, Some("src/main.rs"), None, None, None)
            .expect("query remaining");
        assert!(remaining.is_empty(), "main.rs has no symbols");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_symbols_by_name_substring() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![FileData {
            path: "src/users.rs".into(),
            sha256: "a".into(),
            mtime: 1,
            language: "rust".into(),
            symbols: vec![
                CodeIntelSymbol {
                    file: "src/users.rs".into(),
                    name: "User".into(),
                    kind: "struct".into(),
                    line: 1,
                    column: 0,
                    snippet: "struct User;".into(),
                    language: "rust".into(),
                },
                CodeIntelSymbol {
                    file: "src/users.rs".into(),
                    name: "UserService".into(),
                    kind: "struct".into(),
                    line: 5,
                    column: 0,
                    snippet: "struct UserService;".into(),
                    language: "rust".into(),
                },
                CodeIntelSymbol {
                    file: "src/users.rs".into(),
                    name: "Admin".into(),
                    kind: "struct".into(),
                    line: 10,
                    column: 0,
                    snippet: "struct Admin;".into(),
                    language: "rust".into(),
                },
            ],
            deps: vec![],
            edges: vec![],
        }];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_symbols(Some("User"), None, None, None, None, None)
            .expect("query");
        assert_eq!(results.len(), 2, "should match User and UserService");
        let names: Vec<&str> = results.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"User"));
        assert!(names.contains(&"UserService"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_symbols_by_path_suffix() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![
            FileData {
                path: "src/auth.rs".into(),
                sha256: "a".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/auth.rs".into(),
                    name: "login".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn login() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
            FileData {
                path: "lib/auth.rs".into(),
                sha256: "b".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "lib/auth.rs".into(),
                    name: "authenticate".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn authenticate() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
        ];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_symbols(None, None, Some("auth.rs"), None, None, None)
            .expect("query");
        assert_eq!(results.len(), 2, "suffix match should return both files");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_symbols_by_path_exact() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![
            FileData {
                path: "src/auth.rs".into(),
                sha256: "a".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/auth.rs".into(),
                    name: "login".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn login() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
            FileData {
                path: "lib/auth.rs".into(),
                sha256: "b".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "lib/auth.rs".into(),
                    name: "authenticate".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn authenticate() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
        ];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_symbols(None, None, Some("src/auth.rs"), None, None, None)
            .expect("query");
        assert_eq!(results.len(), 1, "exact match should return 1 file");
        assert_eq!(results[0].name, "login");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_symbols_with_path_filter() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![
            FileData {
                path: "src/auth.rs".into(),
                sha256: "a".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/auth.rs".into(),
                    name: "login".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn login() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
            FileData {
                path: "tests/auth_test.rs".into(),
                sha256: "b".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "tests/auth_test.rs".into(),
                    name: "test_login".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn test_login() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
        ];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_symbols(None, None, None, Some("src"), None, None)
            .expect("query");
        assert_eq!(results.len(), 1, "path prefix filter should match 1 file");
        assert_eq!(results[0].name, "login");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_symbols_with_multiple_filters() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![
            FileData {
                path: "src/main.rs".into(),
                sha256: "a".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![
                    CodeIntelSymbol {
                        file: "src/main.rs".into(),
                        name: "main".into(),
                        kind: "function".into(),
                        line: 1,
                        column: 0,
                        snippet: "fn main() {}".into(),
                        language: "rust".into(),
                    },
                    CodeIntelSymbol {
                        file: "src/main.rs".into(),
                        name: "Helper".into(),
                        kind: "struct".into(),
                        line: 5,
                        column: 0,
                        snippet: "struct Helper;".into(),
                        language: "rust".into(),
                    },
                ],
                deps: vec![],
                edges: vec![],
            },
            FileData {
                path: "src/lib.rs".into(),
                sha256: "b".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/lib.rs".into(),
                    name: "helper".into(),
                    kind: "function".into(),
                    line: 10,
                    column: 0,
                    snippet: "fn helper() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
        ];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_symbols(
                Some("helper"),
                Some("function"),
                None,
                None,
                Some("rust"),
                None,
            )
            .expect("query");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "helper");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_symbols_no_match_returns_empty() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![FileData {
            path: "src/main.rs".into(),
            sha256: "a".into(),
            mtime: 1,
            language: "rust".into(),
            symbols: vec![CodeIntelSymbol {
                file: "src/main.rs".into(),
                name: "main".into(),
                kind: "function".into(),
                line: 1,
                column: 0,
                snippet: "fn main() {}".into(),
                language: "rust".into(),
            }],
            deps: vec![],
            edges: vec![],
        }];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_symbols(Some("NoSuchSymbol"), None, None, None, None, None)
            .expect("query");
        assert!(results.is_empty(), "no-match query should return empty vec");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_symbols_with_max_results() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let mut symbols = Vec::new();
        for i in 0..100 {
            symbols.push(CodeIntelSymbol {
                file: "src/lib.rs".into(),
                name: format!("func_{}", i),
                kind: "function".into(),
                line: i as usize,
                column: 0,
                snippet: format!("fn func_{}() {{}}", i),
                language: "rust".into(),
            });
        }
        let files = vec![FileData {
            path: "src/lib.rs".into(),
            sha256: "a".into(),
            mtime: 1,
            language: "rust".into(),
            symbols,
            deps: vec![],
            edges: vec![],
        }];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_symbols(None, None, None, None, None, Some(10))
            .expect("query");
        assert_eq!(results.len(), 10, "max_results=10 should return 10 symbols");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_symbols_like_escaping() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![FileData {
            path: "src/engine.rs".into(),
            sha256: "a".into(),
            mtime: 1,
            language: "rust".into(),
            symbols: vec![
                CodeIntelSymbol {
                    file: "src/engine.rs".into(),
                    name: "engine_state".into(),
                    kind: "struct".into(),
                    line: 1,
                    column: 0,
                    snippet: "struct EngineState;".into(),
                    language: "rust".into(),
                },
                CodeIntelSymbol {
                    file: "src/engine.rs".into(),
                    name: "engineXstate".into(),
                    kind: "struct".into(),
                    line: 2,
                    column: 0,
                    snippet: "struct EngineXState;".into(),
                    language: "rust".into(),
                },
            ],
            deps: vec![],
            edges: vec![],
        }];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_symbols(Some("engine_state"), None, None, None, None, None)
            .expect("query");
        let names: Vec<&str> = results.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"engine_state"),
            "should match literal engine_state"
        );
        assert!(
            !names.contains(&"engineXstate"),
            "should NOT match engineXstate (underscore not wildcard)"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_deps_normal() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![FileData {
            path: "src/engine.rs".into(),
            sha256: "a".into(),
            mtime: 1,
            language: "rust".into(),
            symbols: vec![],
            deps: vec![
                CodeIntelDep {
                    target: "crate::models".into(),
                    line: 1,
                    kind: "use".into(),
                },
                CodeIntelDep {
                    target: "tokio".into(),
                    line: 2,
                    kind: "use".into(),
                },
            ],
            edges: vec![],
        }];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_deps(Some("src/engine.rs"), None, false, None, None)
            .expect("query");
        assert_eq!(results.len(), 2, "should return 2 deps");
        let targets: Vec<&str> = results
            .iter()
            .map(|r| r["target"].as_str().unwrap())
            .collect();
        assert!(targets.contains(&"crate::models"));
        assert!(targets.contains(&"tokio"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_deps_reverse_distinct() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![
            FileData {
                path: "src/engine.rs".into(),
                sha256: "a".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![
                    CodeIntelDep {
                        target: "shared::util".into(),
                        line: 1,
                        kind: "use".into(),
                    },
                    CodeIntelDep {
                        target: "shared::util".into(),
                        line: 2,
                        kind: "use".into(),
                    },
                ],
                edges: vec![],
            },
            FileData {
                path: "src/other.rs".into(),
                sha256: "b".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![CodeIntelDep {
                    target: "shared::util".into(),
                    line: 1,
                    kind: "use".into(),
                }],
                edges: vec![],
            },
            FileData {
                path: "src/unrelated.rs".into(),
                sha256: "c".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![CodeIntelDep {
                    target: "something::else".into(),
                    line: 1,
                    kind: "use".into(),
                }],
                edges: vec![],
            },
        ];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_deps(None, Some("shared::util"), true, None, None)
            .expect("query");
        assert_eq!(
            results.len(),
            2,
            "DISTINCT should return 2 files, not 3 rows"
        );
        let files_found: Vec<&str> = results
            .iter()
            .map(|r| r["file"].as_str().unwrap())
            .collect();
        assert!(files_found.contains(&"src/engine.rs"));
        assert!(files_found.contains(&"src/other.rs"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_deps_with_language_filter() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![
            FileData {
                path: "src/main.rs".into(),
                sha256: "a".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![
                    CodeIntelDep {
                        target: "std::io".into(),
                        line: 1,
                        kind: "use".into(),
                    },
                    CodeIntelDep {
                        target: "crate::helper".into(),
                        line: 2,
                        kind: "use".into(),
                    },
                ],
                edges: vec![],
            },
            FileData {
                path: "src/main.py".into(),
                sha256: "b".into(),
                mtime: 1,
                language: "python".into(),
                symbols: vec![],
                deps: vec![CodeIntelDep {
                    target: "os".into(),
                    line: 1,
                    kind: "import".into(),
                }],
                edges: vec![],
            },
        ];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_deps(None, None, false, None, Some("rust"))
            .expect("query");
        assert!(
            results.is_empty(),
            "normal mode without file filter returns empty"
        );

        let results = db
            .query_deps(Some("src/main.rs"), None, false, None, Some("rust"))
            .expect("query");
        assert_eq!(results.len(), 2, "should return 2 rust deps");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_deps_depth_one() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![FileData {
            path: "src/main.rs".into(),
            sha256: "a".into(),
            mtime: 1,
            language: "rust".into(),
            symbols: vec![],
            deps: vec![CodeIntelDep {
                target: "std::collections".into(),
                line: 1,
                kind: "use".into(),
            }],
            edges: vec![],
        }];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let results = db
            .query_deps(Some("src/main.rs"), None, false, Some(1), None)
            .expect("query");
        assert_eq!(results.len(), 1, "depth=1 returns direct deps");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_deps_depth_greater_than_one_returns_error() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![FileData {
            path: "src/main.rs".into(),
            sha256: "a".into(),
            mtime: 1,
            language: "rust".into(),
            symbols: vec![],
            deps: vec![CodeIntelDep {
                target: "std::io".into(),
                line: 1,
                kind: "use".into(),
            }],
            edges: vec![],
        }];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let err = db
            .query_deps(Some("src/main.rs"), None, false, Some(2), None)
            .expect_err("depth>1 should error");
        assert!(
            err.contains("not yet supported"),
            "error should mention 'not yet supported': {}",
            err
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_bulk_upsert_files_transaction_atomicity() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![
            FileData {
                path: "src/a.rs".into(),
                sha256: "a".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/a.rs".into(),
                    name: "A".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn a() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
            FileData {
                path: "src/b.rs".into(),
                sha256: "b".into(),
                mtime: 2,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/b.rs".into(),
                    name: "B".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn b() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
            FileData {
                path: "src/c.rs".into(),
                sha256: "c".into(),
                mtime: 3,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/c.rs".into(),
                    name: "C".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn c() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![],
                edges: vec![],
            },
        ];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let (count, mtime) = db.get_file_count_and_max_mtime().expect("count");
        assert_eq!(count, 3, "all 3 files should be committed");
        assert_eq!(mtime, 3, "max mtime should be 3");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_delete_stale_files() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![
            FileData {
                path: "src/a.rs".into(),
                sha256: "a".into(),
                mtime: 1,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/a.rs".into(),
                    name: "A".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn a() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![CodeIntelDep {
                    target: "std::io".into(),
                    line: 1,
                    kind: "use".into(),
                }],
                edges: vec![],
            },
            FileData {
                path: "src/b.rs".into(),
                sha256: "b".into(),
                mtime: 2,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/b.rs".into(),
                    name: "B".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn b() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![CodeIntelDep {
                    target: "tokio".into(),
                    line: 1,
                    kind: "use".into(),
                }],
                edges: vec![],
            },
            FileData {
                path: "src/c.rs".into(),
                sha256: "c".into(),
                mtime: 3,
                language: "rust".into(),
                symbols: vec![CodeIntelSymbol {
                    file: "src/c.rs".into(),
                    name: "C".into(),
                    kind: "function".into(),
                    line: 1,
                    column: 0,
                    snippet: "fn c() {}".into(),
                    language: "rust".into(),
                }],
                deps: vec![CodeIntelDep {
                    target: "serde".into(),
                    line: 1,
                    kind: "use".into(),
                }],
                edges: vec![],
            },
        ];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let known = vec!["src/a.rs".into(), "src/b.rs".into()];
        db.delete_stale_files(&known).expect("delete stale");

        let (count, _) = db.get_file_count_and_max_mtime().expect("count");
        assert_eq!(count, 2, "only 2 files should remain");

        let c_syms = db
            .query_symbols(None, None, Some("src/c.rs"), None, None, None)
            .expect("query");
        assert!(c_syms.is_empty(), "c.rs symbols should be deleted");

        let c_deps = db
            .query_deps(Some("src/c.rs"), None, false, None, None)
            .expect("query");
        assert!(c_deps.is_empty(), "c.rs deps should be deleted");

        let a_syms = db
            .query_symbols(None, None, Some("src/a.rs"), None, None, None)
            .expect("query");
        assert_eq!(a_syms.len(), 1);
        assert_eq!(a_syms[0].name, "A");

        let b_syms = db
            .query_symbols(None, None, Some("src/b.rs"), None, None, None)
            .expect("query");
        assert_eq!(b_syms.len(), 1);
        assert_eq!(b_syms[0].name, "B");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_file_count_and_max_mtime() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![
            FileData {
                path: "src/a.rs".into(),
                sha256: "a".into(),
                mtime: 1000,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![],
                edges: vec![],
            },
            FileData {
                path: "src/b.rs".into(),
                sha256: "b".into(),
                mtime: 5000,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![],
                edges: vec![],
            },
            FileData {
                path: "src/c.rs".into(),
                sha256: "c".into(),
                mtime: 3000,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![],
                edges: vec![],
            },
        ];
        db.bulk_upsert_files(&files).expect("bulk upsert");

        let (count, mtime) = db.get_file_count_and_max_mtime().expect("count");
        assert_eq!(count, 3);
        assert_eq!(mtime, 5000, "max mtime should be 5000");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_multiple_rebuilds_preserves_data() {
        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");
        let files = vec![FileData {
            path: "src/main.rs".into(),
            sha256: "abc".into(),
            mtime: 1000,
            language: "rust".into(),
            symbols: vec![CodeIntelSymbol {
                file: "src/main.rs".into(),
                name: "main".into(),
                kind: "function".into(),
                line: 1,
                column: 0,
                snippet: "fn main() {}".into(),
                language: "rust".into(),
            }],
            deps: vec![CodeIntelDep {
                target: "std::io".into(),
                line: 1,
                kind: "use".into(),
            }],
            edges: vec![],
        }];
        db.bulk_upsert_files(&files).expect("first upsert");

        db.bulk_upsert_files(&files).expect("second upsert");

        let syms = db
            .query_symbols(None, None, None, None, None, None)
            .expect("query");
        assert_eq!(syms.len(), 1, "symbols preserved after second upsert");
        assert_eq!(syms[0].name, "main");

        let deps = db
            .query_deps(Some("src/main.rs"), None, false, None, None)
            .expect("query");
        assert_eq!(deps.len(), 1, "deps preserved after second upsert");
        assert_eq!(deps[0]["target"], "std::io");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_resolved_edges_round_trip() {
        use wm_engine::models::edge_type_model::EdgeProvenance;

        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");

        assert!(!db.has_resolved_edges().unwrap());
        assert_eq!(db.count_resolved_edges().unwrap(), 0);
        assert!(db.load_resolved_edges().unwrap().is_empty());

        let edges = vec![
            ResolvedCodeEdge {
                edge_type: "calls".into(),
                source_file: "src/main.rs".into(),
                source_symbol: Some("caller".into()),
                target_file: "src/lib.rs".into(),
                target_symbol: Some("helper".into()),
                line: 10,
                provenance: EdgeProvenance::Explicit,
                via: Vec::new(),
            },
            ResolvedCodeEdge {
                edge_type: "imports".into(),
                source_file: "src/main.rs".into(),
                source_symbol: None,
                target_file: "src/bar.rs".into(),
                target_symbol: Some("crate::foo::Bar".into()),
                line: 3,
                provenance: EdgeProvenance::Derived,
                via: vec!["src/foo.rs".into()],
            },
        ];

        db.replace_resolved_edges(&edges).expect("replace");
        assert!(db.has_resolved_edges().unwrap());
        assert_eq!(db.count_resolved_edges().unwrap(), 2);

        let loaded = db.load_resolved_edges().unwrap();
        assert_eq!(loaded.len(), 2);

        let call_edge = loaded.iter().find(|e| e.edge_type == "calls").unwrap();
        assert_eq!(call_edge.source_file, "src/main.rs");
        assert_eq!(call_edge.source_symbol, Some("caller".into()));
        assert_eq!(call_edge.target_file, "src/lib.rs");
        assert_eq!(call_edge.target_symbol, Some("helper".into()));
        assert_eq!(call_edge.line, 10);
        assert_eq!(call_edge.provenance, EdgeProvenance::Explicit);
        assert!(call_edge.via.is_empty());

        let import_edge = loaded.iter().find(|e| e.edge_type == "imports").unwrap();
        assert_eq!(import_edge.source_file, "src/main.rs");
        assert_eq!(import_edge.source_symbol, None);
        assert_eq!(import_edge.target_file, "src/bar.rs");
        assert_eq!(import_edge.target_symbol, Some("crate::foo::Bar".into()));
        assert_eq!(import_edge.line, 3);
        assert_eq!(import_edge.provenance, EdgeProvenance::Derived);
        assert_eq!(import_edge.via, vec!["src/foo.rs".to_string()]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_replace_resolved_edges_is_idempotent() {
        use wm_engine::models::edge_type_model::EdgeProvenance;

        let db = CodeIndexDb::open(PathBuf::from(":memory:")).expect("open");

        let edges_v1 = vec![ResolvedCodeEdge {
            edge_type: "calls".into(),
            source_file: "src/a.rs".into(),
            source_symbol: Some("a".into()),
            target_file: "src/b.rs".into(),
            target_symbol: Some("b".into()),
            line: 1,
            provenance: EdgeProvenance::Explicit,
            via: Vec::new(),
        }];

        db.replace_resolved_edges(&edges_v1).expect("first replace");
        assert_eq!(db.count_resolved_edges().unwrap(), 1);

        let edges_v2 = vec![
            ResolvedCodeEdge {
                edge_type: "imports".into(),
                source_file: "src/x.rs".into(),
                source_symbol: None,
                target_file: "src/y.rs".into(),
                target_symbol: Some("y".into()),
                line: 5,
                provenance: EdgeProvenance::Derived,
                via: vec!["src/z.rs".into()],
            },
            ResolvedCodeEdge {
                edge_type: "calls".into(),
                source_file: "src/x.rs".into(),
                source_symbol: Some("x".into()),
                target_file: "src/y.rs".into(),
                target_symbol: Some("y_fn".into()),
                line: 10,
                provenance: EdgeProvenance::Explicit,
                via: Vec::new(),
            },
        ];

        db.replace_resolved_edges(&edges_v2)
            .expect("second replace");
        assert_eq!(db.count_resolved_edges().unwrap(), 2);

        let loaded = db.load_resolved_edges().unwrap();
        assert!(
            loaded.iter().all(|e| e.source_file == "src/x.rs"),
            "all edges should be from v2"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_resolved_edges_survive_process_restart_simulation() {
        use wm_engine::models::edge_type_model::EdgeProvenance;

        let tmp = std::env::temp_dir().join(format!("wm_test_resolved_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&tmp);

        {
            let db = CodeIndexDb::open(tmp.clone()).expect("open");
            let edges = vec![ResolvedCodeEdge {
                edge_type: "calls".into(),
                source_file: "src/main.rs".into(),
                source_symbol: Some("main".into()),
                target_file: "src/lib.rs".into(),
                target_symbol: Some("run".into()),
                line: 7,
                provenance: EdgeProvenance::Explicit,
                via: Vec::new(),
            }];
            db.replace_resolved_edges(&edges).expect("write");
        }

        {
            let db = CodeIndexDb::open(tmp.clone()).expect("reopen");
            assert!(db.has_resolved_edges().unwrap());
            let loaded = db.load_resolved_edges().unwrap();
            assert_eq!(loaded.len(), 1);
            assert_eq!(loaded[0].source_file, "src/main.rs");
            assert_eq!(loaded[0].target_file, "src/lib.rs");
        }

        let _ = std::fs::remove_file(&tmp);
    }
}
