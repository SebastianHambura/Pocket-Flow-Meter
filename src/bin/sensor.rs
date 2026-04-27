use esp_hal::delay::Delay;
use esp_hal::i2c::master::I2c;
use sensirion_SLF::models::SLF3S_0600F;
use sensirion_SLF::slf3_driver::Slf3sDriver;
use sensirion_SLF::{SensorCommunication, SensorDriver};

type BlockingI2C = I2c<'static, esp_hal::Blocking>;

pub fn auto_dectect_sensor(
    i2c: BlockingI2C,
) -> Result<SensorDriver<BlockingI2C>, (BlockingI2C, anyhow::Error)> {
    let mut slf_sensor: Slf3sDriver<_, SLF3S_0600F> =
        sensirion_SLF::slf3_driver::Slf3sDriver::new(i2c);

    if let Err(err) = slf_sensor.soft_reset() {
        log::error!("Soft reset: {err}")
        //return Err((slf_sensor.into_inner(), err));
    };

    Delay::new().delay_millis(50);

    match slf_sensor.read_product_id() {
        Ok((id, serial_number)) => {
            log::info!("Connected to sensor {serial_number}");
            let i2c = slf_sensor.into_inner();
            Ok(SensorDriver::new(i2c, id))
        }
        Err(err) => return Err((slf_sensor.into_inner(), err)),
    }
}
