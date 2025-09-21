use std::any;

use anyhow::{Result, anyhow};
use embedded_hal::i2c::I2c;

/// According to https://sensirion.com/media/documents/C4F8D965/66F56F53/LQ_DS_SLF3S-0600F_Datasheet.pdf
///
/// 4.2 I2C Sequences
/// The commands are 16-bit.
#[repr(u16)]
#[derive(Clone, Copy)]
pub enum Command {
    /// This command starts the continuous measurement
    /// mode for H2O. Outputs are the liquid flow rate, the
    /// sensor’s temperature and the signaling flags.
    ContinuousMeasurementWater = 0x3608,
    /// This command starts the continuous measurement
    /// mode for IPA. Outputs are the liquid flow rate, the
    /// sensor’s temperature and the signaling flags.
    ContinuousMeasurementIsopropylAlcohol = 0x3615,

    /// This command stops the continuous measurement and
    /// puts the sensor in idle mode. After it receives the stop
    /// command, the sensor needs up to 0.5 ms to power
    /// down the heater, enter idle mode and be receptive for a
    /// new command.
    StopContinuousMeasurment = 0x3FF9,

    /// This sequence resets the sensor with a separate reset
    /// block, which is as much as possible detached from the
    /// rest of the system on chip.
    /// **Note that the I2C address is 0x00, which is the general call
    /// address, and that the command is 8-bit**, i.e., the soft reset
    /// command must not be preceded by an I2C write header.
    /// The reset is implemented according to the I2C
    /// specification
    GeneralCallReset = 0x0006,

    ReadProductIdentifier1 = 0x367C,
    ReadProductIdentifier2 = 0xE102,
}

impl Command {
    /// Returns a big endian byte representation of the command.
    pub fn to_be_bytes(&self) -> [u8; 2] {
        (*self as u16).to_be_bytes()
    }
}

pub struct SLF3S<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> SLF3S<I2C> {
    const ADDRESS: u8 = 0x61;

    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    fn read<const DATA_SIZE: usize>(
        &mut self,
        command: Command,
    ) -> anyhow::Result<[u8; DATA_SIZE]> {
        self.write(command, None)?;
        let mut data = [0; DATA_SIZE];
        sensirion_i2c::i2c::read_words_with_crc(&mut self.i2c, Self::ADDRESS, &mut data)
            .map_err(|err| convert_error(err))?;
        Ok(data)
    }

    fn write(&mut self, command: Command, data: Option<&[u8]>) -> anyhow::Result<()> {
        sensirion_i2c::i2c::write_command_u16(&mut self.i2c, Self::ADDRESS, command as u16)
            .map_err(|err| anyhow!("{:?}", err))?;
        Ok(())
    }

    /// Implements "4.3.4 Read Product Identifier and Serial Number" from the documentation
    pub fn read_product_id(&mut self) -> Result<(u32, u64)> {
        self.write(Command::ReadProductIdentifier1, None)?;
        let data = self.read::<18>(Command::ReadProductIdentifier2)?;

        // According to the documentation, table 12:
        let product_number = u32::from_be_bytes([data[0], data[1], data[3], data[4]]);
        let serial_number = u64::from_be_bytes([
            data[6], data[7], data[9], data[10], data[12], data[13], data[15], data[16],
        ]);

        Ok((product_number, serial_number))
    }

    pub fn start_continuous_measurement_water(&mut self) -> Result<()> {
        self.write(Command::ContinuousMeasurementWater, None)?;
        Ok(())
    }

    pub fn start_continuous_measurement_alcohol(&mut self) -> Result<()> {
        self.write(Command::ContinuousMeasurementIsopropylAlcohol, None)?;
        Ok(())
    }
    /// See "4.3.1 Start Continuous Measurement"
    ///
    /// Return  Ok((flow, temp, signal))
    pub fn read_measurement(&mut self) -> Result<(u16, u16, u16)> {
        let mut data = [0; 3 * (2 + 1)];
        sensirion_i2c::i2c::read_words_with_crc(&mut self.i2c, Self::ADDRESS, &mut data)
            .map_err(|err| convert_error(err))?;
        let flow = u16::from_be_bytes([data[0], data[1]]);
        let temp = u16::from_be_bytes([data[3], data[4]]);
        let signal = u16::from_be_bytes([data[6], data[7]]);
        Ok((flow, temp, signal))
    }

