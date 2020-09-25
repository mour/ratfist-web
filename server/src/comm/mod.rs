use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

pub mod i2c;
pub mod serial;

lazy_static! {
    static ref SERIAL_PATH_REGISTRY: Mutex<HashMap<u32, Arc<Mutex<serial::CommChannelTx>>>> =
        Mutex::new(HashMap::new());
    static ref I2C_PATH_REGISTRY: Mutex<HashMap<u32, Arc<Mutex<i2c::CommChannel>>>> =
        Mutex::new(HashMap::new());
}

pub fn get_serial_comm_path(serial_comm_path_id: u32) -> Arc<Mutex<serial::CommChannelTx>> {
    let mut map = SERIAL_PATH_REGISTRY.lock().expect("mutex poisoned");

    let comm_path = map.entry(serial_comm_path_id).or_insert_with(|| {
        Arc::new(Mutex::new(
            serial::create_serial_comm_task(serial_comm_path_id).0,
        ))
    });

    comm_path.clone()
}

pub fn get_i2c_comm_path(i2c_comm_path_id: u32) -> Arc<Mutex<i2c::CommChannel>> {
    let mut map = I2C_PATH_REGISTRY.lock().expect("mutex poisoned");

    let comm_path = map
        .entry(i2c_comm_path_id)
        .or_insert_with(|| Arc::new(Mutex::new(i2c::CommChannel::new(i2c_comm_path_id))));

    comm_path.clone()
}
