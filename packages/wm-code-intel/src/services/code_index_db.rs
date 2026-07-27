use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::models::dep_model::CodeIntelDep;
use crate::models::symbol_model::CodeIntelSymbol;
use turso::params_from_iter;
use turso::Value;

#[derive(Debug, Clone)]
pub struct FileData {
    pub path: String,
    pub sha256: String,
    pub mtime: i64,
    pub language: String,
    pub symbols: Vec<CodeIntelSymbol>,
    pub deps: Vec<CodeIntelDep>,
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

async fn bulk_upsert_files_impl(
    conn: &turso::Connection,
    files: &[FileData],
) -> Result<(), String> {
    conn.execute("BEGIN TRANSACTION", ())
        .await
        .map_err(|e| e.to_string())?;

    let result = async {
        for file in files {
            conn.execute(
                "INSERT INTO code_files (path, sha256, mtime, language)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(path) DO UPDATE SET
                     sha256 = excluded.sha256,
                     mtime = excluded.mtime,
                     language = excluded.language",
                (
                    file.path.as_str(),
                    file.sha256.as_str(),
                    file.mtime,
                    file.language.as_str(),
                ),
            )
            .await
            .map_err(|e| e.to_string())?;

            conn.execute(
                "DELETE FROM code_symbols WHERE file = ?1",
                [file.path.as_str()],
            )
            .await
            .map_err(|e| e.to_string())?;

            for sym in &file.symbols {
                let line_i64: i64 = sym
                    .line
                    .try_into()
                    .map_err(|_| format!("line overflow for symbol `{}`", sym.name))?;
                let col_i64: i64 = sym
                    .column
                    .try_into()
                    .map_err(|_| format!("column overflow for symbol `{}`", sym.name))?;
                conn.execute(
                    "INSERT INTO code_symbols (file, name, kind, line, column, snippet, language)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    (
                        sym.file.as_str(),
                        sym.name.as_str(),
                        sym.kind.as_str(),
                        line_i64,
                        col_i64,
                        sym.snippet.as_str(),
                        sym.language.as_str(),
                    ),
                )
                .await
                .map_err(|e| e.to_string())?;
            }

            conn.execute(
                "DELETE FROM code_deps WHERE file = ?1",
                [file.path.as_str()],
            )
            .await
            .map_err(|e| e.to_string())?;

            for dep in &file.deps {
                let line_i64: i64 = dep
                    .line
                    .try_into()
                    .map_err(|_| format!("line overflow for dep `{}`", dep.target))?;
                conn.execute(
                    "INSERT INTO code_deps (file, target, line, kind, language)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    (
                        file.path.as_str(),
                        dep.target.as_str(),
                        line_i64,
                        dep.kind.as_str(),
                        file.language.as_str(),
                    ),
                )
                .await
                .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    };

    match result.await {
        Ok(()) => {
            conn.execute("COMMIT", ())
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await.map_err(|_| ());
            Err(e)
        }
    }
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

async fn delete_stale_files_impl(
    conn: &turso::Connection,
    known_paths: &[String],
) -> Result<(), String> {
    let known_set: HashSet<&str> = known_paths.iter().map(|s| s.as_str()).collect();

    let mut db_paths: Vec<String> = Vec::new();
    {
        let mut rows = conn
            .query("SELECT path FROM code_files", ())
            .await
            .map_err(|e| e.to_string())?;
        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            let p: String = row.get(0).map_err(|e| e.to_string())?;
            db_paths.push(p);
        }
    }

    for path in &db_paths {
        if !known_set.contains(path.as_str()) {
            let p = path.clone();
            conn.execute("DELETE FROM code_files WHERE path = ?1", [p.as_str()])
                .await
                .map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM code_symbols WHERE file = ?1", [p.as_str()])
                .await
                .map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM code_deps WHERE file = ?1", [p.as_str()])
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Run an async operation, bridging sync↔async.
/// Works both inside and outside a tokio runtime.
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

impl CodeIndexDb {
    /// Open or create the code index database at the given path.
    ///
    /// Must be called from within a tokio multi-thread runtime context
    /// (e.g. inside `#[tokio::main]` or a `#[tokio::test]`).
    ///
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let path_str = path.to_str().ok_or("invalid path")?.to_string();
        let conn = run_async(open_db(&path_str))?;
        Ok(Self {
            db: Arc::new(Mutex::new(InnerDb { conn })),
        })
    }

    /// Load all file hashes from the database.
    /// Returns a map of path → (sha256, mtime).
    ///
    pub fn load_file_hashes(&self) -> Result<HashMap<String, (String, i64)>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(load_file_hashes_impl(&db.conn))
    }

    /// Bulk upsert multiple files, their symbols, and deps in a single transaction.
    ///
    pub fn bulk_upsert_files(&self, files: &[FileData]) -> Result<(), String> {
        let files = files.to_vec();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(bulk_upsert_files_impl(&db.conn, &files))
    }

    /// Query symbols with optional filters. Builds a dynamic WHERE clause.
    ///
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

    /// Query dependencies with optional filters. Supports reverse lookup.
    /// Depth > 1 returns an error (not yet supported).
    ///
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

    /// Get the total number of indexed files and the maximum mtime.
    ///
    pub fn get_file_count_and_max_mtime(&self) -> Result<(usize, i64), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(get_file_count_and_max_mtime_impl(&db.conn))
    }

    /// Delete all entries for files not in `known_paths`.
    ///
    pub fn delete_stale_files(&self, known_paths: &[String]) -> Result<(), String> {
        let known = known_paths.to_vec();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        run_async(delete_stale_files_impl(&db.conn, &known))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
            },
            FileData {
                path: "src/lib.rs".into(),
                sha256: "def".into(),
                mtime: 2000,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![],
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
                    }, // duplicate dep
                ],
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
            },
            FileData {
                path: "src/b.rs".into(),
                sha256: "b".into(),
                mtime: 5000,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![],
            },
            FileData {
                path: "src/c.rs".into(),
                sha256: "c".into(),
                mtime: 3000,
                language: "rust".into(),
                symbols: vec![],
                deps: vec![],
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
}
