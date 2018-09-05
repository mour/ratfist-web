use super::schema::{nodes};

#[derive(Identifiable, Queryable, Debug, Clone)]
pub(super) struct Node {
    pub id: i32,
    pub public_id: i32,
    pub name: String,
}
