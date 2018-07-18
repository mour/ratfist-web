use dispatcher::Module;
use dispatcher::MsgSender;

use rand::distributions::Normal;
use rand::prelude::*;

use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
enum MeasurementType {
    Temperature,
    Humidity,
    Pressure,
    LightLevel,
}

#[derive(Debug)]
pub struct MeteoModule {
    last_values: HashMap<MeasurementType, HashMap<u32, f64>>,
    rng: SmallRng,
}

impl MeteoModule {
    pub fn new() -> MeteoModule {
        let mut last_values = HashMap::new();
        last_values.insert(MeasurementType::Temperature, HashMap::new());
        last_values.insert(MeasurementType::Humidity, HashMap::new());
        last_values.insert(MeasurementType::Pressure, HashMap::new());
        last_values.insert(MeasurementType::LightLevel, HashMap::new());

        let rng = SmallRng::from_entropy();

        MeteoModule { last_values, rng }
    }
    fn generate_new_value(&mut self, meas_type: MeasurementType, ch_num: u32) -> f32 {
        let dist = {
            let last_value = self.last_values[&meas_type].get(&ch_num);

            match meas_type {
                MeasurementType::Temperature => Normal::new(*last_value.unwrap_or(&25.0), 1.0),
                MeasurementType::Humidity => Normal::new(*last_value.unwrap_or(&65.0), 0.3),
                MeasurementType::Pressure => Normal::new(*last_value.unwrap_or(&101_325.0), 100.0),
                MeasurementType::LightLevel => Normal::new(*last_value.unwrap_or(&1000.0), 10.0),
            }
        };

        trace!("Using distribution: {:?}", dist);

        let val = self.rng.sample(dist);

        self.last_values
            .get_mut(&meas_type)
            .unwrap()
            .insert(ch_num, val);

        val as f32
    }
}

impl Module for MeteoModule {
    fn handle_incoming_msg(
        &mut self,
        msg_writer: &mut MsgSender,
        transaction_id: u32,
        msg_str: &str,
    ) -> Result<(), ()> {
        debug!("Handling message: {} {}", transaction_id, msg_str);

        let mut values = msg_str.split(',');

        let msg_type = values.next().ok_or(())?;
        let ch_num = values.next().ok_or(())?.parse::<u32>().map_err(|_| ())?;

        let response_payload_str = match msg_type {
            "GET_TEMPERATURE" => format!(
                "TEMPERATURE_REPLY,{},{}",
                ch_num,
                self.generate_new_value(MeasurementType::Temperature, ch_num)
            ),
            "GET_HUMIDITY" => format!(
                "HUMIDITY_REPLY,{},{}",
                ch_num,
                self.generate_new_value(MeasurementType::Humidity, ch_num)
            ),
            "GET_PRESSURE" => format!(
                "PRESSURE_REPLY,{},{}",
                ch_num,
                self.generate_new_value(MeasurementType::Pressure, ch_num)
            ),
            "GET_LIGHT_LEVEL" => format!(
                "LIGHT_LEVEL_REPLY,{},{}",
                ch_num,
                self.generate_new_value(MeasurementType::LightLevel, ch_num)
            ),
            unknown_msg_type_str => {
                warn!("Unknown message type: {}", unknown_msg_type_str);
                return Err(());
            }
        };

        msg_writer.write_msg(transaction_id, "METEO", &response_payload_str)
    }
}
