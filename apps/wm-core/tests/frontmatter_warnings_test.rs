use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use tracing_subscriber::fmt::MakeWriter;
use wm_engine::parser::{extract_frontmatter, parse_wiki_page};

const INVALID_CATEGORY_PAGE: &str =
    "---\ntitle: Broken\ntype: rule\nstatus: active\ncategory: coding\n---\n\nBody.\n";
const MULTI_DOCUMENT_PAGE: &str = "---\n{}\ntags: [learning]\n---\n\nBody.\n";
const VALID_PAGE: &str = "---\ntitle: Fine\ntype: concept\nstatus: draft\n---\n\nBody.\n";
const REPEAT_PARSES: usize = 4;

#[derive(Clone, Default)]
struct CapturedLogs(Arc<Mutex<Vec<u8>>>);

struct CapturedWriter(Arc<Mutex<Vec<u8>>>);

impl Write for CapturedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for CapturedLogs {
    type Writer = CapturedWriter;

    fn make_writer(&'a self) -> Self::Writer {
        CapturedWriter(self.0.clone())
    }
}

impl CapturedLogs {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

fn capture<F: FnOnce()>(run: F) -> String {
    let logs = CapturedLogs::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(logs.clone())
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::WARN)
        .finish();
    tracing::subscriber::with_default(subscriber, run);
    logs.text()
}

fn lines_naming(output: &str, needle: &str) -> usize {
    output.lines().filter(|line| line.contains(needle)).count()
}

#[test]
fn broken_page_parsed_repeatedly_warns_once_naming_its_file() {
    let path = Path::new(".wm/wiki/rules/dedupe-repeat.md");
    let output = capture(|| {
        for _ in 0..REPEAT_PARSES {
            parse_wiki_page(path, INVALID_CATEGORY_PAGE);
        }
    });
    assert_eq!(lines_naming(&output, "dedupe-repeat.md"), 1, "{output}");
    assert!(
        output.contains("category"),
        "reason must name the field: {output}"
    );
}

#[test]
fn one_failed_parse_produces_exactly_one_line() {
    let path = Path::new(".wm/wiki/rules/single-line.md");
    let output = capture(|| {
        parse_wiki_page(path, INVALID_CATEGORY_PAGE);
    });
    assert_eq!(output.lines().count(), 1, "{output}");
}

#[test]
fn multi_document_frontmatter_produces_one_line_naming_the_file() {
    let path = Path::new(".wm/wiki/patterns/multi-document.md");
    let output = capture(|| {
        for _ in 0..REPEAT_PARSES {
            parse_wiki_page(path, MULTI_DOCUMENT_PAGE);
        }
    });
    assert_eq!(output.lines().count(), 1, "{output}");
    assert_eq!(lines_naming(&output, "multi-document.md"), 1, "{output}");
}

#[test]
fn each_distinct_broken_page_warns_once() {
    let first = Path::new(".wm/wiki/rules/distinct-first.md");
    let second = Path::new(".wm/wiki/rules/distinct-second.md");
    let output = capture(|| {
        for _ in 0..REPEAT_PARSES {
            parse_wiki_page(first, INVALID_CATEGORY_PAGE);
            parse_wiki_page(second, INVALID_CATEGORY_PAGE);
        }
    });
    assert_eq!(lines_naming(&output, "distinct-first.md"), 1, "{output}");
    assert_eq!(lines_naming(&output, "distinct-second.md"), 1, "{output}");
    assert_eq!(output.lines().count(), 2, "{output}");
}

#[test]
fn extract_frontmatter_without_a_source_never_warns() {
    let output = capture(|| {
        for _ in 0..REPEAT_PARSES {
            let (meta, _) = extract_frontmatter(INVALID_CATEGORY_PAGE);
            assert!(meta.is_none());
        }
    });
    assert!(output.is_empty(), "{output}");
}

#[test]
fn valid_page_produces_no_warning() {
    let path = Path::new(".wm/wiki/concepts/valid-page.md");
    let output = capture(|| {
        parse_wiki_page(path, VALID_PAGE);
    });
    assert!(output.is_empty(), "{output}");
}
