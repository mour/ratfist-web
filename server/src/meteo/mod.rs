use rocket::Route;
use rocket_contrib::json::Json;

use std::error::Error;
use std::fmt::Display;

pub mod fetcher;
mod immediate;
pub mod models;
pub mod node;
pub mod schema;
mod stored;

#[derive(Debug)]
pub struct MeteoError;

impl Display for MeteoError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "{:?}", Self)
    }
}

impl Error for MeteoError {}

type MeteoResponse<T> = Result<Json<T>, MeteoError>;

pub fn get_routes() -> Vec<Route> {
    routes![
        immediate::query_current_values,
        stored::get_stored_values,
        stored::get_global_structure,
    ]
}
