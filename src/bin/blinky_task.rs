#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    interrupt::software::SoftwareInterruptControl,
    timer::timg::TimerGroup,
};

use defmt::info;

use esp_backtrace as _;
use esp_println as _;

esp_bootloader_esp_idf::esp_app_desc!();

/// Main entry point for the blinky example
///
/// This example blinks the onboard LED (GPIO7) at 1Hz
#[esp_rtos::main]
async fn main(spawner: Spawner) {
    info!("Embassy blinky example started!");

    // Inititalize HAL and obtain peripherals object
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Configure esp_rtos (similar to embassy-executor for Arm)
    // This lets us run async code
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    // LED is connected to GPIO7 on the ESP32-C3 Rust board
    let led = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());

    info!("LED initialized on GPIO7");

    spawner.spawn(run(led)).unwrap();

    loop {
        defmt::info!("Bing!");
        Timer::after(Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task]
async fn run(mut led: Output<'static>) {
    // TODO: Instead of blinking by time, blink on a button press.
    // The User Button (BOOT) is on GPIO 9
    // And you can await the button falling edge, see
    // https://docs.espressif.com/projects/rust/esp-hal/1.0.0/esp32c3/esp_hal/gpio/struct.Input.html#method.wait_for_falling_edge

    loop {
        info!("LED ON");
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        info!("LED OFF");
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
