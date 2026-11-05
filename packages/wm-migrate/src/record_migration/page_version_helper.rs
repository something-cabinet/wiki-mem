const VERSION_KEY: &str = "schema_version:";
const UNVERSIONED: u32 = 0;

pub fn page_version(body: &str) -> u32 {
    body.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .and_then(|line| line.strip_prefix(VERSION_KEY))
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(UNVERSIONED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prose_body_has_no_version() {
        assert_eq!(page_version("## Context\n\nSome prose.\n"), UNVERSIONED);
    }

    #[test]
    fn empty_body_has_no_version() {
        assert_eq!(page_version(""), UNVERSIONED);
    }

    #[test]
    fn envelope_without_schema_version_has_no_version() {
        assert_eq!(
            page_version("state: |-\n  text\nanswers: {}\n"),
            UNVERSIONED
        );
    }

    #[test]
    fn reads_the_version_of_a_record() {
        assert_eq!(page_version("schema_version: 1\nstate: |-\n  text\n"), 1);
    }

    #[test]
    fn reads_a_higher_version() {
        assert_eq!(
            page_version("\n\nschema_version: 7\nstate: |-\n  text\n"),
            7
        );
    }

    #[test]
    fn a_non_numeric_version_counts_as_unversioned() {
        assert_eq!(
            page_version("schema_version: one\nstate: |-\n  text\n"),
            UNVERSIONED
        );
    }

    #[test]
    fn a_version_later_in_the_body_is_prose() {
        assert_eq!(
            page_version("## Notes\n\nschema_version: 1 is the record version\n"),
            UNVERSIONED
        );
    }

    #[test]
    fn the_key_constant_matches_the_envelope_line() {
        assert!(VERSION_KEY.ends_with(':'));
    }
}
