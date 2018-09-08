use super::schema::{nodes};

#[derive(Identifiable, Queryable, Debug, Clone)]
pub struct Node {
    pub id: i32,
    pub public_id: i32,
    pub name: String,
    pub route_type: String,
    pub route_param: Option<String>
}
