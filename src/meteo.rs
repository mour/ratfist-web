
use rocket::Route;
use rocket::State;

use comm;

use utils::IdRange;

pub enum OutgoingMessage {
    GetPressure(u32),
    GetTemperature(u32),
    GetHumidity(u32),
    GetLightLevel(u32),
}

enum IncomingMessage {
    Pressure(u32, f32),
    Temperature(u32, f32),
    Humidity(u32, f32),
    RetVal(i32),
}

#[get("/<id_range>/pressure")]
fn query_pressure(id_range: IdRange, comm_state: State<comm::CommState>) -> String {
    format!("{:?}", id_range)
}

#[get("/<id_range>/temperature")]
fn query_temperature(id_range: IdRange, comm_state: State<comm::CommState>) -> String {
    format!("{:?}", id_range)
}

#[get("/<id_range>/humidity")]
fn query_humidity(id_range: IdRange, comm_state: State<comm::CommState>) -> String {
    format!("{:?}", id_range)
}

#[get("/<id_range>/light_level")]
fn query_light_level(id_range: IdRange, comm_state: State<comm::CommState>) -> String {
    format!("{:?}", id_range)
}


pub fn get_routes() -> Vec<Route> {
    routes![query_pressure,
            query_temperature,
            query_humidity,
            query_light_level]
}
