/// A typed frontmatter value routed through the single YAML builder choke
/// point (`build_frontmatter`) so scalar quoting stays consistent across every
/// string-built CREATE path.
///
/// Each variant maps to a reusable quoting primitive in `yaml_helper` rather
/// than hand-rolled quoting:
/// - [`Scalar`](FrontmatterValue::Scalar) quotes only when the value would
///   otherwise misparse (e.g. a title beginning with `[` or containing `:`).
/// - [`Id`](FrontmatterValue::Id) always double-quotes, so values like
///   `652e07` can never re-parse as scientific-notation floats.
/// - [`Int`](FrontmatterValue::Int) emits a bare integer scalar.
/// - [`List`](FrontmatterValue::List) emits an inline flow sequence with
///   per-element quoting.
/// - [`Nested`](FrontmatterValue::Nested) emits an indented sub-mapping whose
///   children follow the same rules.
pub enum FrontmatterValue {
    Scalar(String),
    Id(String),
    Int(i64),
    List(Vec<String>),
    Nested(Vec<(&'static str, FrontmatterValue)>),
}
