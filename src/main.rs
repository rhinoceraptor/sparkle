#![no_std]
#![no_main]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use esp_println as _;
use embassy_time::{Duration, Timer};
use embassy_executor::Spawner;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use defmt;

mod ble;
mod display;
mod spark_message;

pub type DisplayString = arrayvec::ArrayString<40>;
static CHANNEL: Channel<CriticalSectionRawMutex, DisplayString, 40> = Channel::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    defmt::info!("Hello world");
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let timg1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg0.timer0);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    spawner.spawn(ble::run(
        timg0.timer1,
        peripherals.RNG,
        peripherals.RADIO_CLK,
        peripherals.BT,
        CHANNEL.sender(),
    )).unwrap();

    // +-------+------+------+---------+
    // | ESP32 |      |      | Display |
    // | WROOM | GPIO | VSPI |   pin   |
    // +-------+------+------+---------+
    // |  D18  |  18  | CLK  |   SCL   |
    // |  D19  |  19  | MISO |   SDA   |
    // |  D23  |  23  | MOSI |   N/A   |
    // |  D5   |  5   | CS   |   CS    |
    // |  D4   |  4   |      |   RES   |
    // |  D2   |  2   |      |   DC    |
    // +-------+------+------+---------+

    spawner.spawn(display::run(
        peripherals.GPIO18, // sclk
        peripherals.GPIO19, // mosi
        peripherals.GPIO4,  // rst
        peripherals.GPIO5,  // cs
        peripherals.GPIO2,  // dc
        peripherals.GPIO15, // Backlight
        peripherals.GPIO21, // miso
        timg1.timer0,       // timer
        peripherals.SPI3,   // spi
        CHANNEL.receiver(),
    )).unwrap();

    loop {
        defmt::info!("Main loop");
        Timer::after(Duration::from_secs(10)).await;
    }
}
