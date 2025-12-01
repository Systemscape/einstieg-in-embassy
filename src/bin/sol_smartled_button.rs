#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    watch::{Receiver, Sender, Watch},
};
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    interrupt::software::SoftwareInterruptControl,
    rmt::{PulseCode, Rmt},
    time::Rate,
    timer::timg::TimerGroup,
};
use static_cell::StaticCell;

use esp_hal_smartled::{SmartLedsAdapterAsync, buffer_size_async};
use smart_leds::{SmartLedsWriteAsync, brightness, colors::*};

use defmt::info;
use esp_backtrace as _;
use esp_println as _;

esp_bootloader_esp_idf::esp_app_desc!();

const BUFFER_SIZE: usize = buffer_size_async(1);

/// Embassy Watch that can be signalled by one task and awaited by N others
static BUTTON_WATCH: Watch<CriticalSectionRawMutex, (), 2> = Watch::new();

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
    let smartled = SmartLedsAdapterAsync::new(rmt_channel, peripherals.GPIO2, rmt_buffer);
    info!("RMT initialized");

    // LED is connected to GPIO7 on the ESP32-C3 Rust board
    let userled = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());

    let button = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(Pull::Up),
    );

    // Spawn Smart-LED color changing task
    spawner
        .spawn(smartled_task(smartled, BUTTON_WATCH.receiver().unwrap()))
        .unwrap();

    // Spawn User-LED blinking task
    spawner
        .spawn(userled_task(userled, BUTTON_WATCH.receiver().unwrap()))
        .unwrap();

    // Spawn Button press detection task
    spawner
        .spawn(button_task(button, BUTTON_WATCH.sender()))
        .unwrap();

    // TODO: Add another task that changes the user LED at GPIO7.
    // See blink.rs and blinky_task.rs for how to configure the LED.
    // Hint: embassy-sync Signal is only for a _single_ consumer,
    // see https://docs.rs/embassy-sync/latest/embassy_sync/signal/struct.Signal.html
    // You might want to use something that can notify multiple consumers
    //
    // Bonus Task: After cycling the LED colors, cycle the brightness

    loop {
        defmt::info!("Bing!");
        Timer::after(Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task]
async fn smartled_task(
    mut led: SmartLedsAdapterAsync<'static, BUFFER_SIZE>,
    mut watch_rx: Receiver<'static, CriticalSectionRawMutex, (), 2>,
) {
    loop {
        for color in [RED, GREEN, BLUE] {
            // Wait for BUTTON_WATCH to be signaled
            watch_rx.changed().await;
            info!("Watch signal received. Changing color.");
            led.write(brightness([color].into_iter(), 100))
                .await
                .unwrap();
        }
    }
}

#[embassy_executor::task]
async fn userled_task(
    mut led: Output<'static>,
    mut watch_rx: Receiver<'static, CriticalSectionRawMutex, (), 2>,
) {
    loop {
        watch_rx.changed().await;
        info!("Watch signal received. Toggling LED.");
        led.toggle();
    }
}

#[embassy_executor::task]
async fn button_task(
    mut button: Input<'static>,
    watch_tx: Sender<'static, CriticalSectionRawMutex, (), 2>,
) {
    loop {
        info!("Waiting for button falling edge");

        // Await both Futures. If either of them yields, the select-Future yields.
        match select(button.wait_for_falling_edge(), Timer::after_secs(10)).await {
            Either::First(_) => info!("Button pressed!"),
            Either::Second(_) => info!("No button press for 10 seconds. Signalling anyway."),
        }
        watch_tx.send(());
    }
}
