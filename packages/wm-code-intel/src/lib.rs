pub mod config_types;
pub(crate) mod helpers;
pub mod models;
pub mod services;

pub use models::*;
pub use services::{
    extract_deps, extract_edges, extract_symbols, infer_language_from_ext, load_lsp_config,
    CodeIntelEngine,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_parser_basic() {
        let source = "fn hello() {}";
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        let root_kind = root.kind();
        let child_count = root.child_count();
        if root_kind != "source_file" || child_count == 0 {
            panic!(
                "Rust parser failed: kind={}, children={}",
                root_kind, child_count
            );
        }
        let query = tree_sitter::Query::new(
            &tree_sitter_rust::LANGUAGE.into(),
            "(function_item name: (identifier) @name)",
        )
        .unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        use streaming_iterator::StreamingIterator;
        let mut matches = cursor.matches(&query, root, source.as_bytes());
        let mut count = 0;
        while matches.next().is_some() {
            count += 1;
        }
        assert_eq!(count, 1, "Should find 1 function, found {}", count);
    }

    #[test]
    fn test_rust_functions_and_structs() {
        let source = r#"
pub fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub struct User {
    name: String,
    age: u32,
}

pub enum Status {
    Active,
    Inactive,
}

pub trait Runnable {
    fn run(&self);
}

impl Runnable for User {
    fn run(&self) {
        println!("running");
    }
}

pub const MAX_RETRIES: u32 = 3;

mod utils;

pub type Callback = Box<dyn Fn()>;

macro_rules! define_impl {
    () => {};
}
"#;
        let syms = extract_symbols(source, "test.rs", "rs");
        assert!(!syms.is_empty(), "Should find symbols in Rust source");

        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"hello"), "Should find function hello");
        assert!(names.contains(&"User"), "Should find struct User");
        assert!(names.contains(&"Status"), "Should find enum Status");
        assert!(names.contains(&"Runnable"), "Should find trait Runnable");
        assert!(
            names.contains(&"MAX_RETRIES"),
            "Should find const MAX_RETRIES"
        );
        assert!(names.contains(&"Callback"), "Should find type Callback");
        assert!(names.contains(&"utils"), "Should find module utils");

        let kinds: Vec<&str> = syms.iter().map(|s| s.kind.as_str()).collect();
        assert!(kinds.contains(&"function"), "Should include function kind");
        assert!(kinds.contains(&"struct"), "Should include struct kind");
        assert!(kinds.contains(&"enum"), "Should include enum kind");
        assert!(kinds.contains(&"trait"), "Should include trait kind");
        assert!(kinds.contains(&"const"), "Should include const kind");
        assert!(kinds.contains(&"type"), "Should include type kind");
        assert!(kinds.contains(&"module"), "Should include module kind");
    }

    #[test]
    fn test_rust_deps() {
        let source = r#"
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::engine::EngineState;
"#;
        let deps = extract_deps(source, "rs");
        assert_eq!(deps.len(), 3, "Should find 3 use declarations");
        assert_eq!(deps[0].target, "std::collections::HashMap");
        assert_eq!(deps[1].target, "serde::{Serialize, Deserialize}");
        assert_eq!(deps[2].target, "crate::engine::EngineState");
    }

    #[test]
    fn test_typescript_symbols() {
        let source = r#"
function greet(name: string): string {
    return `Hello, ${name}`;
}

class Person {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
    sayHello(): void {}
}

interface Talker {
    talk(): void;
}

type Point = {
    x: number;
    y: number;
};

enum Color {
    Red,
    Green,
    Blue,
}
"#;
        let syms = extract_symbols(source, "test.ts", "ts");
        assert!(!syms.is_empty(), "Should find symbols in TS source");

        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"greet"), "Should find function greet");
        assert!(names.contains(&"Person"), "Should find class Person");
        assert!(names.contains(&"Talker"), "Should find interface Talker");
        assert!(names.contains(&"Point"), "Should find type Point");
        assert!(names.contains(&"Color"), "Should find enum Color");
        assert!(names.contains(&"sayHello"), "Should find method sayHello");
    }

    #[test]
    fn test_typescript_deps() {
        let source = r#"
import { Component } from '@angular/core';
import * as fs from 'fs';
import('./lazy').then(m => m.run());
"#;
        let deps = extract_deps(source, "ts");
        assert!(deps.len() >= 2, "Should find import declarations");
        assert!(deps.iter().any(|d| d.target.contains("@angular/core")));
        assert!(deps.iter().any(|d| d.target.contains("fs")));
    }

    #[test]
    fn test_python_symbols() {
        let source = r#"
def hello(name: str) -> str:
    return f"Hello, {name}"

class Person:
    def __init__(self, name: str):
        self.name = name

    def greet(self) -> str:
        return f"Hi, {self.name}"

@dataclass
class Config:
    debug: bool = False
"#;
        let syms = extract_symbols(source, "test.py", "py");
        assert!(!syms.is_empty(), "Should find symbols in Python source");

        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"hello"), "Should find function hello");
        assert!(names.contains(&"Person"), "Should find class Person");
        assert!(names.contains(&"Config"), "Should find class Config");
    }

    #[test]
    fn test_python_deps() {
        let source = r#"
import os
import sys
from datetime import datetime
"#;
        let deps = extract_deps(source, "py");
        assert_eq!(deps.len(), 3, "Should find 3 import declarations");
        assert!(deps.iter().any(|d| d.target == "os"));
        assert!(deps.iter().any(|d| d.target == "datetime"));
    }

    #[test]
    fn test_go_symbols() {
        let source = r#"
package main

func hello(name string) string {
    return "Hello, " + name
}

type User struct {
    Name string
    Age  int
}

type Reader interface {
    Read(p []byte) (n int, err error)
}

func (u *User) Greet() string {
    return "Hi, " + u.Name
}
"#;
        let syms = extract_symbols(source, "test.go", "go");
        assert!(!syms.is_empty(), "Should find symbols in Go source");

        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"hello"), "Should find function hello");
        assert!(names.contains(&"User"), "Should find type User");
        assert!(
            names.contains(&"Reader"),
            "Should find type Reader (interface)"
        );
        assert!(names.contains(&"Greet"), "Should find method Greet");
    }

    #[test]
    fn test_go_deps() {
        let source = r#"
import (
    "fmt"
    "net/http"
    "github.com/gorilla/mux"
)
"#;
        let deps = extract_deps(source, "go");
        assert_eq!(deps.len(), 3, "Should find 3 import declarations");
        assert!(deps.iter().any(|d| d.target == "\"fmt\""));
    }

    #[test]
    fn test_html_structure() {
        let source = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
    <div class="container">
        <h1>Hello</h1>
        <p>World</p>
        <br />
    </div>
    <script>console.log("hi")</script>
</body>
</html>"#;
        let syms = extract_symbols(source, "test.html", "html");
        assert!(!syms.is_empty(), "Should find symbols in HTML source");
        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"html"), "Should find html element");
        assert!(names.contains(&"div"), "Should find div element");
        assert!(names.contains(&"h1"), "Should find h1 element");
    }

    #[test]
    fn test_svelte_structure() {
        let source = r#"<script>
    let count = 0;
    function increment() {
        count += 1;
    }
</script>

<main>
    <h1>Hello Svelte</h1>
    <button on:click={increment}>
        Clicked {count} times
    </button>
</main>

<style>
    h1 { color: red; }
</style>"#;
        let syms = extract_symbols(source, "test.svelte", "svelte");
        assert!(!syms.is_empty(), "Should find symbols in Svelte source");
        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"main"), "Should find main element");
        assert!(names.contains(&"h1"), "Should find h1 element");
        assert!(names.contains(&"button"), "Should find button element");
    }

    #[test]
    fn test_language_detection() {
        let labelled: &[(&str, &str)] = &[
            ("rs", "rust"),
            ("ts", "typescript"),
            ("tsx", "tsx"),
            ("js", "javascript"),
            ("mjs", "javascript"),
            ("cjs", "javascript"),
            ("jsx", "jsx"),
            ("py", "python"),
            ("go", "go"),
            ("html", "html"),
            ("htm", "html"),
            ("svelte", "svelte"),
            ("css", "css"),
            ("scss", "scss"),
            ("sass", "scss"),
            ("json", "json"),
            ("yaml", "yaml"),
            ("yml", "yaml"),
            ("toml", "toml"),
            ("md", "markdown"),
            ("mdx", "markdown"),
            ("sh", "bash"),
            ("bash", "bash"),
            ("zsh", "bash"),
            ("sql", "sql"),
            ("vue", "vue"),
            ("java", "java"),
            ("kt", "kotlin"),
            ("kts", "kotlin"),
            ("swift", "swift"),
            ("c", "c"),
            ("h", "c"),
            ("cpp", "cpp"),
            ("hpp", "cpp"),
            ("cc", "cpp"),
            ("cxx", "cpp"),
            ("rb", "ruby"),
            ("php", "php"),
            ("scala", "scala"),
            ("dart", "dart"),
            ("lua", "lua"),
            ("r", "r"),
            ("R", "r"),
            ("zig", "zig"),
            ("ex", "elixir"),
            ("exs", "elixir"),
            ("erl", "erlang"),
            ("clj", "clojure"),
            ("cljs", "clojure"),
            ("cljc", "clojure"),
        ];
        for (ext, expected) in labelled {
            assert_eq!(
                infer_language_from_ext(ext),
                Some(*expected),
                "extension {:?} should map to {:?}",
                ext,
                expected
            );
        }
        assert_eq!(infer_language_from_ext("unknown"), None);
        assert_eq!(infer_language_from_ext(""), None);
    }

    #[test]
    fn test_engine_supported_extensions() {
        let engine = CodeIntelEngine::global();
        let exts = engine.supported_extensions();
        assert!(exts.contains(&"rs"));
        assert!(exts.contains(&"ts"));
        assert!(exts.contains(&"py"));
        assert!(exts.contains(&"go"));
        assert!(exts.contains(&"html"));
        assert!(exts.contains(&"svelte"));
    }

    #[test]
    fn test_unsupported_extension_returns_empty() {
        let syms = extract_symbols("some content", "test.js", "js");
        assert!(syms.is_empty());

        let deps = extract_deps("some content", "js");
        assert!(deps.is_empty());
    }
}
