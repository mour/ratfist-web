use std::collections::BTreeMap;
use std::convert::TryInto;
use std::sync::Arc;

use crate::db;
use crate::db::models::Node;

use super::models::SensorTypeEnum;
use super::MeteoError;

use diesel::prelude::*;

mod enviro_phat;
mod serial_node;

pub trait SensorNode: Sync + Send {
    fn measure(&self, measurement_type: SensorTypeEnum, sensor_id: u32) -> Result<f32, MeteoError>;
}

#[derive(Clone)]
pub struct SensorNodeRegistry {
    node_map: Arc<BTreeMap<u32, Arc<dyn SensorNode>>>,
}

impl SensorNodeRegistry {
    pub fn new(db_conn: db::Db) -> SensorNodeRegistry {
        let nodes = {
            use crate::db::schema::nodes;

            nodes::table
                .load::<Node>(&*db_conn)
                .expect("Error loading Node entries from DB.")
        };

        let mut node_map: BTreeMap<u32, Arc<dyn SensorNode>> = BTreeMap::new();

        for node in nodes {
            let public_id = node.public_id.try_into().unwrap();

            let sensor_node: Arc<dyn SensorNode> = match node.route_type.as_str() {
                "serial" => {
                    let route_param_str = node.route_param.unwrap_or_else(|| {
                        panic!("Missing route param info for node ID {}", public_id)
                    });

                    let comm_path_id = route_param_str.parse::<u32>().unwrap_or_else(|_| {
                        panic!(
                            "Invalid route param '{}' for node ID {}.",
                            route_param_str, public_id
                        )
                    });

                    Arc::new(serial_node::SerialNode::new(public_id, comm_path_id))
                }
                "enviro_phat" => {
                    let route_param_str = node.route_param.unwrap_or_else(|| {
                        panic!("Missing route param info for node ID {}", public_id)
                    });

                    let comm_path_id = route_param_str.parse::<u32>().unwrap_or_else(|_| {
                        panic!(
                            "Invalid route param '{}' for node ID {}.",
                            route_param_str, public_id
                        )
                    });

                    Arc::new(enviro_phat::EnviroPHat::new(comm_path_id))
                }
                route_type => panic!(
                    "Invalid route type '{}' for node ID {}.",
                    route_type, public_id
                ),
            };

            node_map.insert(public_id, sensor_node);
        }

        SensorNodeRegistry {
            node_map: Arc::new(node_map),
        }
    }

    pub fn get_node(&self, node_id: u32) -> Result<Arc<dyn SensorNode>, MeteoError> {
        self.node_map
            .get(&node_id)
            .ok_or(MeteoError)
            .map(|arc| arc.clone())
    }
}

unsafe impl Sync for SensorNodeRegistry {}
