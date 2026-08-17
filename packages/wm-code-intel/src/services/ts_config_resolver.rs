//! TypeScript configuration resolution (spec FR-2.5).
//!
//! Discovers `tsconfig.json` files, parses `compilerOptions.paths` and
//! `compilerOptions.baseUrl`, and resolves aliased import specifiers to
//! project-relative file paths.
//!
//! Also handles npm/pnpm/yarn workspace package resolution by scanning
//! `package.json` files at the project root for workspace globs.
//!
//! Design: deterministic, local, no network (NFR-2.1). Configuration is
//! loaded once at index time and cached.

use std::collections::HashMap;
use std::path::Path;

/// Parsed tsconfig path mappings for a specific tsconfig.json.
#[derive(Debug, Clone)]
pub struct TsPathMapping {
    /// The pattern (e.g. `@ui/button` or `@app/*`).
    pub pattern: String,
    /// The replacement paths relative to the tsconfig's baseUrl or dir.
    /// e.g. `["./src/libs/ui/button/src"]` or `["./src/app/*"]`.
    pub targets: Vec<String>,
}

/// Aggregated TypeScript resolution context for a project.
#[derive(Debug, Clone, Default)]
pub struct TsResolutionContext {
    /// Path mappings from all discovered tsconfig.json files.
    /// Key: directory containing the tsconfig (project-relative).
    /// Value: (baseUrl resolved to project-relative path, path mappings).
    pub configs: Vec<TsConfigEntry>,
    /// Workspace packages: package name → project-relative entry dir.
    pub workspace_packages: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TsConfigEntry {
    /// Project-relative directory containing the tsconfig.json.
    pub config_dir: String,
    /// Resolved baseUrl as a project-relative path (defaults to config_dir).
    pub base_url: String,
    /// Path alias mappings.
    pub mappings: Vec<TsPathMapping>,
}

impl TsResolutionContext {
    /// Discover and load all tsconfig.json files and workspace packages
    /// from a project root. This is deterministic and local (NFR-2.1).
    pub fn discover(project_root: &Path) -> Self {
        let mut ctx = Self::default();
        ctx.load_tsconfigs(project_root);
        ctx.load_workspace_packages(project_root);
        ctx
    }

    /// Resolve a TS/TSX import specifier using tsconfig paths and workspace
    /// packages. Returns candidate file paths (project-relative) if the
    /// specifier matches a known alias or workspace package.
    ///
    /// The `source_file` is project-relative so we can pick the right
    /// tsconfig based on which directory the source lives in.
    pub fn resolve_specifier(
        &self,
        source_file: &str,
        specifier: &str,
    ) -> Option<Vec<String>> {
        // 1. Try tsconfig path aliases.
        if let Some(candidates) = self.resolve_via_paths(source_file, specifier) {
            if !candidates.is_empty() {
                return Some(candidates);
            }
        }

        // 2. Try workspace packages.
        if let Some(candidates) = self.resolve_via_workspace(specifier) {
            if !candidates.is_empty() {
                return Some(candidates);
            }
        }

        None
    }

    fn resolve_via_paths(&self, source_file: &str, specifier: &str) -> Option<Vec<String>> {
        // Find the most specific tsconfig that covers the source file.
        let config = self.find_config_for(source_file)?;

        for mapping in &config.mappings {
            if let Some(matched) = match_path_pattern(&mapping.pattern, specifier) {
                let mut candidates = Vec::new();
                for target_pattern in &mapping.targets {
                    let resolved = apply_path_target(&config.base_url, target_pattern, &matched);
                    candidates.extend(ts_file_candidates_for(&resolved));
                }
                if !candidates.is_empty() {
                    return Some(candidates);
                }
            }
        }
        None
    }

