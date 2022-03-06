use std::collections::BTreeMap;
use std::convert::TryInto;
use std::sync::Arc;

use crate::db;
use crate::db::models::Node;

use super::models::SensorTypeEnum;

use diesel::prelude::*;

mod enviro_phat;
mod serial_node;

use crate::utils::Result;
use anyhow::anyhow;

pub trait SensorNode: Sync + Send {
    fn measure(&self, measurement_type: SensorTypeEnum, sensor_id: u32) -> Result<f32>;
}

#[derive(Clone)]
pub struct SensorNodeRegistry {
    node_map: Arc<BTreeMap<u32, Arc<dyn SensorNode>>>,
}

impl SensorNodeRegistry {
    pub fn new(db_conn: db::Db) -> Result<SensorNodeRegistry> {
        let nodes = {
            use crate::db::schema::nodes;

            nodes::table
                .load::<Node>(&*db_conn)
                .map_err(|e| anyhow!("Error loading Node entries from DB. {e:?}"))?
        };

        let mut node_map: BTreeMap<u32, Arc<dyn SensorNode>> = BTreeMap::new();

        for node in nodes {
            let public_id: u32 = node.public_id.try_into().map_err(|e| {
                anyhow!(
                    "Error converting node public ID {} into u32. {:?}",
                    node.public_id,
                    e
                )
            })?;

            let sensor_node: Arc<dyn SensorNode> = match node.route_type.as_str() {
                "serial" => {
                    let route_param_str = node
                        .route_param
                        .ok_or(anyhow!("Missing route param info for node ID {public_id}."))?;

                    let comm_path_id = route_param_str.parse::<u32>().map_err(|e| {
                        anyhow!(
                            "Invalid route param '{route_param_str}' for node ID {public_id}. {e:?}"
                    )
                    })?;

                    Arc::new(serial_node::SerialNode::new(public_id, comm_path_id)?)
                }
                "envirophat" => {
                    let route_param_str = node
                        .route_param
                        .ok_or(anyhow!("Missing route param info for node ID {public_id}."))?;

                    let comm_path_id = route_param_str.parse::<u32>().map_err(|e| anyhow!(
                            "Invalid route param '{route_param_str}' for node ID {public_id}. {e:?}")
                    )?;

                    Arc::new(enviro_phat::EnviroPHat::new(comm_path_id)?)
                }
                route_type => {
                    return Err(anyhow!(
                        "Invalid route type '{route_type}' for node ID {public_id}."
                    )
                    .into());
                }
            };

            node_map.insert(public_id, sensor_node);
        }

        Ok(SensorNodeRegistry {
            node_map: Arc::new(node_map),
        })
    }

    pub fn get_node(&self, node_id: u32) -> Result<Arc<dyn SensorNode>> {
        self.node_map
            .get(&node_id)
            .ok_or_else(|| anyhow!("Could not find node {node_id} in sensor node registry.").into())
            .map(|arc| arc.clone())
    }
}

unsafe impl Sync for SensorNodeRegistry {}
