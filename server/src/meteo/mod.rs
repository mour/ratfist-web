use rocket::Route;
use rocket_contrib::json::Json;

pub mod fetcher;
mod immediate;
mod messages;
pub mod models;
pub mod schema;
mod stored;

#[derive(Debug)]
pub struct MeteoError;

type MeteoResponse<T> = Result<Json<T>, MeteoError>;

pub fn get_routes() -> Vec<Route> {
    routes![
        immediate::query_all_sensors,
        immediate::query_current_values,
        stored::get_stored_values,
        stored::get_global_structure,
    ]
}
