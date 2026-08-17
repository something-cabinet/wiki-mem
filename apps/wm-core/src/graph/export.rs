//! Graph export formats.
//!
//! Exports are **snapshots only, never a storage format**. They read
//! the in-memory `StableGraph` snapshot (arc-swap) and render it on demand;
//! markdown pages stay canonical and no on-disk graph persistence is
//! introduced. Every exporter is deterministic — the same graph snapshot
//! always renders the same bytes.
//!
//! Supported formats:
//! - JSON: mirrors the `wm_graph.full` wire shape so the dump validates
//!   against the same schema.
//! - GraphML: hand-written XML (no new dependencies) with a `graphml` root,
//!   `key` defs for node/edge attributes, and `node`/`edge` elements carrying
//!   stable ids — opens in Gephi / yEd.
//! - Obsidian: a vault directory with one `<type>/<name>.md` per page and
//!   `[[wikilink]]` lines matching wiki-mem outbound edges. Provenance is
//!   preserved as an HTML comment on each link line — Obsidian has no edge
//!   attribute model, so a comment is the least intrusive place.
//!   Exports are non-destructive: existing vault files are never deleted.

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;

use crate::engine::{EdgeType, GraphEdge, WikiPageMeta};

/// Canonical edge-type string, identical to the one used during graph
/// construction/dedup (graph/mod.rs). Standard variants map to their kebab
/// form (`references`, `extends`, ...); `Custom` emits its raw name.
fn edge_type_str(edge_type: &EdgeType) -> String {
    match edge_type {
        EdgeType::Custom(name) => name.to_lowercase(),
        other => other.as_yaml_str().to_string(),
    }
}

/// Resolve a path to its canonical absolute form for equality/nesting checks.
/// `canonicalize` fails for paths that do not exist yet (the export target
/// may be created by the exporter), so fall back to canonicalizing the parent
/// and appending the leaf, then to the raw path.
fn canonical_or_abs(p: &Path) -> PathBuf {
    if let Ok(c) = std::fs::canonicalize(p) {
        return c;
    }
    if let (Some(parent), Some(leaf)) = (p.parent(), p.file_name()) {
        if let Ok(parent_c) = std::fs::canonicalize(parent) {
            return parent_c.join(leaf);
        }
    }
    p.to_path_buf()
}

/// Convert a graph snapshot to the `wm_graph.full` wire shape
/// (`{success, nodes, node_count, edges, edge_count}`). Node objects carry
/// `id`/`title`/`page_type`/`degree`; edge objects carry
/// `source`/`target`/`edge_type`/`provenance` — provenance included where
/// format allows.
pub fn graph_to_json(graph: &StableGraph<WikiPageMeta, GraphEdge>) -> serde_json::Value {
    let nodes: Vec<serde_json::Value> = graph
        .node_indices()
        .map(|idx| {
            let meta = &graph[idx];
            let degree = graph.edges(idx).count().wrapping_add(
                graph
                    .edges_directed(idx, petgraph::Direction::Incoming)
                    .count(),
            );
            serde_json::json!({
                "id": meta.id,
                "title": meta.title,
                "page_type": meta.page_type.as_str(),
                "degree": degree,
            })
        })
        .collect();

    let edges: Vec<serde_json::Value> = graph
        .edge_indices()
        .map(|edge_idx| {
            let (source, target) = graph.edge_endpoints(edge_idx).expect("edge endpoints");
            let weight = &graph[edge_idx];
            serde_json::json!({
                "source": graph[source].id,
                "target": graph[target].id,
                "edge_type": edge_type_str(&weight.edge_type),
                "provenance": weight.provenance.as_str(),
            })
        })
        .collect();

    serde_json::json!({
        "success": true,
        "nodes": nodes,
        "node_count": nodes.len(),
        "edges": edges,
        "edge_count": edges.len(),
    })
}

