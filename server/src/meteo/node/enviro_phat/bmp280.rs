use i2cdev::core::*;
use i2cdev::linux::LinuxI2CMessage;

use std::sync::{Arc, Mutex};

use crate::comm::i2c;

use crate::utils::Result;
use anyhow::anyhow;

use log::debug;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum StandbyTime {
    Time0_5ms = 0b000,
    Time62_5ms = 0b001,
    Time125ms = 0b010,
    Time250ms = 0b011,
    Time500ms = 0b100,
    Time1000ms = 0b101,
    Time2000ms = 0b110,
    Time4000ms = 0b111,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum IIRCoeficient {
    Off = 0b001,
    Mult2X = 0b010,
    Mult4X = 0b011,
    Mult8X = 0b100,
    Mult16X = 0b101,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Oversampling {
    Mult1X = 0b001,
    Mult2X = 0b010,
    Mult4X = 0b011,
    Mult8X = 0b100,
    Mult16X = 0b101,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Mode {
    Sleep = 0b00,
    Forced = 0b01,
    Normal = 0b11,
}

struct CalibrationData {
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,
}

pub struct Bmp280 {
    comm_path: Arc<Mutex<i2c::CommChannel>>,
    calib: CalibrationData,
    press_oversampling: Oversampling,
    temp_oversampling: Oversampling,
    mode: Mode,
}

impl Bmp280 {
    const I2C_ADDR: u16 = 0x77;

    const CHIP_ID_REG_ADDR: u8 = 0xd0;
    const CHIP_ID_EXPECTED: u8 = 0x58;

    const CALIB_REG_ADDR: u8 = 0x88;
    const CALIB_DATA_SIZE: usize = 24;

    const CTRL_MEAS_REG_ADDR: u8 = 0xf4;
    const CONFIG_REG_ADDR: u8 = 0xf5;

    const DATA_REG_ADDR: u8 = 0xf7;
    const DATA_REG_SIZE: usize = 6;

    pub fn new(
        comm_path: Arc<Mutex<i2c::CommChannel>>,
        standby_time: StandbyTime,
        iir_coef: IIRCoeficient,
        press_oversampling: Oversampling,
        temp_oversampling: Oversampling,
        mode: Mode,
    ) -> Result<Bmp280> {
        // Check that we're dealing with the correct chip
        let mut id_data = [0];
        let mut id_msgs = [
            LinuxI2CMessage::write(&[Self::CHIP_ID_REG_ADDR]).with_address(Self::I2C_ADDR),
            LinuxI2CMessage::read(&mut id_data).with_address(Self::I2C_ADDR),
        ];

        debug!("Reading out chip ID");
        comm_path
            .lock()
            .expect("Mutex poisoned.")
            .transfer(&mut id_msgs)?;

        debug!("Chip ID is {}", id_data[0]);

        if id_data[0] != Self::CHIP_ID_EXPECTED {
            return Err(anyhow!(
                "BMP280 unexpected chip ID (0x{:X}) at address 0x{:X}",
                id_data[0],
                Self::CHIP_ID_REG_ADDR
            )
            .into());
        }

        debug!("Reading out BMP280 calibration data.");

        // Read out the factory calibration data
        let mut calib_data = [0; Self::CALIB_DATA_SIZE];
        let mut calib_msgs = [
            LinuxI2CMessage::write(&[Self::CALIB_REG_ADDR]).with_address(Self::I2C_ADDR),
            LinuxI2CMessage::read(&mut calib_data).with_address(Self::I2C_ADDR),
        ];

        comm_path
            .lock()
            .expect("Mutex poisoned.")
            .transfer(&mut calib_msgs)?;

        let calib = CalibrationData {
            dig_t1: ((calib_data[1] as u16) << 8) | (calib_data[0] as u16),
            dig_t2: (((calib_data[3] as u16) << 8) | (calib_data[2] as u16)) as i16,
            dig_t3: (((calib_data[5] as u16) << 8) | (calib_data[4] as u16)) as i16,
            dig_p1: ((calib_data[7] as u16) << 8) | (calib_data[6] as u16),
            dig_p2: (((calib_data[9] as u16) << 8) | (calib_data[8] as u16)) as i16,
            dig_p3: (((calib_data[11] as u16) << 8) | (calib_data[10] as u16)) as i16,
            dig_p4: (((calib_data[13] as u16) << 8) | (calib_data[12] as u16)) as i16,
            dig_p5: (((calib_data[15] as u16) << 8) | (calib_data[14] as u16)) as i16,
            dig_p6: (((calib_data[17] as u16) << 8) | (calib_data[16] as u16)) as i16,
            dig_p7: (((calib_data[19] as u16) << 8) | (calib_data[18] as u16)) as i16,
            dig_p8: (((calib_data[21] as u16) << 8) | (calib_data[20] as u16)) as i16,
            dig_p9: (((calib_data[23] as u16) << 8) | (calib_data[22] as u16)) as i16,
        };

        debug!("Calibration read out OK.");

        // Create the sensor struct & configure it.
        let bmp = Bmp280 {
            comm_path,
            calib,
            press_oversampling,
            temp_oversampling,
            mode,
        };

        debug!("Configuring BMP280.");

        bmp.reconfigure(
            standby_time,
            iir_coef,
            press_oversampling,
            temp_oversampling,
            mode,
        )?;

        debug!("BMP280 configuration OK.");

        Ok(bmp)
    }

    #[rustfmt::skip]
    pub fn query_press_and_temp(&self) -> Result<(f32, f32)> {

        if self.mode != Mode::Normal {
            let ctrl_meas_reg =
                ((self.temp_oversampling as u8) << 5) |
                ((self.press_oversampling as u8) << 2) |
                (Mode::Forced as u8);

            let mut config_msgs = [
                LinuxI2CMessage::write(&[Self::CTRL_MEAS_REG_ADDR, ctrl_meas_reg])
                    .with_address(Self::I2C_ADDR),
            ];

            self.comm_path
                .lock()
                .expect("Mutex poisoned.")
                .transfer(&mut config_msgs)?;

            // Wait times for single samples calculated from table 13
            // (3.8. Measurement Time) in the datasheet.
            // Add 2ms and round up just to be sure.
            let t_press_sample_ms: f32 = 2.2;
            let t_temp_sample_ms: f32 = 4.3;
            let wait_time_ms: u64 =
                (t_press_sample_ms * ((self.press_oversampling as u8) as f32) +
                 t_temp_sample_ms * ((self.temp_oversampling as u8) as f32) +
                 2.0)
                .ceil() as u64;

            std::thread::sleep(std::time::Duration::from_millis(wait_time_ms));
        }

        let mut raw_data = [0; Self::DATA_REG_SIZE];

        let mut read_data_msgs = [
            LinuxI2CMessage::write(&[Self::DATA_REG_ADDR]).with_address(Self::I2C_ADDR),
            LinuxI2CMessage::read(&mut raw_data).with_address(Self::I2C_ADDR),
        ];

        debug!("Reading out raw BMP280 data.");
        self.comm_path
            .lock()
            .expect("Mutex poisoned.")
            .transfer(&mut read_data_msgs)?;

        let raw_press =
            (((raw_data[0] as u32) << 12) |
             ((raw_data[1] as u32) << 4) |
             ((raw_data[2] as u32) >> 4)) as i32;

        let raw_temp =
            (((raw_data[3] as u32) << 12) |
             ((raw_data[4] as u32) << 4) |
             ((raw_data[5] as u32) >> 4)) as i32;

        debug!("Raw data: raw_press {}, raw_temp {}", raw_press, raw_temp);

        // See appendix 8.1 in the BMP280 datasheet for the explanation of this
        // algorithm.
        let t_var1: f32 =
            ((raw_temp as f32) / 16384.0 -
            (self.calib.dig_t1 as f32) / 1024.0) * (self.calib.dig_t2 as f32);

        let t_var2: f32 =
            ((raw_temp as f32) / 131072.0 - (self.calib.dig_t1 as f32) / 8192.0) *
            ((raw_temp as f32) / 131072.0 - (self.calib.dig_t1 as f32) / 8192.0) *
            (self.calib.dig_t3 as f32);

        let t_fine = t_var1 + t_var2;
        let output_temp = t_fine / 5120.0;


        let mut p_var1: f32 = (t_fine as f32) / 2.0 - 64000.0;
        let mut p_var2: f32 =
            p_var1 * p_var1 * (self.calib.dig_p6 as f32) / 32768.0 +
            p_var1 * (self.calib.dig_p5 as f32) * 2.0;
        p_var2 = (p_var2 / 4.0) + ((self.calib.dig_p4 as f32) * 65536.0);
        p_var1 =
            (((self.calib.dig_p3 as f32) * p_var1 * p_var1 / 524288.0) +
             ((self.calib.dig_p2 as f32) * p_var1)) / 524288.0;
        p_var1 = (1.0 + p_var1 / 32768.0) * (self.calib.dig_p1 as f32);

        let mut p_var3: f32 = 1048576.0 - (raw_press as f32);
        p_var3 = (p_var3 - (p_var2 / 4096.0)) * 6250.0 / p_var1;
        p_var1 = (self.calib.dig_p9 as f32) * p_var3 * p_var3 / 2147483648.0;
        p_var2 = p_var3 * (self.calib.dig_p8 as f32) / 32768.0;
        let output_press = p_var3 + (p_var1 + p_var2 + (self.calib.dig_p7 as f32)) / 16.0;

        debug!("Calculated BMP280 output: Pressure {} Pa, Temperature {} C",
               output_press, output_temp);

        Ok((output_press, output_temp))
    }

    #[rustfmt::skip]
    fn reconfigure(
        &self,
        standby_time: StandbyTime,
        iir_coef: IIRCoeficient,
        press_oversampling: Oversampling,
        temp_oversampling: Oversampling,
        mode: Mode,
    ) -> Result<()> {

        debug!("Reconfiguring BMP280: standby_time {:?}, iir_coef {:?},\
                press_oversampling {:?}, temp_oversampling {:?}, mode {:?}",
                standby_time, iir_coef, press_oversampling, temp_oversampling, mode);

        let ctrl_meas_reg =
            ((temp_oversampling as u8) << 5) |
            ((press_oversampling as u8) << 2) |
            (mode as u8);

        let config_reg = ((standby_time as u8) << 5) | ((iir_coef as u8) << 2);

        let mut config_msgs = [
            LinuxI2CMessage::write(&[Self::CTRL_MEAS_REG_ADDR, ctrl_meas_reg])
                .with_address(Self::I2C_ADDR),
            LinuxI2CMessage::write(&[Self::CONFIG_REG_ADDR, config_reg])
                .with_address(Self::I2C_ADDR),
        ];

        self.comm_path
            .lock()
            .expect("Mutex poisoned.")
            .transfer(&mut config_msgs)?;

        Ok(())
    }
}
