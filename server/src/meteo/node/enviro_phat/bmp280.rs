use std::sync::{Arc, Mutex};

use crate::comm::i2c;
use crate::meteo::MeteoError;

pub enum StandbyTime {
    Time0_5ms,
    Time62_5ms,
    Time125ms,
    Time250ms,
    Time500ms,
    Time1000ms,
    Time2000ms,
    Time4000ms,
}

pub enum IIRCoeficient {
    Off,
    Mult2X,
    Mult4X,
    Mult8X,
    Mult16X,
}

pub enum Oversampling {
    Mult1X,
    Mult2X,
    Mult4X,
    Mult8X,
    Mult16X,
}

pub enum Mode {
    Sleep,
    Normal,
    Forced,
}

pub struct Bmp280 {
    comm_path: Arc<Mutex<i2c::CommChannel>>,
}

impl Bmp280 {
    pub fn new(
        comm_path: Arc<Mutex<i2c::CommChannel>>,
        standby_time: StandbyTime,
        iir_coef: IIRCoeficient,
        press_oversampling: Oversampling,
        temp_oversampling: Oversampling,
        mode: Mode,
    ) -> Result<Bmp280, MeteoError> {
        let bmp = Bmp280 { comm_path };

        bmp.reconfigure(
            standby_time,
            iir_coef,
            press_oversampling,
            temp_oversampling,
            mode,
        )?;

        Ok(bmp)
    }

    fn reconfigure(
        &self,
        standby_time: StandbyTime,
        iir_coef: IIRCoeficient,
        press_oversampling: Oversampling,
        temp_oversampling: Oversampling,
        mode: Mode,
    ) -> Result<(), MeteoError> {
        Err(MeteoError)
    }

    pub fn query_press_and_temp(&self) -> Result<(f32, f32), MeteoError> {
        Err(MeteoError)
    }
}
