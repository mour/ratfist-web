use rocket::serde::json::Json;
use rocket::Route;

pub mod fetcher;
mod immediate;
pub mod models;
pub mod node;
pub mod schema;
mod stored;

use crate::utils::Result;

type MeteoResponse<T> = Result<Json<T>>;

pub fn get_routes() -> Vec<Route> {
    routes![
        immediate::query_current_values,
        stored::get_stored_values,
        stored::get_global_structure,
    ]
}
