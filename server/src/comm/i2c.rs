use i2cdev::core::I2CTransfer;
use i2cdev::linux::{LinuxI2CBus, LinuxI2CMessage};

use anyhow::anyhow;
pub struct CommChannel(LinuxI2CBus);

impl CommChannel {
    pub fn new(i2c_comm_path_id: u32) -> crate::utils::Result<CommChannel> {
        let env_var_str = format!("I2C_BUS_{}_PATH", i2c_comm_path_id);

        let bus = LinuxI2CBus::new(
            &dotenv::var(&env_var_str)
                .map_err(|e| anyhow!("Missing {env_var_str} env variable. {e:?}"))?,
        )
        .map_err(|e| anyhow!("Could not open I2C bus. {e:?}"))?;

        Ok(CommChannel(bus))
    }

    pub fn transfer(&mut self, msgs: &mut [LinuxI2CMessage]) -> crate::utils::Result<()> {
        self.0
            .transfer(msgs)
            .map(|_| ())
            .map_err(|e| anyhow!("I2C transfer error. {e:?}").into())
    }
}
