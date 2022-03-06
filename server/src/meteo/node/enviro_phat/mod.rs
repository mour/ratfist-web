use super::SensorNode;

use crate::meteo::models::SensorTypeEnum;

use crate::comm;

mod bmp280;
mod tcs3472;

use bmp280::{Bmp280, IIRCoeficient, Mode, Oversampling, StandbyTime};

use tcs3472::Tcs3472;

use crate::utils::Result;
use anyhow::anyhow;

pub struct EnviroPHat {
    bmp: Bmp280,
    tcs: Tcs3472,
}

impl EnviroPHat {
    pub fn new(i2c_comm_path_id: u32) -> Result<EnviroPHat> {
        let comm_path = comm::get_i2c_comm_path(i2c_comm_path_id)?;

        let bmp = bmp280::Bmp280::new(
            comm_path.clone(),
            StandbyTime::Time1000ms,
            IIRCoeficient::Mult4X,
            Oversampling::Mult16X,
            Oversampling::Mult2X,
            Mode::Normal,
        )?;

        let tcs = tcs3472::Tcs3472::new(comm_path)?;

        Ok(EnviroPHat { bmp, tcs })
    }
}

impl SensorNode for EnviroPHat {
    fn measure(&self, measurement_type: SensorTypeEnum, sensor_id: u32) -> Result<f32> {
        if sensor_id != 0 {
            return Err(anyhow!("Invalid sensor ID {sensor_id}. ID must be 0.").into());
        }

        match measurement_type {
            SensorTypeEnum::Pressure => Ok(self.bmp.query_press_and_temp()?.0),
            SensorTypeEnum::Temperature => Ok(self.bmp.query_press_and_temp()?.1),
            SensorTypeEnum::Humidity => {
                Err(anyhow!("Humidity measurements not supported on the EnviroPHat.").into())
            }
            SensorTypeEnum::LightLevel => Ok(self.tcs.query_light_level()?),
        }
    }
}