    fn resolve_via_workspace(&self, specifier: &str) -> Option<Vec<String>> {
        // Check if the specifier matches a workspace package name.
        // e.g. `import { x } from '@myorg/shared'` → packages/shared/src/index.ts
        let take_count = 1 + usize::from(specifier.starts_with('@'));
        let pkg_name = specifier.split('/').take(take_count).collect::<Vec<_>>().join("/");

        let subpath = specifier.get(pkg_name.len() + 1..).unwrap_or("");

        if let Some(pkg_dir) = self.workspace_packages.get(&pkg_name) {
            let mut candidates = Vec::new();
            if subpath.is_empty() {
                candidates.extend(ts_file_candidates_for(&format!("{}/src/index", pkg_dir)));
                candidates.extend(ts_file_candidates_for(&format!("{}/index", pkg_dir)));
            }
            if !subpath.is_empty() {
                candidates.extend(ts_file_candidates_for(&format!("{}/src/{}", pkg_dir, subpath)));
                candidates.extend(ts_file_candidates_for(&format!("{}/{}", pkg_dir, subpath)));
            }
            if !candidates.is_empty() {
                return Some(candidates);
            }
        }
        None
    }

    fn find_config_for(&self, source_file: &str) -> Option<&TsConfigEntry> {
        // Find the tsconfig whose config_dir is the longest prefix of source_file.
        self.configs
            .iter()
            .filter(|c| {
                source_file.starts_with(&c.config_dir)
                    || c.config_dir.is_empty()
                    || source_file.starts_with(&format!("{}/", c.config_dir))
            })
            .max_by_key(|c| c.config_dir.len())
    }

    fn load_tsconfigs(&mut self, project_root: &Path) {
        // Walk for tsconfig.json files (skip node_modules, .git, etc.)
        for entry in walkdir::WalkDir::new(project_root)
            .into_iter()
            .filter_entry(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| !is_config_skip_dir(s))
                    .unwrap_or(false)
            })
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let name = entry.file_name().to_str().unwrap_or("");
            // Only process tsconfig.json (skip tsconfig.spec.json, tsconfig.app.json etc.
            // unless they also have paths — we'll read extensions from extends later)
            if name == "tsconfig.json" {
                if let Some(config) = parse_tsconfig(project_root, entry.path()) {
                    self.configs.push(config);
                }
            }
        }
        // Sort by config_dir length (most specific last for the find_config_for search).
        self.configs.sort_by_key(|c| c.config_dir.len());
    }

    fn load_workspace_packages(&mut self, project_root: &Path) {
        // Check root package.json for workspaces field.
        let root_pkg = project_root.join("package.json");
        if let Ok(content) = std::fs::read_to_string(&root_pkg) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(workspaces) = json.get("workspaces") {
                    let globs = extract_workspace_globs(workspaces);
                    for glob_pattern in globs {
                        self.scan_workspace_glob(project_root, &glob_pattern);
                    }
                }
            }
        }

        // Check pnpm-workspace.yaml
        let pnpm_ws = project_root.join("pnpm-workspace.yaml");
        if let Ok(content) = std::fs::read_to_string(&pnpm_ws) {
            // Simple YAML parsing for `packages:` list
            let mut in_packages = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == "packages:" {
                    in_packages = true;
                    continue;
                }
                if in_packages {
                    if trimmed.starts_with('-') {
                        let glob_pattern = trimmed
                            .trim_start_matches('-')
                            .trim()
                            .trim_matches('\'')
                            .trim_matches('"');
                        self.scan_workspace_glob(project_root, glob_pattern);
                    } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        break;
                    }
                }
            }
        }
    }

    fn scan_workspace_glob(&mut self, project_root: &Path, pattern: &str) {
        // Simple glob expansion: only handle `packages/*` and `apps/*` style.
        let base = pattern.trim_end_matches('*').trim_end_matches('/');
        let base_path = project_root.join(base);
        if !base_path.is_dir() {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let pkg_json = entry.path().join("package.json");
                if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                            let rel_dir = entry
                                .path()
                                .strip_prefix(project_root)
                                .unwrap_or(entry.path().as_path())
                                .to_string_lossy()
                                .to_string();
                            self.workspace_packages.insert(name.to_string(), rel_dir);
                        }
                    }
                }
            }
        }
    }
}

