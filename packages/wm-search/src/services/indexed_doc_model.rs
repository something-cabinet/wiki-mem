use super::field_model::Field;

/// A single searchable document with weighted fields
#[derive(Clone, Debug)]
pub struct IndexedDoc {
    pub id: String,
    pub fields: Vec<Field>,
}