    pub fn stop_measurement(&mut self) -> Result<()> {
        self.write(Command::StopContinuousMeasurment, None)?;
        Ok(())
    }

    pub fn soft_reset(&mut self) -> Result<()> {
        sensirion_i2c::i2c::write_command_u8(&mut self.i2c, 0, 0x0006)
            .map_err(|err| anyhow!("{:?}", err))?;
        Ok(())
    }
}

fn convert_error<I: embedded_hal::i2c::ErrorType>(
    error: sensirion_i2c::i2c::Error<I>,
) -> anyhow::Error {
    match error {
        sensirion_i2c::i2c::Error::I2cWrite(err) => anyhow!("{:?}", err),
        sensirion_i2c::i2c::Error::I2cRead(err) => anyhow!("{:?}", err),
        sensirion_i2c::i2c::Error::Crc => anyhow::anyhow!("CRC error"),
    }
}

#[cfg(test)]
pub mod tests {
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use sensirion_i2c::crc8;

    use crate::{Command, SLF3S};

    fn with_crc(data: Vec<u8>) -> Vec<u8> {
        assert!(data.len() % 2 == 0);
        data.chunks_exact(2)
            .flat_map(|bytes| [bytes[0], bytes[1], crc8::calculate(&[bytes[0], bytes[1]])])
            .collect()
    }

    #[test]
    fn test_measurements() {
        let addr = 0x61;
        let flow: u16 = 0xDEAD;
        let temp: u16 = 0xBEEF;
        let signal: u16 = 0x1234;
        let bytes: Vec<_> = [flow.to_be_bytes(), temp.to_be_bytes(), signal.to_be_bytes()].concat();
        let expectations = [
            I2cTransaction::write(
                addr,
                Command::ContinuousMeasurementWater.to_be_bytes().to_vec(),
            ),
            I2cTransaction::read(addr, with_crc(bytes.clone())),
            I2cTransaction::read(addr, with_crc(bytes.clone())),
            I2cTransaction::read(addr, with_crc(bytes)),
            I2cTransaction::write(
                addr,
                Command::StopContinuousMeasurment.to_be_bytes().to_vec(),
            ),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut sensirion = SLF3S::new(i2c.clone());

        sensirion.start_continuous_measurement_water().unwrap();
        let (flow_read, temp_read, signal_read) = sensirion.read_measurement().unwrap();
        assert_eq!(flow_read, flow);
        assert_eq!(temp_read, temp);
        assert_eq!(signal_read, signal);
        println!("flow: {flow:#x}, temp:{temp:#x}, signal: {signal:#x}");
        let (flow, temp, signal) = sensirion.read_measurement().unwrap();
        println!("flow: {flow:#x}, temp:{temp:#x}, signal: {signal:#x}");
        let (flow, temp, signal) = sensirion.read_measurement().unwrap();
        println!("flow: {flow:#x}, temp:{temp:#x}, signal: {signal:#x}");
        sensirion.stop_measurement().unwrap();

        i2c.done();
    }

    #[test]
    fn test_read_product() {
        let addr = 0x61;
        let PN: u32 = 0x07030302;
        let SN: u64 = 0xDEADBEEF_DEADBEEF;
        let mut bytes = PN.to_be_bytes().to_vec();
        bytes.extend_from_slice(&SN.to_be_bytes());
        let expectations = [
            I2cTransaction::write(addr, Command::ReadProductIdentifier1.to_be_bytes().to_vec()),
            I2cTransaction::write(addr, Command::ReadProductIdentifier2.to_be_bytes().to_vec()),
            I2cTransaction::read(addr, with_crc(bytes)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut sensirion = SLF3S::new(i2c.clone());

        let (device, SN) = sensirion.read_product_id().unwrap();
        println!("device: {device:#X}, SN: {SN:#X}");

        i2c.done();
    }
}
