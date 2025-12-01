#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_io::Write;
use esp_hal::{
    i2c::master::I2c, interrupt::software::SoftwareInterruptControl, timer::timg::TimerGroup,
};

use icm42670::{Address, Icm42670, prelude::*};

use defmt::info;

use esp_backtrace as _;
use esp_println as _;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    info!("Embassy blinky example started!");

    // Inititalize HAL and obtain peripherals object
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Configure esp_rtos (similar to embassy-executor for Arm)
    // This lets us run async code
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    let i2c = I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO10)
        .with_scl(peripherals.GPIO8);

    let mut icm = Icm42670::new(i2c, Address::Primary).unwrap();

    loop {
        let accel_norm = icm.accel_norm().unwrap();
        let gyro_norm = icm.gyro_norm().unwrap();

        let mut buf = [0u8; 1000];
        // Use write! instead of defmt for float precision settings
        write!(
            &mut buf[..],
            "ACCEL  =  X: {:+.04} Y: {:+.04} Z: {:+.04}\tGYRO  =  X: {:+.04} Y: {:+.04} Z: {:+.04}\tTEMP = {:+0.2}",
            accel_norm.x,
            accel_norm.y,
            accel_norm.z,
            gyro_norm.x,
            gyro_norm.y,
            gyro_norm.z,
            icm.temperature().unwrap()
        ).unwrap();
        info!("{}", str::from_utf8(&buf).unwrap());

        Timer::after_millis(500).await;
    }

    // TODO: Spawn a task that lights up the User-LED when
}
