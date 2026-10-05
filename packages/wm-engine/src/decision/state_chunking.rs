pub const MAX_INPUT_TOKENS: usize = 512;
pub const CHUNK_WORDS: usize = 384;
pub const CHUNK_OVERLAP_WORDS: usize = 64;

pub fn chunk_state(state: &str) -> Vec<String> {
    let words: Vec<&str> = state.split_whitespace().collect();
    if words.len() <= MAX_INPUT_TOKENS {
        return vec![state.to_owned()];
    }
    let step = CHUNK_WORDS.saturating_sub(CHUNK_OVERLAP_WORDS);
    if step == 0 {
        return vec![state.to_owned()];
    }

    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < words.len() {
        let end = start.saturating_add(CHUNK_WORDS).min(words.len());
        chunks.push(words[start..end].join(" "));
        if end == words.len() {
            break;
        }
        start = start.saturating_add(step);
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_state_is_a_single_chunk() {
        let chunks = chunk_state("## Context\n\nA short decision body.");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "## Context\n\nA short decision body.");
    }

    #[test]
    fn long_state_splits_into_overlapping_windows() {
        let words: Vec<String> = (0..MAX_INPUT_TOKENS + CHUNK_WORDS)
            .map(|index| format!("w{index}"))
            .collect();
        let state = words.join(" ");
        let chunks = chunk_state(&state);
        assert!(chunks.len() > 1, "long state must chunk");
        assert_eq!(chunks[0].split_whitespace().count(), CHUNK_WORDS);
        let first: Vec<&str> = chunks[0].split_whitespace().collect();
        let second: Vec<&str> = chunks[1].split_whitespace().collect();
        assert_eq!(
            &first[first.len() - CHUNK_OVERLAP_WORDS..],
            &second[..CHUNK_OVERLAP_WORDS],
            "consecutive windows must overlap"
        );
    }

    #[test]
    fn empty_state_is_a_single_empty_chunk() {
        assert_eq!(chunk_state(""), vec![String::new()]);
    }
}