/// Parse a tsconfig.json file and extract path mappings.
fn parse_tsconfig(project_root: &Path, tsconfig_path: &Path) -> Option<TsConfigEntry> {
    let content = std::fs::read_to_string(tsconfig_path).ok()?;
    // Strip comments (tsconfig allows // comments)
    let stripped = strip_json_comments(&content);
    let json: serde_json::Value = serde_json::from_str(&stripped).ok()?;

    let config_dir = tsconfig_path
        .parent()?
        .strip_prefix(project_root)
        .unwrap_or(Path::new(""))
        .to_string_lossy()
        .to_string();

    let compiler_options = json.get("compilerOptions")?;

    // Parse baseUrl (defaults to config dir)
    let base_url_raw = compiler_options
        .get("baseUrl")
        .and_then(|b| b.as_str())
        .unwrap_or(".");

    let base_url = normalize_base_url(&config_dir, base_url_raw);

    // Parse paths
    let paths_obj = compiler_options.get("paths");
    let mappings: Vec<TsPathMapping> = paths_obj
        .and_then(|p| p.as_object())
        .map(|paths| {
            paths
                .iter()
                .map(|(pattern, targets)| {
                    let target_list = targets
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    TsPathMapping {
                        pattern: pattern.clone(),
                        targets: target_list,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    if mappings.is_empty() {
        return None; // No paths = nothing to contribute
    }

    Some(TsConfigEntry {
        config_dir,
        base_url,
        mappings,
    })
}

/// Normalize baseUrl relative to the config directory.
fn normalize_base_url(config_dir: &str, base_url: &str) -> String {
    if base_url == "." || base_url == "./" {
        return config_dir.to_string();
    }
    let rel = base_url.strip_prefix("./").unwrap_or(base_url);
    if config_dir.is_empty() {
        return rel.to_string();
    }
    format!("{}/{}", config_dir, rel)
}

/// Match a tsconfig path pattern against a specifier.
/// Returns the wildcard match (empty string for exact matches).
///
/// Patterns:
/// - `@ui/button` (exact) — matches only `@ui/button`
/// - `@app/*` (wildcard) — matches `@app/foo`, capture = `foo`
fn match_path_pattern(pattern: &str, specifier: &str) -> Option<String> {
    if let Some(prefix) = pattern.strip_suffix('*') {
        if !specifier.starts_with(prefix) {
            return None;
        }
        return Some(specifier[prefix.len()..].to_string());
    }
    // Exact match
    if specifier == pattern {
        return Some(String::new());
    }
    None
}

/// Apply a wildcard match to a target pattern.
/// e.g. base_url="apps/wm-web", target="./src/libs/*", matched="ui/button"
/// → "apps/wm-web/src/libs/ui/button"
fn apply_path_target(base_url: &str, target_pattern: &str, matched: &str) -> String {
    // Strip leading ./
    let target = target_pattern
        .strip_prefix("./")
        .unwrap_or(target_pattern);

    let resolved = match (target.contains('*'), !matched.is_empty()) {
        (true, _) => target.replace('*', matched),
        (false, true) => format!("{}/{}", target.trim_end_matches('/'), matched),
        (false, false) => target.to_string(),
    };

    if base_url.is_empty() {
        return resolved;
    }
    format!("{}/{}", base_url.trim_end_matches('/'), resolved)
}

/// Generate TS file candidates from a resolved path.
fn ts_file_candidates_for(base: &str) -> Vec<String> {
    let base = base.trim_end_matches('/');
    vec![
        format!("{}.ts", base),
        format!("{}.tsx", base),
        format!("{}.js", base),
        format!("{}/index.ts", base),
        format!("{}/index.tsx", base),
    ]
}

/// Strip // and /* */ comments from JSON (tsconfig allows them).
fn strip_json_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            output.push(ch);
            escape_next = false;
            continue;
        }
        if in_string {
            output.push(ch);
            if ch == '\\' {
                escape_next = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }
        if ch == '/' {
            match chars.peek() {
                Some('/') => {
                    for c in chars.by_ref() {
                        if c == '\n' {
                            output.push('\n');
                            break;
                        }
                    }
                }
                Some('*') => {
                    chars.next();
                    loop {
                        match chars.next() {
                            Some('*') if chars.peek() == Some(&'/') => {
                                chars.next();
                                break;
                            }
                            Some('\n') => output.push('\n'),
                            Some(_) => {}
                            None => break,
                        }
                    }
                }
                _ => output.push(ch),
            }
            continue;
        }
        output.push(ch);
    }
    output
}

/// Extract workspace globs from a `workspaces` field in package.json.
fn extract_workspace_globs(workspaces: &serde_json::Value) -> Vec<String> {
    match workspaces {
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        serde_json::Value::Object(obj) => {
            // { packages: [...] } format (yarn)
            obj.get("packages")
                .and_then(|p| p.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

fn is_config_skip_dir(name: &str) -> bool {
    matches!(
        name,
        "node_modules" | ".git" | "dist" | "build" | "target" | ".wm" | "out" | ".cache"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_exact_pattern() {
        assert_eq!(
            match_path_pattern("@ui/button", "@ui/button"),
            Some(String::new())
        );
        assert_eq!(match_path_pattern("@ui/button", "@ui/card"), None);
    }

    #[test]
    fn test_match_wildcard_pattern() {
        assert_eq!(
            match_path_pattern("@app/*", "@app/services/auth"),
            Some("services/auth".to_string())
        );
        assert_eq!(match_path_pattern("@app/*", "@other/foo"), None);
    }

    #[test]
    fn test_apply_path_target_exact() {
        let result = apply_path_target("apps/wm-web", "src/libs/ui/button/src", "");
        assert_eq!(result, "apps/wm-web/src/libs/ui/button/src");
    }

    #[test]
    fn test_apply_path_target_wildcard() {
        let result = apply_path_target("apps/wm-web", "src/app/*", "services/auth");
        assert_eq!(result, "apps/wm-web/src/app/services/auth");
    }

    #[test]
    fn test_apply_path_target_empty_base() {
        let result = apply_path_target("", "src/libs/*", "ui/button");
        assert_eq!(result, "src/libs/ui/button");
    }

    #[test]
    fn test_strip_json_comments() {
        let input = r#"{
  // This is a comment
  "key": "value", /* block comment */
  "paths": {}
}"#;
        let output = strip_json_comments(input);
        assert!(!output.contains("comment"));
        assert!(output.contains("\"key\""));
    }

    #[test]
    fn test_normalize_base_url() {
        assert_eq!(normalize_base_url("apps/wm-web", "."), "apps/wm-web");
        assert_eq!(normalize_base_url("apps/wm-web", "./src"), "apps/wm-web/src");
        assert_eq!(normalize_base_url("", "."), "");
        assert_eq!(normalize_base_url("", "./src"), "src");
    }

    #[test]
    fn test_tsconfig_paths_resolution() {
        let ctx = TsResolutionContext {
            configs: vec![TsConfigEntry {
                config_dir: "apps/wm-web".to_string(),
                base_url: "apps/wm-web".to_string(),
                mappings: vec![
                    TsPathMapping {
                        pattern: "@ui/button".to_string(),
                        targets: vec!["./src/libs/ui/button/src".to_string()],
                    },
                    TsPathMapping {
                        pattern: "@app/*".to_string(),
                        targets: vec!["./src/app/*".to_string()],
                    },
                ],
            }],
            workspace_packages: HashMap::new(),
        };

        // Exact alias match
        let result = ctx
            .resolve_specifier("apps/wm-web/src/app/main.ts", "@ui/button")
            .unwrap();
        assert!(result.iter().any(|c| c.contains("libs/ui/button/src")));

        // Wildcard alias match
        let result = ctx
            .resolve_specifier("apps/wm-web/src/app/main.ts", "@app/services/auth")
            .unwrap();
        assert!(result.iter().any(|c| c.contains("src/app/services/auth")));

        // No match — outside the tsconfig scope
        let result = ctx.resolve_specifier("other/file.ts", "@ui/button");
        assert!(result.is_none(), "file outside tsconfig scope should not match alias");
    }

    #[test]
    fn test_workspace_package_resolution() {
        let mut packages = HashMap::new();
        packages.insert("@myorg/shared".to_string(), "packages/shared".to_string());

        let ctx = TsResolutionContext {
            configs: Vec::new(),
            workspace_packages: packages,
        };

        // Package root import
        let result = ctx
            .resolve_specifier("apps/web/src/main.ts", "@myorg/shared")
            .unwrap();
        assert!(result.iter().any(|c| c.contains("packages/shared")));

        // Package subpath import
        let result = ctx
            .resolve_specifier("apps/web/src/main.ts", "@myorg/shared/utils")
            .unwrap();
        assert!(result.iter().any(|c| c.contains("packages/shared") && c.contains("utils")));
    }

    #[test]
    fn test_no_match_returns_none() {
        let ctx = TsResolutionContext::default();
        assert_eq!(ctx.resolve_specifier("src/main.ts", "unknown-pkg"), None);
    }
}
