use core::option::Option;
use core::option::Option::*;
use core::result::Result::*;
use embedded_charts::data::Point2D;
use esp_hal::i2c::master::I2c;
use micromath::F32Ext;

type BlockingI2C = I2c<'static, esp_hal::Blocking>;
pub struct Sensor {
    sensor: sensirion_SLF::SLF3S<BlockingI2C>,
    do_measurements: bool

}

pub struct Measurement {
    pub flow: Point2D,
    pub temp: Point2D, 
}

impl Measurement {
    pub fn new(timestamp: f32, flow: f32, temp: f32) -> Self {
        Self { flow: Point2D::new(timestamp, flow), temp: Point2D::new(timestamp, temp) }
    }
}


impl Sensor {
    pub fn new(mut i2c: BlockingI2C) -> Self {
        Self {
            sensor: sensirion_SLF::SLF3S::new(i2c),
            do_measurements: false,
            
        }
    }

    pub fn get_ID(&mut self) -> Option<u32> {
        match self.sensor.read_product_id() {
            Ok(id) => Some(id.0),
            Err(err) => {
                log::error!("{:?}", err);
                None
            }
        }
    }

    pub fn swap_measurement(&mut self) -> anyhow::Result<()> {
        match self.do_measurements {
            true => {
                self.sensor.stop_measurement()
            },
            false => {
                self.sensor.start_continuous_measurement_water()
            },
        }
    }

    pub fn get_measurement(&mut self, i: f32) -> Option<Measurement> {
        match self.sensor.read_measurement() {
            Ok((flow, temp, signal)) => {
                Some(
                    Measurement::new(i, flow.into(), temp.into())
                )
            },
            Err(err) =>{
                log::error!("{:?}", err);
                None
            }
        }
        // let timestamp = i * 0.1;
        // let value = 50.0 + 20.0 * (timestamp * 0.5).sin();
        // Measurement::new(timestamp, value)
    }
}
