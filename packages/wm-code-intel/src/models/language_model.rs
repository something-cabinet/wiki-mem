use tree_sitter::Language;

#[derive(Debug, Clone)]
pub(crate) enum SupportedLanguage {
    Rust,
    TypeScript,
    Tsx,
    Python,
    Go,
    Html,
    Svelte,
}

impl SupportedLanguage {
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "py" => Some(Self::Python),
            "go" => Some(Self::Go),
            "html" | "htm" => Some(Self::Html),
            "svelte" => Some(Self::Svelte),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::Python => "python",
            Self::Go => "go",
            Self::Html => "html",
            Self::Svelte => "svelte",
        }
    }

    pub fn load_language(&self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::Html => tree_sitter_html::LANGUAGE.into(),
            Self::Svelte => tree_sitter_svelte_ng::LANGUAGE.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum LanguageLabel {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
    Jsx,
    Python,
    Go,
    Html,
    Svelte,
    Css,
    Scss,
    Json,
    Yaml,
    Toml,
    Markdown,
    Bash,
    Sql,
    Vue,
    Java,
    Kotlin,
    Swift,
    C,
    Cpp,
    Ruby,
    Php,
    Scala,
    Dart,
    Lua,
    R,
    Zig,
    Elixir,
    Erlang,
    Clojure,
}

impl LanguageLabel {
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "jsx" => Some(Self::Jsx),
            "py" => Some(Self::Python),
            "go" => Some(Self::Go),
            "html" | "htm" => Some(Self::Html),
            "svelte" => Some(Self::Svelte),
            "css" => Some(Self::Css),
            "scss" | "sass" => Some(Self::Scss),
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "md" | "mdx" => Some(Self::Markdown),
            "sh" | "bash" | "zsh" => Some(Self::Bash),
            "sql" => Some(Self::Sql),
            "vue" => Some(Self::Vue),
            "java" => Some(Self::Java),
            "kt" | "kts" => Some(Self::Kotlin),
            "swift" => Some(Self::Swift),
            "c" | "h" => Some(Self::C),
            "cpp" | "hpp" | "cc" | "cxx" => Some(Self::Cpp),
            "rb" => Some(Self::Ruby),
            "php" => Some(Self::Php),
            "scala" => Some(Self::Scala),
            "dart" => Some(Self::Dart),
            "lua" => Some(Self::Lua),
            "r" | "R" => Some(Self::R),
            "zig" => Some(Self::Zig),
            "ex" | "exs" => Some(Self::Elixir),
            "erl" => Some(Self::Erlang),
            "clj" | "cljs" | "cljc" => Some(Self::Clojure),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::JavaScript => "javascript",
            Self::Jsx => "jsx",
            Self::Python => "python",
            Self::Go => "go",
            Self::Html => "html",
            Self::Svelte => "svelte",
            Self::Css => "css",
            Self::Scss => "scss",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Markdown => "markdown",
            Self::Bash => "bash",
            Self::Sql => "sql",
            Self::Vue => "vue",
            Self::Java => "java",
            Self::Kotlin => "kotlin",
            Self::Swift => "swift",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Ruby => "ruby",
            Self::Php => "php",
            Self::Scala => "scala",
            Self::Dart => "dart",
            Self::Lua => "lua",
            Self::R => "r",
            Self::Zig => "zig",
            Self::Elixir => "elixir",
            Self::Erlang => "erlang",
            Self::Clojure => "clojure",
        }
    }
}
