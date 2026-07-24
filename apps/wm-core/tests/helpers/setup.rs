pub fn setup_test_project() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().to_path_buf();

    let wm_dir = root.join(".wm");
    std::fs::create_dir_all(wm_dir.join("wiki")).expect("create .wm/wiki");
    std::fs::create_dir_all(wm_dir.join("sources")).expect("create .wm/sources");
    std::fs::create_dir_all(wm_dir.join("state")).expect("create .wm/state");
    std::fs::create_dir_all(root.join(".agents").join("skills")).expect("create .agents/skills");

    std::fs::create_dir_all(wm_dir.join("memory")).expect("create .wm/memory");

    for sub in &[
        "tasks", "specs", "concepts", "patterns", "decisions", "howto", "reference", "core",
    ] {
        std::fs::create_dir_all(wm_dir.join("wiki").join(sub)).expect("create wiki subdir");
    }

    let config = serde_json::json!({
        "project_name": "",
        "schema_version": 1,
        "embedding": {
            "model_name": "bge-small-en-v1.5",
            "dimensions": 384,
            "batch_size": 32
        },
        "permissions": { "preset": "read-write" },
        "custom_edge_types": [],
        "source_dirs": ["docs/", "specs/"],
        "source_extensions": ["md", "yaml", "txt"],
        "search": {
            "default_mode": "hybrid",
            "default_limit": 20,
            "rrf_k": 60
        }
    });
    std::fs::write(
        wm_dir.join("config.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .expect("write config.json");

    let agents = "# AGENTS.md — Wiki Memory Engine Agent Handbook\n\n## Wiki Conventions\n...\n";
    std::fs::write(wm_dir.join("AGENTS.md"), agents).expect("write AGENTS.md");

    (dir, root)
}
