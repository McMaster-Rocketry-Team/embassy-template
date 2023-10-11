#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::time::mhz;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

// For STM32F303 NUCLEO: 
// PA5 is the LED, when high it's on
// PC13 is the button, when pressed it's low, external pullup
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config: Config = Default::default();
    config.rcc.sysclk = Some(mhz(72));
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    
}
