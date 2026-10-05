use super::field_model::Field;

#[derive(Debug, Clone)]
pub struct IndexedDoc {
    pub id: String,
    pub fields: Vec<Field>,
}
