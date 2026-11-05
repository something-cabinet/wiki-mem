use super::skip_reason_model::SkipReason;

const EMPTY_MAP_LINE: &str = "{}";
const RELATES_TO_KEY: &str = "relates_to";
const TITLE_KEY: &str = "title";
const KEY_SEPARATOR: char = ':';

struct Entry {
    key: String,
    lines: Vec<String>,
}

pub fn merge_frontmatter_blocks(blocks: &[String]) -> Result<String, SkipReason> {
    if let [only] = blocks {
        return Ok(only.clone());
    }
    let parsed: Vec<Vec<Entry>> = blocks.iter().map(|block| entries_of(block)).collect();
    let base_index =
        base_block_index(&parsed).ok_or(SkipReason::UnparseableFrontmatter { block: 0 })?;
    let mut merged: Vec<Entry> = Vec::new();
    for entry in parsed.get(base_index).into_iter().flatten() {
        merged.push(Entry {
            key: entry.key.clone(),
            lines: entry.lines.clone(),
        });
    }
    for (index, entries) in parsed.iter().enumerate() {
        if index == base_index {
            continue;
        }
        for entry in entries {
            absorb(&mut merged, entry)?;
        }
    }
    Ok(render(&merged))
}

fn base_block_index(parsed: &[Vec<Entry>]) -> Option<usize> {
    let titled = parsed
        .iter()
        .position(|entries| entries.iter().any(|entry| entry.key == TITLE_KEY));
    titled.or_else(|| parsed.iter().position(|entries| !entries.is_empty()))
}

fn absorb(merged: &mut Vec<Entry>, incoming: &Entry) -> Result<(), SkipReason> {
    let Some(existing) = merged.iter_mut().find(|entry| entry.key == incoming.key) else {
        merged.push(Entry {
            key: incoming.key.clone(),
            lines: incoming.lines.clone(),
        });
        return Ok(());
    };
    if existing.lines == incoming.lines {
        return Ok(());
    }
    if incoming.key == RELATES_TO_KEY {
        union_items(existing, incoming);
        return Ok(());
    }
    Err(SkipReason::DuplicateFrontmatterConflict {
        key: incoming.key.clone(),
    })
}

fn union_items(existing: &mut Entry, incoming: &Entry) {
    for line in incoming.lines.iter().skip(1) {
        let known = existing
            .lines
            .iter()
            .any(|present| present.trim() == line.trim());
        if !known {
            existing.lines.push(line.clone());
        }
    }
}

fn entries_of(block: &str) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    for line in block.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.trim() == EMPTY_MAP_LINE || trimmed_end.trim().is_empty() {
            continue;
        }
        match key_of(trimmed_end) {
            Some(key) => entries.push(Entry {
                key,
                lines: vec![trimmed_end.to_owned()],
            }),
            None => {
                if let Some(last) = entries.last_mut() {
                    last.lines.push(trimmed_end.to_owned());
                }
            }
        }
    }
    entries
}

fn key_of(line: &str) -> Option<String> {
    let first = line.chars().next()?;
    if first.is_whitespace() || first == '-' || first == '#' {
        return None;
    }
    let (key, _) = line.split_once(KEY_SEPARATOR)?;
    Some(key.trim().to_owned())
}

fn render(entries: &[Entry]) -> String {
    entries
        .iter()
        .flat_map(|entry| entry.lines.iter())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    #[test]
    fn a_single_block_is_returned_unchanged() {
        let merged = merge_frontmatter_blocks(&blocks(&["title: One\ntype: concept"]));

        assert_eq!(merged, Ok("title: One\ntype: concept".to_owned()));
    }

    #[test]
    fn an_empty_map_stub_is_dropped() {
        let merged = merge_frontmatter_blocks(&blocks(&["{}", "title: One\ntype: concept"]));

        assert_eq!(merged, Ok("title: One\ntype: concept".to_owned()));
    }

    #[test]
    fn an_empty_stub_is_dropped() {
        let merged = merge_frontmatter_blocks(&blocks(&["", "title: One\ntype: concept"]));

        assert_eq!(merged, Ok("title: One\ntype: concept".to_owned()));
    }

    #[test]
    fn stub_keys_missing_from_the_real_block_are_kept() {
        let stub = "{}\ntags: [a, b]";
        let merged =
            merge_frontmatter_blocks(&blocks(&[stub, "title: One\ntype: concept"])).expect("merge");

        assert_eq!(merged, "title: One\ntype: concept\ntags: [a, b]");
    }

    #[test]
    fn task_acceptance_criteria_held_only_in_the_stub_survive() {
        let stub = "{}\nacceptance_criteria:\n  - {text: \"first\", checked: false}\n  - {text: \"second\", checked: true}";
        let real = "title: Task\ntype: task\nstatus: todo";
        let merged = merge_frontmatter_blocks(&blocks(&[stub, real])).expect("merge");

        assert_eq!(
            merged,
            "title: Task\ntype: task\nstatus: todo\nacceptance_criteria:\n  - {text: \"first\", checked: false}\n  - {text: \"second\", checked: true}"
        );
    }

    #[test]
    fn identical_blocks_collapse_to_one() {
        let block = "title: One\ntype: concept";
        let merged = merge_frontmatter_blocks(&blocks(&[block, block]));

        assert_eq!(merged, Ok(block.to_owned()));
    }

    #[test]
    fn relates_to_items_from_both_blocks_are_unioned() {
        let first = "title: One\nrelates_to:\n  - a";
        let second = "title: One\nrelates_to:\n  - a\n  - b";
        let merged = merge_frontmatter_blocks(&blocks(&[first, second])).expect("merge");

        assert_eq!(merged, "title: One\nrelates_to:\n  - a\n  - b");
    }

    #[test]
    fn a_scalar_that_differs_between_blocks_is_a_conflict() {
        let merged = merge_frontmatter_blocks(&blocks(&["title: One", "title: Two"]));

        assert_eq!(
            merged,
            Err(SkipReason::DuplicateFrontmatterConflict {
                key: "title".to_owned()
            })
        );
    }

    #[test]
    fn the_block_holding_the_title_is_the_base() {
        let stub = "tags: [a]\nstatus: draft";
        let real = "title: One\ntype: concept";
        let merged = merge_frontmatter_blocks(&blocks(&[stub, real])).expect("merge");

        assert_eq!(
            merged,
            "title: One\ntype: concept\ntags: [a]\nstatus: draft"
        );
    }

    #[test]
    fn only_empty_blocks_cannot_be_merged() {
        let merged = merge_frontmatter_blocks(&blocks(&["{}", ""]));

        assert_eq!(merged, Err(SkipReason::UnparseableFrontmatter { block: 0 }));
    }

    #[test]
    fn merging_the_merged_result_changes_nothing() {
        let stub = "{}\ntags: [a]";
        let real = "title: One\ntype: concept";
        let once = merge_frontmatter_blocks(&blocks(&[stub, real])).expect("merge");
        let twice = merge_frontmatter_blocks(&blocks(&[&once]));

        assert_eq!(twice, Ok(once));
    }
}
