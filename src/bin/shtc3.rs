#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    i2c::master::I2c, interrupt::software::SoftwareInterruptControl, timer::timg::TimerGroup,
};


use defmt::info;
use esp_backtrace as _;
use esp_println as _;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    info!("SHTC3 example started!");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    let i2c = I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO10)
        .with_scl(peripherals.GPIO8);

    let mut sht = shtcx::shtc3(i2c);

    loop {
        match sht.measure(shtcx::PowerMode::NormalMode, &mut embassy_time::Delay) {
            Ok(measurement) => {
                info!(
                    "Temperature: {} °C, Humidity: {} %",
                    measurement.temperature.as_degrees_celsius(),
                    measurement.humidity.as_percent()
                );
            }
            Err(_) => info!("Error reading SHTC3"),
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}
