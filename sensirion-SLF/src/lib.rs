use anyhow;
use embedded_hal::i2c::I2c;

/// According to https://sensirion.com/media/documents/C4F8D965/66F56F53/LQ_DS_SLF3S-0600F_Datasheet.pdf
#[derive(Clone, Copy)]
enum Command {
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

    ReadProductIdentifier32 = 0x367C,
    ReadProductIdentifier64 = 0xE102,
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
    const WRITE_FLAG: u8 = 0x00;
    const READ_FLAG: u8 = 0x01;

    pub fn new(i2c: I2C) -> Self {
        Self { i2c: i2c }
    }

    pub fn read<const DATA_SIZE: usize>(
        &mut self,
        command: Command,
    ) -> anyhow::Result<[u8; DATA_SIZE]> {
        self.write(command, None)?;
        let mut data = [0; DATA_SIZE];
        self.i2c.read(Self::ADDRESS | Self::READ_FLAG, &mut data);
        Ok(data)
    }

    pub fn write(&mut self, command: Command, data: Option<&[u8]>) -> anyhow::Result<()> {
        let mut sent = [command.to_be_bytes()[0], command.to_be_bytes()[1], 0, 0, 0];

        let len = if let Some(data) = data {
            if data.len() != 2 {
                anyhow::bail!("Incorrect data len");
            }
            sent[2] = data[0];
            sent[3] = data[1];
            sent[4] = crc8::compute_crc8(data);
            5
        } else {
            2
        };
        self.i2c
            .write(Self::ADDRESS | Self::WRITE_FLAG, &sent[..len]);
        Ok(())
    }
}

mod crc8 {
    pub fn crc8_matches(data: &[u8], crc: u8) -> bool {
        compute_crc8(data) == crc
    }

    const INITIAL: u8 = 0xFF;
    const XOR: u8 = 0x31;

    /// Computes a CRC-8 according to NRSC-5
    /// width=8 poly=0x31 init=0xff refin=false refout=false xorout=0x00 check=0xf7 residue=0x00 name="CRC-8/NRSC-5"
    pub fn compute_crc8(data: &[u8]) -> u8 {
        let mut crc = INITIAL;
        for byte in data.iter() {
            crc ^= byte;
            for _ in 0..8 {
                if (crc & 0x80) != 0 {
                    crc = (crc << 1) ^ XOR;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }
}

#[cfg(test)]
pub mod tests {
    use crate::crc8::*;

    #[test]
    fn test_crc8() {
        assert!(crc8_matches(&[0xBE, 0xEF], 0x92))
    }
}
