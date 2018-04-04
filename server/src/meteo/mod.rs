
use rocket::Route;

mod immediate;

pub fn get_routes() -> Vec<Route> {
    routes![
        immediate::query_current_pressure,
        immediate::query_current_temperature,
        immediate::query_current_humidity,
        immediate::query_current_light_level,
    ]
}