/// Escape a string for inclusion in XML text or attribute values.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Render a graph snapshot as GraphML (directed graph, string keys for
/// `title`/`page_type` on nodes and `edge_type`/`provenance` on edges).
/// Deterministic: node ids are the wiki page ids, edge ids are `e0..eN-1`
/// in stable edge-index order.
pub fn graph_to_graphml(graph: &StableGraph<WikiPageMeta, GraphEdge>) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(
        "<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" \
         xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" \
         xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns \
         http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n",
    );
    out.push_str("  <key id=\"title\" for=\"node\" attr.name=\"title\" attr.type=\"string\"/>\n");
    out.push_str(
        "  <key id=\"page_type\" for=\"node\" attr.name=\"page_type\" attr.type=\"string\"/>\n",
    );
    out.push_str(
        "  <key id=\"edge_type\" for=\"edge\" attr.name=\"edge_type\" attr.type=\"string\"/>\n",
    );
    out.push_str(
        "  <key id=\"provenance\" for=\"edge\" attr.name=\"provenance\" attr.type=\"string\"/>\n",
    );
    out.push_str("  <graph id=\"wiki-mem\" edgedefault=\"directed\">\n");

    for idx in graph.node_indices() {
        let meta = &graph[idx];
        out.push_str(&format!("    <node id=\"{}\">\n", xml_escape(&meta.id)));
        out.push_str(&format!(
            "      <data key=\"title\">{}</data>\n",
            xml_escape(&meta.title)
        ));
        out.push_str(&format!(
            "      <data key=\"page_type\">{}</data>\n",
            xml_escape(meta.page_type.as_str())
        ));
        out.push_str("    </node>\n");
    }

    for (i, edge_idx) in graph.edge_indices().enumerate() {
        let (source, target) = graph.edge_endpoints(edge_idx).expect("edge endpoints");
        let weight = &graph[edge_idx];
        out.push_str(&format!(
            "    <edge id=\"e{}\" source=\"{}\" target=\"{}\">\n",
            i,
            xml_escape(&graph[source].id),
            xml_escape(&graph[target].id)
        ));
        out.push_str(&format!(
            "      <data key=\"edge_type\">{}</data>\n",
            xml_escape(&edge_type_str(&weight.edge_type))
        ));
        out.push_str(&format!(
            "      <data key=\"provenance\">{}</data>\n",
            xml_escape(weight.provenance.as_str())
        ));
        out.push_str("    </edge>\n");
    }

    out.push_str("  </graph>\n");
    out.push_str("</graphml>\n");
    out
}

/// Result of an Obsidian vault export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObsidianExport {
    /// Pages written (one `.md` per graph node).
    pub pages: usize,
    /// Wikilink lines written across all pages.
    pub wikilinks: usize,
}

/// Derive the vault-relative path (no extension) from a wiki page id.
/// `wiki:concepts:foo` → `concepts/foo`. Reversible with `path_to_id`.
fn id_to_rel_path(id: &str) -> PathBuf {
    PathBuf::from(id.strip_prefix("wiki:").unwrap_or(id).replace(':', "/"))
}

/// Build the frontmatter block for an exported page: keep any source
/// frontmatter fields and merge/overwrite `title`, `type`, `wiki_id`.
fn render_frontmatter(meta: &WikiPageMeta, source_content: &str) -> String {
    let (raw, _body) = crate::parser::extract_raw_frontmatter(source_content);
    let mut merged = match raw {
        Some(yaml) => serde_yaml::from_str::<serde_yaml::Mapping>(&yaml).unwrap_or_default(),
        None => serde_yaml::Mapping::new(),
    };
    merged.insert(
        serde_yaml::Value::String("title".into()),
        serde_yaml::Value::String(meta.title.clone()),
    );
    merged.insert(
        serde_yaml::Value::String("type".into()),
        serde_yaml::Value::String(meta.page_type.as_str().into()),
    );
    merged.insert(
        serde_yaml::Value::String("wiki_id".into()),
        serde_yaml::Value::String(meta.id.clone()),
    );
    let yaml = serde_yaml::to_string(&serde_yaml::Value::Mapping(merged))
        .unwrap_or_else(|_| format!("title: {}\ntype: {}\n", meta.title, meta.page_type.as_str()));
    format!("---\n{}\n---\n", yaml.trim_end())
}

