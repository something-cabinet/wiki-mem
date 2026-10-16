pub enum FrontmatterValue {
    Scalar(String),
    Id(String),
    Int(i64),
    List(Vec<String>),
    Nested(Vec<(&'static str, FrontmatterValue)>),
}
