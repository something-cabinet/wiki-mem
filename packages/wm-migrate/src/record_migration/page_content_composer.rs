const DELIMITER: &str = "---";

pub fn compose_page(frontmatter: &str, body: &str) -> String {
    format!(
        "{DELIMITER}\n{frontmatter}\n{DELIMITER}\n\n{}\n",
        body.trim_end_matches('\n')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_the_frontmatter_and_separates_the_body() {
        let page = compose_page("title: One\ntype: concept", "schema_version: 1\n");

        assert_eq!(
            page,
            "---\ntitle: One\ntype: concept\n---\n\nschema_version: 1\n"
        );
    }

    #[test]
    fn trims_trailing_blank_lines_of_the_body() {
        let page = compose_page("title: One", "body\n\n\n");

        assert_eq!(page, "---\ntitle: One\n---\n\nbody\n");
    }
}
