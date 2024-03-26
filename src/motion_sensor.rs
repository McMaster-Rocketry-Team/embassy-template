// Driver for the h3lis100dl motion sensor

// Global Variables
const WHO_AM_I_ADDR: u8 = 0x0F;
const H3LIS100DL_ID: u8 = 0x32;
const OUT_X_REG: u8 = 0x29;
const OUT_Y_REG: u8 = 0x2B;
const OUT_Z_REG: u8 = 0x2D;
const STATUS_REG: u8 = 0x27;
const CTRL_REG1: u8 = 0x20;
const CTRL_REG2: u8 = 0x21;

// SPI Configuration
use crate::{mutable_freq_spi::MutableFrequencySpi, shared_spi::SpiBusError};

// GPIO Configuration
use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    peripherals::PC8, // PC8 is the chip select pin
    time::mhz,
};

// Time Configuration
use embassy_time::Instant;

// Driver Configuration
use firmware_common::driver::timer::Timer;

// Register Configuration
use self::registers::*;
mod registers;

// Error Configuration
#[derive(defmt::Format, Debug)]
pub enum H3LIS100DLError {
    SpiBusError(SpiBusError),
    InvalidRegister,
}

// Conversion Configuration
impl From<SpiBusError> for H3LIS100DLError {
    fn from(e: SpiBusError) -> Self {
        Self::SpiBusError(e)
    }
}

// The range of the accelerometer is +/- 100g
const ACCEL_RANGE: AccelerometerRange = AccelerometerRange::G_100;
const G: f32 = 9.81;

pub struct H3LIS100DL<'b, B: MutableFrequencySpi, T: Timer> {
    spi: &'b mut B,
    cs: Output<'static, AnyPin>,
    timer: T,
}

impl<'b, B: MutableFrequencySpi, T: Timer> H3LIS100DL<'b, B, T> {
    pub fn new(timer: T, cs_pin: PC8, spi: &'b mut B) -> Self {
        let cs = Output::new(cs_pin.degrade(), Level::Low, Speed::VeryHigh); // Set CS low to enable SPI
        Self { spi, cs, timer }
    }

    // Function to read data from a register
    async fn read_register<R: TryFrom<u8>>(&mut self, address: u8) -> Result<R, H3LIS100DLError> {
        let mut data = [address | 0b10000000, 0u8]; // the MSB of the address must be 1 to indicate a read ; [address, dummy byte]
        self.spi
            .transfer_in_place(mhz(10), &mut data, &mut self.cs) // Transfer data to the SPI bus at 400 Hz (max frequency of the H3LIS100DL)
            .await?;
        Ok(R::try_from(data[1]).map_err(|_| H3LIS100DLError::InvalidRegister)?) // Return the data from the register; if the data is invalid, return an error
    }

    // Function to write data to a register
    async fn write_register<R: Into<u8>>(
        &mut self,
        address: u8,
        data: R,
    ) -> Result<(), H3LIS100DLError> {
        let mut data = [address & !0b10000000, data.into()]; // the MSB of the address must be 0 to indicate a write
        self.spi
            .transfer_in_place(mhz(10), &mut data, &mut self.cs)
            .await?;
        Ok(())
    }
}

impl<'b, B: MutableFrequencySpi, T: Timer> Motion_Sensor for H3LIS100DL<'b, B, T> {
    type Error = H3LIS100DLError;

    // Function to wait for the sensor to power on
    async fn wait_for_power_on(&mut self) -> Result<(), H3LIS100DLError> {
        self.timer.sleep(100.0).await;
        Ok(())
    }

    // Function to reset the sensor
    async fn reset(&mut self) -> Result<(), H3LIS100DLError> {
        //powerdown, sleep, normal mode (400 Hz)
        self.write_register(CTRL_REG1, 0b00000000).await?;
        self.timer.sleep(100.0).await;
        self.write_register(CTRL_REG1, 0b00110000).await?;
        self.timer.sleep(100.0).await;

        //reset internal registers (backup)
        self.write_register(CTRL_REG2, 0b10000000).await?;

        //check the sensor ID
        self.read_register::<WhoAmI>(WHO_AM_I_ADDR).await?;

        Ok(())
    }

    // Function to read the acceleration data from the sensor
    async fn read_acceleration(&mut self) -> Result<Acceleration, H3LIS100DLError> {
        let x = self.read_register(OUT_X_REG).await? as i16;
        let y = self.read_register(OUT_Y_REG).await? as i16;
        let z = self.read_register(OUT_Z_REG).await? as i16;
        Ok(Acceleration {
            x: x as f32 * ACCEL_RANGE.get_scale_factor() * G,
            y: y as f32 * ACCEL_RANGE.get_scale_factor() * G,
            z: z as f32 * ACCEL_RANGE.get_scale_factor() * G,
        })
    }
}
