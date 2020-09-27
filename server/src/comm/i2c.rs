use i2cdev::linux::LinuxI2CBus;

use std::ops::{Deref, DerefMut};

pub struct CommChannel(LinuxI2CBus);

impl Deref for CommChannel {
    type Target = LinuxI2CBus;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CommChannel {
    fn deref_mut(&mut self) -> &mut LinuxI2CBus {
        &mut self.0
    }
}

impl CommChannel {
    pub fn new(i2c_comm_path_id: u32) -> CommChannel {
        let env_var_str = format!("I2C_BUS_{}_PATH", i2c_comm_path_id);

        let bus = LinuxI2CBus::new(
            &dotenv::var(&env_var_str)
                .unwrap_or_else(|_| panic!("missing {} env variable", env_var_str)),
        )
        .expect("could not open I2C bus");

        CommChannel(bus)
    }
}
