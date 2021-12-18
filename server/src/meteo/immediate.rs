use rocket::serde::json::Json;
use rocket::State;

use super::models::SensorTypeEnum;
use super::node::SensorNodeRegistry;

use crate::utils::IdRange;

use crate::meteo::MeteoError;
use std::collections::HashMap;

#[get("/<node_id>/<sensor_type>/<sensor_ids>", format = "application/json")]
pub fn query_current_values(
    node_id: u32,
    sensor_type: SensorTypeEnum,
    sensor_ids: IdRange,
    node_registry: &State<SensorNodeRegistry>,
) -> Result<Json<HashMap<u32, f32>>, MeteoError> {
    let mut response_map = HashMap::new();

    for sensor_id in sensor_ids.iter() {
        let measured_val = node_registry
            .get_node(node_id)?
            .measure(sensor_type, *sensor_id)?;

        response_map.insert(*sensor_id, measured_val);
    }

    Ok(Json(response_map))
}
