use std::path::Path;

use sha2::{Digest, Sha256};
use wm_constants::WM_DIR;

use crate::migrate::managed_files_model::{
    ManagedApplied, ManagedEntry, ManagedFilePlan, ManagedManifest, ManagedPlan, ManagedScope,
    ManagedStatus,
};

const MANIFEST_FILE: &str = "managed-files.json";
const SKILL_DIRS: [&str; 6] = [
    ".claude/skills",
    ".kiro/skills",
    ".opencode/skills",
    ".codex/skills",
    ".agent/skills",
    ".agents/skills",
];
const SKILL_TEMPLATE_PREFIX: &str = "skills/";
const SKILL_TEMPLATE_SUFFIX: &str = "/SKILL.md";
const SKILL_FILE: &str = "SKILL.md";
const FIXED_TARGETS: [(&str, &str, ManagedScope); 8] = [
    (
        ".kiro/steering/wiki-mem.md",
        "steering/wiki-mem.md",
        ManagedScope::Whole,
    ),
    ("WIKI-MEM.md", "shims/WIKI-MEM.md", ManagedScope::Whole),
    ("AGENTS.md", "shims/AGENTS.md", ManagedScope::Block),
    ("CLAUDE.md", "shims/CLAUDE.md", ManagedScope::Block),
    ("GEMINI.md", "shims/GEMINI.md", ManagedScope::Block),
    ("REASONIX.md", "shims/REASONIX.md", ManagedScope::Block),
    ("OPENCODE.md", "shims/OPENCODE.md", ManagedScope::Block),
    (
        ".github/copilot-instructions.md",
        "shims/copilot-instructions.md",
        ManagedScope::Block,
    ),
];

struct ManagedTarget {
    path: String,
    key: String,
    scope: ManagedScope,
}

fn region(scope: ManagedScope, bytes: &[u8]) -> Option<&[u8]> {
    match scope {
        ManagedScope::Whole => Some(bytes),
        ManagedScope::Block => {
            let text = std::str::from_utf8(bytes).ok()?;
            let (start, end) = block_range(text)?;
            Some(&bytes[start..end])
        }
    }
}

fn splice_block(disk: &[u8], template: &[u8]) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(disk).ok()?;
    let (start, end) = block_range(text)?;
    let mut spliced = Vec::with_capacity(disk.len());
    spliced.extend_from_slice(&disk[..start]);
    spliced.extend_from_slice(template);
    spliced.extend_from_slice(&disk[end..]);
    Some(spliced)
}

const BLOCK_START: &str = "<!-- WIKI-MEM GUIDELINES START -->";
const BLOCK_END: &str = "<!-- WIKI-MEM GUIDELINES END -->";

