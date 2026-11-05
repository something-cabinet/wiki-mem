pub struct MigratedPage<'a> {
    pub path: &'a str,
    pub page_type: &'a str,
    pub before: &'a str,
    pub after: &'a str,
    pub steps: &'a [String],
}
