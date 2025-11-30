#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    interrupt::software::SoftwareInterruptControl,
    rmt::{PulseCode, Rmt},
    time::Rate,
    timer::timg::TimerGroup,
};
use static_cell::StaticCell;

use esp_hal_smartled::{SmartLedsAdapterAsync, buffer_size_async};
use smart_leds::{SmartLedsWriteAsync, brightness, colors::*, RGB8};

use defmt::info;
use esp_backtrace as _;
use esp_println as _;

esp_bootloader_esp_idf::esp_app_desc!();

const BUFFER_SIZE: usize = buffer_size_async(1);

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

    // Configure RMT (Remote Control Transceiver) peripheral globally
    // <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/peripherals/rmt.html>
    let rmt: Rmt<'_, esp_hal::Async> = Rmt::new(peripherals.RMT, Rate::from_mhz(80))
        .expect("Failed to initialize RMT")
        .into_async();
    // We use one of the RMT channels to instantiate a `SmartLedsAdapterAsync` which can
    // be used directly with all `smart_led` implementations
    let rmt_channel = rmt.channel0;

    /// Create static buffer that can be used as a reference with 'static lifetime
    /// in the SmartLedsAdapterAsync, so that it can be passed to the task.
    static RMT_BUFFER: StaticCell<[PulseCode; BUFFER_SIZE]> = StaticCell::new();
    let rmt_buffer = RMT_BUFFER.init([esp_hal::rmt::PulseCode::default(); BUFFER_SIZE]);
    let mut led = SmartLedsAdapterAsync::new(rmt_channel, peripherals.GPIO2, rmt_buffer);

    led.write(brightness([ORANGE].into_iter(), 10))
        .await
        .unwrap();

    info!("RMT initialized");

    spawner.spawn(run(led)).unwrap();

    loop {
        defmt::info!("Bing!");
        Timer::after(Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task]
async fn run(mut led: SmartLedsAdapterAsync<'static, BUFFER_SIZE>) {
    let mut hue = 0u8;
    loop {
        let color = wheel(hue);
        led.write(brightness([color].into_iter(), 25))
            .await
            .unwrap();

        hue = hue.wrapping_add(1);
        Timer::after(Duration::from_millis(20)).await;
    }
}

fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}
