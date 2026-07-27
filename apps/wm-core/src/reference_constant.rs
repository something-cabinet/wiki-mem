use regex::Regex;

pub(crate) static REFERENCE_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\B@wiki/([A-Za-z0-9_-]+)/([A-Za-z0-9_./-]+)").expect("valid reference regex")
});