/// Export the graph as an Obsidian vault under `out_dir`.
///
/// - Writes one `<type>/<name>.md` per graph node (path derived from the page
///   id, matching how `path_to_id` maps files → ids).
/// - Each page keeps its source frontmatter fields (merged with `title`,
///   `type`, `wiki_id`) and body, and gains a `## Graph Links` section with
///   one `[[path/to/target]]` line per outbound edge. Provenance (and edge
///   type) ride on each link as an HTML comment — the least intrusive place,
///   since Obsidian has no edge-attribute model.
/// - Non-destructive: creates directories and writes/overwrites the pages in
///   the graph; never deletes anything in an existing vault.
/// - Refuses to export into the canonical wiki directory (equal or nested) to
///   preserve the pages-stay-canonical invariant.
pub fn export_obsidian(
    graph: &StableGraph<WikiPageMeta, GraphEdge>,
    wiki_dir: &Path,
    out_dir: &Path,
) -> io::Result<ObsidianExport> {
    let out_canon = canonical_or_abs(out_dir);
    let wiki_canon = canonical_or_abs(wiki_dir);
    if out_canon == wiki_canon || out_canon.starts_with(&wiki_canon) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "refusing to export Obsidian vault into the canonical wiki directory {} — pages must stay canonical; choose a different --out",
                wiki_dir.display()
            ),
        ));
    }

    std::fs::create_dir_all(out_dir)?;

    let mut pages = 0usize;
    let mut wikilinks = 0usize;

    for idx in graph.node_indices() {
        let meta = &graph[idx];
        let rel = id_to_rel_path(&meta.id);
        let file_path = out_dir.join(&rel).with_extension("md");
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let source_content = match std::fs::read_to_string(wiki_dir.join(&rel).with_extension("md"))
        {
            Ok(content) => content,
            Err(e) => {
                tracing::warn!(
                    "Obsidian export: cannot read source page {} ({}): {} — writing stub body",
                    meta.id,
                    wiki_dir.join(&rel).with_extension("md").display(),
                    e
                );
                String::new()
            }
        };
        let (_, body) = crate::parser::extract_raw_frontmatter(&source_content);

        let mut out = render_frontmatter(meta, &source_content);
        out.push('\n');
        if !body.trim().is_empty() {
            out.push_str(body.trim());
            out.push_str("\n\n");
        }
        out.push_str("## Graph Links\n\n");

        let mut seen: HashSet<(String, String, String)> = HashSet::new();
        for edge in graph.edges(idx) {
            let target_meta = &graph[edge.target()];
            let target_rel = id_to_rel_path(&target_meta.id)
                .to_string_lossy()
                .into_owned();
            let weight = edge.weight();
            let edge_type = edge_type_str(&weight.edge_type);
            let provenance = weight.provenance.as_str();
            if seen.insert((
                target_rel.clone(),
                edge_type.clone(),
                provenance.to_string(),
            )) {
                out.push_str(&format!(
                    "- [[{}]] <!-- {} / {} -->\n",
                    target_rel, edge_type, provenance
                ));
                wikilinks = wikilinks.wrapping_add(1);
            }
        }

        std::fs::write(&file_path, out)?;
        pages = pages.wrapping_add(1);
    }

    Ok(ObsidianExport { pages, wikilinks })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Fixture wiki mirroring the provenance test in graph/mod.rs:
    /// explicit frontmatter edge, explicit body-ref edge, and an ambiguous
    /// resolution. No reciprocal backlink is stored.
    fn fixture_wiki() -> (TempDir, StableGraph<WikiPageMeta, GraphEdge>) {
        let tmp = TempDir::new().unwrap();
        let wiki_dir = tmp.path().join(".wm").join("wiki");
        std::fs::create_dir_all(wiki_dir.join("concepts")).unwrap();
        std::fs::create_dir_all(wiki_dir.join("patterns")).unwrap();

        std::fs::write(
            wiki_dir.join("concepts/author-source.md"),
            r#"---
title: Author Source
type: concept
relates_to:
  - type: references
    target: wiki:patterns:author-target
---

Authored frontmatter edge.
"#,
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("patterns/author-target.md"),
            "See @wiki/concepts/recip-source for details.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("concepts/recip-source.md"),
            "# Recip Source\n\nPlain page.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("concepts/ambig-a.md"),
            "# Ambig A\n\nFirst ambiguous candidate.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("patterns/ambig-a.md"),
            "# Ambig A\n\nSecond ambiguous candidate.\n",
        )
        .unwrap();

        std::fs::write(
            wiki_dir.join("concepts/ambig-source.md"),
            r#"---
relates_to:
  - type: references
    target: ambig-a
---

Intentionally ambiguous short target.
"#,
        )
        .unwrap();

        let graph = build_fixture_graph(&wiki_dir);
        (tmp, graph)
    }

    fn build_fixture_graph(wiki_dir: &Path) -> StableGraph<WikiPageMeta, GraphEdge> {
        crate::graph::build_graph_from_wiki(wiki_dir, &[]).0
    }

    fn edge_exists_with(
        graph: &StableGraph<WikiPageMeta, GraphEdge>,
        from: &str,
        to: &str,
        edge_type: &str,
        provenance: &str,
    ) -> bool {
        graph.edge_indices().any(|e| {
            let (s, t) = graph.edge_endpoints(e).unwrap();
            let w = graph.edge_weight(e).unwrap();
            graph[s].id == from
                && graph[t].id == to
                && edge_type_str(&w.edge_type) == edge_type
                && w.provenance.as_str() == provenance
        })
    }

    #[test]
    fn test_json_export_matches_graph_and_schema() {
        let (_tmp, graph) = fixture_wiki();
        let payload = graph_to_json(&graph);

        assert_eq!(payload["success"], serde_json::json!(true));
        let nodes = payload["nodes"].as_array().expect("nodes array");
        let edges = payload["edges"].as_array().expect("edges array");
        assert_eq!(
            payload["node_count"].as_u64().unwrap() as usize,
            graph.node_count()
        );
        assert_eq!(
            payload["edge_count"].as_u64().unwrap() as usize,
            graph.edge_count()
        );
        assert_eq!(nodes.len(), graph.node_count());
        assert_eq!(edges.len(), graph.edge_count());

        let mut node_ids: Vec<&str> = nodes
            .iter()
            .map(|n| {
                assert!(n["id"].is_string());
                assert!(n["title"].is_string());
                assert!(n["page_type"].is_string());
                n["id"].as_str().unwrap()
            })
            .collect();
        node_ids.sort_unstable();
        let mut expected_ids: Vec<&str> =
            graph.node_indices().map(|i| graph[i].id.as_str()).collect();
        expected_ids.sort_unstable();
        assert_eq!(node_ids, expected_ids);

        for e in edges {
            let source = e["source"].as_str().unwrap();
            let target = e["target"].as_str().unwrap();
            let edge_type = e["edge_type"].as_str().unwrap();
            let provenance = e["provenance"].as_str().unwrap();
            assert!(
                ["explicit", "derived", "ambiguous"].contains(&provenance),
                "provenance must be one of explicit|derived|ambiguous, got {provenance}"
            );
            assert!(
                edge_exists_with(&graph, source, target, edge_type, provenance),
                "exported edge {source} -> {target} ({edge_type}/{provenance}) missing from graph"
            );
        }

        let explicit = edges
            .iter()
            .find(|e| {
                e["source"] == "wiki:concepts:author-source"
                    && e["target"] == "wiki:patterns:author-target"
            })
            .expect("frontmatter edge present");
        assert_eq!(explicit["provenance"], "explicit");

        let reciprocal = edges.iter().find(|e| {
            e["source"] == "wiki:concepts:recip-source"
                && e["target"] == "wiki:patterns:author-target"
        });
        assert!(
            reciprocal.is_none(),
            "reciprocal backlink must not be exported as a stored edge"
        );
        assert!(
            edges.iter().all(|e| e["provenance"] != "derived"),
            "no stored derived edges may exist in the export"
        );

        assert_eq!(edges.len(), graph.edge_count());
        assert!(node_ids
            .iter()
            .all(|id| graph.node_indices().any(|i| &graph[i].id == id)));
    }

    #[test]
    fn test_graphml_is_well_formed_and_matches_graph() {
        let (_tmp, graph) = fixture_wiki();
        let xml = graph_to_graphml(&graph);

        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<graphml"));
        assert!(xml.contains("<graph id=\"wiki-mem\" edgedefault=\"directed\">"));
        assert!(xml.ends_with("</graphml>\n"));
        assert_eq!(xml.matches("<graphml").count(), 1);

        assert!(xml.contains("key=\"provenance\""));
        assert!(xml.contains("key=\"edge_type\""));
        assert!(xml.contains("key=\"title\""));
        assert!(xml.contains("key=\"page_type\""));

        let node_ids: Vec<String> = graph.node_indices().map(|i| graph[i].id.clone()).collect();
        assert_eq!(node_ids.len(), graph.node_count());
        for id in &node_ids {
            let needle = format!("<node id=\"{}\">", xml_escape(id));
            assert_eq!(
                xml.matches(&needle).count(),
                1,
                "node {id} must appear exactly once"
            );
        }
        assert_eq!(xml.matches("<node id=\"").count(), graph.node_count());

        assert_eq!(xml.matches("<edge id=\"").count(), graph.edge_count());
        for edge_idx in graph.edge_indices() {
            let (s, t) = graph.edge_endpoints(edge_idx).unwrap();
            let needle = format!(
                "source=\"{}\" target=\"{}\"",
                xml_escape(&graph[s].id),
                xml_escape(&graph[t].id)
            );
            assert!(xml.contains(&needle), "edge {s:?}->{t:?} missing: {needle}");
        }

        assert!(xml.contains("<data key=\"provenance\">explicit</data>"));
        assert!(xml.contains("<data key=\"provenance\">ambiguous</data>"));
        assert!(
            !xml.contains("<data key=\"provenance\">derived</data>"),
            "stored derived reciprocal edges must not exist"
        );
    }

    #[test]
    fn test_obsidian_export_vault_and_wikilinks() {
        let (tmp, graph) = fixture_wiki();
        let wiki_dir = tmp.path().join(".wm").join("wiki");
        let vault = tmp.path().join("vault");

        std::fs::create_dir_all(vault.join("concepts")).unwrap();
        std::fs::write(vault.join("concepts/stray.md"), "# Stray\n").unwrap();

        let result = export_obsidian(&graph, &wiki_dir, &vault).unwrap();
        assert_eq!(result.pages, graph.node_count());

        assert!(vault.join("concepts/stray.md").exists());

        let author_source =
            std::fs::read_to_string(vault.join("concepts/author-source.md")).unwrap();
        assert!(author_source.starts_with("---\n"), "frontmatter opener");
        assert!(author_source.contains("\ntitle: Author Source\n"));
        assert!(author_source.contains("\ntype: concept\n"));
        assert!(author_source.contains("\nwiki_id: wiki:concepts:author-source\n"));
        assert!(author_source.contains("Authored frontmatter edge."));
        assert!(
            author_source.contains("- [[patterns/author-target]] <!-- references / explicit -->"),
            "wikilink with provenance comment missing:\n{author_source}"
        );

        let recip_source = std::fs::read_to_string(vault.join("concepts/recip-source.md")).unwrap();
        assert!(
            !recip_source.contains("- [["),
            "plain recipient page must not carry wikilinks, got:\n{recip_source}"
        );

        let author_target =
            std::fs::read_to_string(vault.join("patterns/author-target.md")).unwrap();
        assert!(
            author_target.contains("- [[concepts/recip-source]] <!-- references / explicit -->"),
            "authored body-ref wikilink missing:\n{author_target}"
        );

        let ambig_source = std::fs::read_to_string(vault.join("concepts/ambig-source.md")).unwrap();
        let link_line = ambig_source
            .lines()
            .find(|l| l.trim_start().starts_with("- [["))
            .expect("ambiguous edge link");
        assert!(
            link_line.contains("<!-- references / ambiguous -->"),
            "ambiguous provenance missing: {link_line}"
        );
        assert!(
            link_line.contains("[[concepts/ambig-a]]")
                || link_line.contains("[[patterns/ambig-a]]"),
            "ambiguous target must be one of the candidates: {link_line}"
        );

        assert_eq!(result.wikilinks, graph.edge_count());

        let vault2 = tmp.path().join("vault2");
        let result2 = export_obsidian(&graph, &wiki_dir, &vault2).unwrap();
        assert_eq!(result2, result);
        for page in graph.node_indices() {
            let rel = id_to_rel_path(&graph[page].id);
            let a = std::fs::read_to_string(vault.join(&rel).with_extension("md")).unwrap();
            let b = std::fs::read_to_string(vault2.join(&rel).with_extension("md")).unwrap();
            assert_eq!(
                a,
                b,
                "re-export must be byte-identical for {}",
                rel.display()
            );
        }
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(
            xml_escape("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }

    #[test]
    fn test_obsidian_export_refuses_wiki_dir_collision() {
        let (tmp, graph) = fixture_wiki();
        let wiki_dir = tmp.path().join(".wm").join("wiki");

        let err = export_obsidian(&graph, &wiki_dir, &wiki_dir).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);

        let nested = wiki_dir.join("export-vault");
        let err2 = export_obsidian(&graph, &wiki_dir, &nested).unwrap_err();
        assert_eq!(err2.kind(), io::ErrorKind::InvalidInput);

        let sibling = tmp.path().join("vault-sibling");
        assert!(export_obsidian(&graph, &wiki_dir, &sibling).is_ok());
    }
}