fn block_range(text: &str) -> Option<(usize, usize)> {
    let start = text.find(BLOCK_START)?;
    let end_marker = text[start..].find(BLOCK_END)?;
    Some((start, start + end_marker + BLOCK_END.len()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn current_entry(template: &[u8]) -> ManagedEntry {
    ManagedEntry {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        sha256: sha256_hex(template),
    }
}

fn template_bytes(key: &str) -> Option<Vec<u8>> {
    wm_core::embed_files::EmbeddedFiles::get(key).map(|file| file.data.to_vec())
}

fn skill_names() -> Vec<String> {
    let mut names: Vec<String> = wm_core::embed_files::EmbeddedFiles::iter()
        .filter_map(|path| {
            path.strip_prefix(SKILL_TEMPLATE_PREFIX)
                .and_then(|rest| rest.strip_suffix(SKILL_TEMPLATE_SUFFIX))
                .map(str::to_owned)
        })
        .collect();
    names.sort();
    names
}

fn managed_targets() -> Vec<ManagedTarget> {
    let mut targets: Vec<ManagedTarget> = FIXED_TARGETS
        .iter()
        .map(|(path, key, scope)| ManagedTarget {
            path: (*path).to_owned(),
            key: (*key).to_owned(),
            scope: *scope,
        })
        .collect();
    for name in skill_names() {
        let key = format!("{SKILL_TEMPLATE_PREFIX}{name}{SKILL_TEMPLATE_SUFFIX}");
        for dir in SKILL_DIRS {
            targets.push(ManagedTarget {
                path: format!("{dir}/{name}/{SKILL_FILE}"),
                key: key.clone(),
                scope: ManagedScope::Whole,
            });
        }
    }
    targets
}

fn manifest_path(root: &Path) -> std::path::PathBuf {
    root.join(WM_DIR).join(MANIFEST_FILE)
}

pub fn load_manifest(root: &Path) -> Result<ManagedManifest, anyhow::Error> {
    let path = manifest_path(root);
    if !path.exists() {
        return Ok(ManagedManifest::default());
    }
    let text = std::fs::read_to_string(&path)?;
    serde_json::from_str(&text)
        .map_err(|error| anyhow::anyhow!("Invalid {}: {}", path.display(), error))
}

fn save_manifest(root: &Path, manifest: &ManagedManifest) -> Result<(), anyhow::Error> {
    let path = manifest_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(manifest)?)?;
    Ok(())
}

fn classify(disk: &[u8], template: &[u8], entry: Option<&ManagedEntry>) -> ManagedStatus {
    let disk_hash = sha256_hex(disk);
    let wanted = current_entry(template);
    if disk_hash == wanted.sha256 {
        return match entry == Some(&wanted) {
            true => ManagedStatus::UpToDate,
            false => ManagedStatus::Record,
        };
    }
    match entry {
        Some(known) if known.sha256 == disk_hash => ManagedStatus::Update,
        Some(_) => ManagedStatus::Edited,
        None => ManagedStatus::Unmanaged,
    }
}

pub fn plan_managed_files(root: &Path) -> Result<ManagedPlan, anyhow::Error> {
    let manifest = load_manifest(root)?;
    let mut plan = ManagedPlan::default();
    for target in managed_targets() {
        let Ok(disk) = std::fs::read(root.join(&target.path)) else {
            continue;
        };
        let Some(template_full) = template_bytes(&target.key) else {
            continue;
        };
        let Some(template) = region(target.scope, &template_full).map(<[u8]>::to_vec) else {
            continue;
        };
        let status = match region(target.scope, &disk) {
            Some(current) => classify(current, &template, manifest.files.get(&target.path)),
            None => ManagedStatus::Unmanaged,
        };
        plan.files.push(ManagedFilePlan {
            path: target.path,
            status,
            scope: target.scope,
            template,
        });
    }
    Ok(plan)
}

fn write_update(root: &Path, file: &ManagedFilePlan) -> Result<bool, anyhow::Error> {
    let path = root.join(&file.path);
    let content = match file.scope {
        ManagedScope::Whole => Some(file.template.clone()),
        ManagedScope::Block => splice_block(&std::fs::read(&path)?, &file.template),
    };
    let Some(content) = content else {
        return Ok(false);
    };
    std::fs::write(path, content)?;
    Ok(true)
}

pub fn apply_managed_plan(
    root: &Path,
    plan: &ManagedPlan,
) -> Result<ManagedApplied, anyhow::Error> {
    let mut manifest = load_manifest(root)?;
    let mut applied = ManagedApplied::default();
    for file in &plan.files {
        match file.status {
            ManagedStatus::Record => {
                manifest
                    .files
                    .insert(file.path.clone(), current_entry(&file.template));
                applied.recorded = applied.recorded.saturating_add(1);
            }
            ManagedStatus::Update => {
                if write_update(root, file)? {
                    manifest
                        .files
                        .insert(file.path.clone(), current_entry(&file.template));
                    applied.updated = applied.updated.saturating_add(1);
                }
            }
            ManagedStatus::UpToDate | ManagedStatus::Edited | ManagedStatus::Unmanaged => {}
        }
    }
    if applied != ManagedApplied::default() {
        save_manifest(root, &manifest)?;
    }
    Ok(applied)
}

pub fn record_managed_files(root: &Path) -> Result<(), anyhow::Error> {
    let plan = plan_managed_files(root)?;
    apply_managed_plan(root, &plan)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrate::managed_files_model::ManagedStatus;

    const STEERING_PATH: &str = ".kiro/steering/wiki-mem.md";
    const AGENTS_PATH: &str = "AGENTS.md";
    const AGENTS_TEMPLATE: &str = "shims/AGENTS.md";
    const STEERING_TEMPLATE: &str = "steering/wiki-mem.md";
    const WIKI_MEM_PATH: &str = "WIKI-MEM.md";
    const SKILL_PATH: &str = ".claude/skills/wm-init/SKILL.md";
    const SKILL_TEMPLATE: &str = "skills/wm-init/SKILL.md";
    const OLD_CONTENT: &[u8] = b"old template text";
    const EDITED_CONTENT: &[u8] = b"user edited text";

    fn block_of(bytes: &[u8]) -> String {
        let text = std::str::from_utf8(bytes).expect("utf8");
        let (start, end) = block_range(text).expect("block markers");
        text[start..end].to_owned()
    }

    fn status_of(plan: &ManagedPlan, path: &str) -> Option<ManagedStatus> {
        plan.files
            .iter()
            .find(|file| file.path == path)
            .map(|file| file.status)
    }

    fn template(key: &str) -> Vec<u8> {
        wm_core::embed_files::EmbeddedFiles::get(key)
            .expect("embedded template")
            .data
            .to_vec()
    }

    fn write(root: &Path, relative: &str, content: &[u8]) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(path, content).expect("write");
    }

    fn read(root: &Path, relative: &str) -> Vec<u8> {
        std::fs::read(root.join(relative)).expect("read")
    }

    fn sha(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        hex::encode(Sha256::digest(bytes))
    }

    fn write_manifest(root: &Path, entries: &[(&str, &[u8], &str)]) {
        let files: serde_json::Map<String, serde_json::Value> = entries
            .iter()
            .map(|(path, content, version)| {
                (
                    (*path).to_owned(),
                    serde_json::json!({ "version": version, "sha256": sha(content) }),
                )
            })
            .collect();
        write(
            root,
            ".wm/managed-files.json",
            serde_json::json!({ "files": files }).to_string().as_bytes(),
        );
    }

    #[test]
    fn an_untracked_file_equal_to_the_template_is_recorded() {
        let temp = tempfile::tempdir().expect("tempdir");
        write(temp.path(), STEERING_PATH, &template(STEERING_TEMPLATE));

        let plan = plan_managed_files(temp.path()).expect("plan");
        let applied = apply_managed_plan(temp.path(), &plan).expect("apply");

        assert_eq!(status_of(&plan, STEERING_PATH), Some(ManagedStatus::Record));
        assert_eq!(applied.recorded, 1);
        let manifest = load_manifest(temp.path()).expect("manifest");
        assert_eq!(
            manifest.files[STEERING_PATH].sha256,
            sha(&template(STEERING_TEMPLATE))
        );
        let again = plan_managed_files(temp.path()).expect("plan again");
        assert_eq!(
            status_of(&again, STEERING_PATH),
            Some(ManagedStatus::UpToDate)
        );
    }

    #[test]
    fn a_pristine_old_file_is_updated_to_the_current_template() {
        let temp = tempfile::tempdir().expect("tempdir");
        write(temp.path(), STEERING_PATH, OLD_CONTENT);
        write_manifest(temp.path(), &[(STEERING_PATH, OLD_CONTENT, "0.1.0")]);

        let plan = plan_managed_files(temp.path()).expect("plan");
        let applied = apply_managed_plan(temp.path(), &plan).expect("apply");

        assert_eq!(status_of(&plan, STEERING_PATH), Some(ManagedStatus::Update));
        assert_eq!(applied.updated, 1);
        assert_eq!(
            read(temp.path(), STEERING_PATH),
            template(STEERING_TEMPLATE)
        );
        let manifest = load_manifest(temp.path()).expect("manifest");
        assert_eq!(
            manifest.files[STEERING_PATH].version,
            env!("CARGO_PKG_VERSION")
        );
    }

    #[test]
    fn an_edited_file_is_kept() {
        let temp = tempfile::tempdir().expect("tempdir");
        write(temp.path(), STEERING_PATH, EDITED_CONTENT);
        write_manifest(temp.path(), &[(STEERING_PATH, OLD_CONTENT, "0.1.0")]);

        let plan = plan_managed_files(temp.path()).expect("plan");
        let applied = apply_managed_plan(temp.path(), &plan).expect("apply");

        assert_eq!(status_of(&plan, STEERING_PATH), Some(ManagedStatus::Edited));
        assert_eq!(applied, ManagedApplied::default());
        assert_eq!(read(temp.path(), STEERING_PATH), EDITED_CONTENT);
    }

    #[test]
    fn a_different_file_without_a_manifest_entry_is_unmanaged_and_kept() {
        let temp = tempfile::tempdir().expect("tempdir");
        write(temp.path(), WIKI_MEM_PATH, EDITED_CONTENT);

        let plan = plan_managed_files(temp.path()).expect("plan");
        let applied = apply_managed_plan(temp.path(), &plan).expect("apply");

        assert_eq!(
            status_of(&plan, WIKI_MEM_PATH),
            Some(ManagedStatus::Unmanaged)
        );
        assert_eq!(applied, ManagedApplied::default());
        assert_eq!(read(temp.path(), WIKI_MEM_PATH), EDITED_CONTENT);
    }

    #[test]
    fn a_shim_is_managed_by_its_guidelines_block_only() {
        let temp = tempfile::tempdir().expect("tempdir");
        let region = block_of(&template(AGENTS_TEMPLATE));
        let disk = format!("head\n{region}\n@extra-import\n");
        write(temp.path(), AGENTS_PATH, disk.as_bytes());

        let plan = plan_managed_files(temp.path()).expect("plan");
        apply_managed_plan(temp.path(), &plan).expect("apply");

        assert_eq!(status_of(&plan, AGENTS_PATH), Some(ManagedStatus::Record));
        let again = plan_managed_files(temp.path()).expect("plan again");
        assert_eq!(
            status_of(&again, AGENTS_PATH),
            Some(ManagedStatus::UpToDate)
        );
    }

    #[test]
    fn an_old_pristine_block_is_replaced_and_the_rest_of_the_shim_is_kept() {
        let temp = tempfile::tempdir().expect("tempdir");
        let old_block = format!("{BLOCK_START}\nold guidelines\n{BLOCK_END}");
        let disk = format!("head\n{old_block}\n@extra-import\n");
        write(temp.path(), AGENTS_PATH, disk.as_bytes());
        write_manifest(temp.path(), &[(AGENTS_PATH, old_block.as_bytes(), "0.1.0")]);

        let plan = plan_managed_files(temp.path()).expect("plan");
        let applied = apply_managed_plan(temp.path(), &plan).expect("apply");

        assert_eq!(status_of(&plan, AGENTS_PATH), Some(ManagedStatus::Update));
        assert_eq!(applied.updated, 1);
        let text = String::from_utf8(read(temp.path(), AGENTS_PATH)).expect("utf8");
        assert!(text.starts_with("head\n"), "{text}");
        assert!(text.ends_with("\n@extra-import\n"), "{text}");
        assert!(
            text.contains(&block_of(&template(AGENTS_TEMPLATE))),
            "{text}"
        );
        assert!(!text.contains("old guidelines"), "{text}");
    }

    #[test]
    fn an_edited_block_is_kept() {
        let temp = tempfile::tempdir().expect("tempdir");
        let old_block = format!("{BLOCK_START}\nold guidelines\n{BLOCK_END}");
        let edited = format!("{BLOCK_START}\nuser words\n{BLOCK_END}");
        write(temp.path(), AGENTS_PATH, edited.as_bytes());
        write_manifest(temp.path(), &[(AGENTS_PATH, old_block.as_bytes(), "0.1.0")]);

        let plan = plan_managed_files(temp.path()).expect("plan");
        apply_managed_plan(temp.path(), &plan).expect("apply");

        assert_eq!(status_of(&plan, AGENTS_PATH), Some(ManagedStatus::Edited));
        assert_eq!(read(temp.path(), AGENTS_PATH), edited.as_bytes());
    }

    #[test]
    fn a_shim_without_the_block_markers_is_unmanaged_and_kept() {
        let temp = tempfile::tempdir().expect("tempdir");
        write(temp.path(), AGENTS_PATH, b"my own instructions\n");

        let plan = plan_managed_files(temp.path()).expect("plan");
        apply_managed_plan(temp.path(), &plan).expect("apply");

        assert_eq!(
            status_of(&plan, AGENTS_PATH),
            Some(ManagedStatus::Unmanaged)
        );
        assert_eq!(read(temp.path(), AGENTS_PATH), b"my own instructions\n");
    }

    #[test]
    fn missing_files_are_ignored() {
        let temp = tempfile::tempdir().expect("tempdir");

        let plan = plan_managed_files(temp.path()).expect("plan");

        assert!(plan.files.is_empty());
        assert!(!temp.path().join(STEERING_PATH).exists());
    }

    #[test]
    fn skills_in_any_platform_directory_are_managed() {
        let temp = tempfile::tempdir().expect("tempdir");
        write(temp.path(), SKILL_PATH, &template(SKILL_TEMPLATE));
        write(
            temp.path(),
            ".opencode/skills/wm-init/SKILL.md",
            OLD_CONTENT,
        );

        let plan = plan_managed_files(temp.path()).expect("plan");

        assert_eq!(status_of(&plan, SKILL_PATH), Some(ManagedStatus::Record));
        assert_eq!(
            status_of(&plan, ".opencode/skills/wm-init/SKILL.md"),
            Some(ManagedStatus::Unmanaged)
        );
    }

    #[test]
    fn recording_after_setup_tracks_every_managed_file_on_disk() {
        let temp = tempfile::tempdir().expect("tempdir");
        write(temp.path(), STEERING_PATH, &template(STEERING_TEMPLATE));
        write(temp.path(), SKILL_PATH, &template(SKILL_TEMPLATE));

        record_managed_files(temp.path()).expect("record");

        let manifest = load_manifest(temp.path()).expect("manifest");
        assert!(manifest.files.contains_key(STEERING_PATH));
        assert!(manifest.files.contains_key(SKILL_PATH));
    }

    #[test]
    fn an_invalid_manifest_is_an_error() {
        let temp = tempfile::tempdir().expect("tempdir");
        write(temp.path(), ".wm/managed-files.json", b"not json");

        assert!(plan_managed_files(temp.path()).is_err());
    }
}
