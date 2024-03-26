use crate::{mutable_freq_spi::MutableFrequencySpi, shared_spi::SpiBusError};

use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    peripherals::PC8,
    time::mhz,
};
use embassy_time::Instant;
use firmware_common::driver::{
    imu::{IMUReading, IMU},
    timer::Timer,
};

use self::registers::*;

mod registers;

#[derive(defmt::Format, Debug)]
pub enum MPU6500Error {
    SpiBusError(SpiBusError),
    InvalidRegister,
}

impl From<SpiBusError> for MPU6500Error {
    fn from(e: SpiBusError) -> Self {
        Self::SpiBusError(e)
    }
}

const GYRO_RANGE: GyroscopeRange = GyroscopeRange::DPS_2000;
const ACCEL_RANGE: AccelerometerRange = AccelerometerRange::G_16;
const G: f32 = 9.81;

pub struct MPU6500<'b, B: MutableFrequencySpi, T: Timer> {
    spi: &'b mut B,
    cs: Output<'static, AnyPin>,
    timer: T,
}

impl<'b, B: MutableFrequencySpi, T: Timer> MPU6500<'b, B, T> {
    pub fn new(timer: T, cs: PC8, spi: &'b mut B) -> Self {
        Self {
            spi,
            cs: Output::new(cs.degrade(), Level::High, Speed::VeryHigh),
            timer,
        }
    }

    async fn read_register<R: TryFrom<u8>>(&mut self, address: u8) -> Result<R, MPU6500Error> {
        let mut data = [address | 0b10000000, 0u8]; // the MSB of the address must be 1 to indicate a read
        self.spi
            .transfer_in_place(mhz(1), &mut data, &mut self.cs)
            .await?;
        Ok(R::try_from(data[1]).map_err(|_| MPU6500Error::InvalidRegister)?)
    }

    async fn write_register<R: Into<u8>>(
        &mut self,
        address: u8,
        data: R,
    ) -> Result<(), MPU6500Error> {
        let mut data = [address & !0b10000000, data.into()]; // the MSB of the address must be 0 to indicate a write
        self.spi
            .transfer_in_place(mhz(1), &mut data, &mut self.cs)
            .await?;
        Ok(())
    }
}

impl<'b, B: MutableFrequencySpi, T: Timer> IMU for MPU6500<'b, B, T> {
    type Error = MPU6500Error;

    async fn wait_for_power_on(&mut self) -> Result<(), MPU6500Error> {
        self.timer.sleep(100.0).await;
        Ok(())
    }

    async fn reset(&mut self) -> Result<(), MPU6500Error> {
        // reset imu
        {
            let mut reg = PowerManagement1::default();
            reg.set_device_reset(true);
            self.write_register(PWR_MGMT_1_ADDRESS, reg).await?;

            self.timer.sleep(100.0).await;
        }

        // check who am i
        {
            self.read_register::<WhoAmI>(WHO_AM_I_ADDRESS).await?;
        }

        // disable i2c interface
        {
            let mut reg = UserControl::default();
            reg.set_i2c_if_dis(true);
            self.write_register(USER_CONTROL_ADDRESS, reg).await?;
        }

        // configure accelerometer
        {
            let mut reg = AccelerometerConfig1::default();
            reg.set_range(ACCEL_RANGE);
            self.write_register(ACC_CONFIG_1_ADDRESS, reg).await?;

            let mut reg = AccelerometerConfig2::default();
            reg.set_data_rate(AccelerometerDataRate::Hz_4K);
            self.write_register(ACC_CONFIG_2_ADDRESS, reg).await?;
        }

        // configure gyroscope
        {
            let mut reg = GyroscopeConfig::default();
            reg.set_range(GYRO_RANGE);
            self.write_register(GYRO_CONFIG_ADDRESS, reg).await?;
        }

        Ok(())
    }

    async fn read(&mut self) -> Result<IMUReading, MPU6500Error> {
        let mut write: [u8; 15] = [0; 15];
        write[0] = 0b10111011;
        let mut read: [u8; 15] = [0; 15];
        let timestamp = Instant::now().as_micros() as f64 / 1000.0;
        self.spi
            .transfer(mhz(20), &mut read, &write, &mut self.cs)
            .await?;

        let x = i16::from_be_bytes([read[1], read[2]]) as f32 / ACCEL_RANGE.get_scale_factor() * G;
        let y = i16::from_be_bytes([read[3], read[4]]) as f32 / ACCEL_RANGE.get_scale_factor() * G;
        let z = i16::from_be_bytes([read[5], read[6]]) as f32 / ACCEL_RANGE.get_scale_factor() * G;

        let acc = [x, y, z];

        let x = i16::from_be_bytes([read[9], read[10]]) as f32 / GYRO_RANGE.get_scale_factor();
        let y = i16::from_be_bytes([read[11], read[12]]) as f32 / GYRO_RANGE.get_scale_factor();
        let z = i16::from_be_bytes([read[13], read[14]]) as f32 / GYRO_RANGE.get_scale_factor();
        let gyro = [x, y, z];
        Ok(IMUReading {
            timestamp,
            acc,
            gyro,
        })
    }
}