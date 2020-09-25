use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CBus, LinuxI2CMessage};

pub struct CommChannel {
    bus: LinuxI2CBus,
}

impl CommChannel {
    pub fn new(i2c_comm_path_id: u32) -> CommChannel {
        let env_var_str = format!("I2C_BUS_{}_PATH", i2c_comm_path_id);

        let bus = LinuxI2CBus::new(
            &dotenv::var(&env_var_str)
                .unwrap_or_else(|_| panic!("missing {} env variable", env_var_str)),
        )
        .expect("could not open I2C bus");

        CommChannel { bus }
    }

    pub fn transfer(&mut self, messages: &mut [LinuxI2CMessage]) -> Result<(), ()> {
        self.bus.transfer(messages).map_err(|_| ())?;

        Ok(())
    }
}
