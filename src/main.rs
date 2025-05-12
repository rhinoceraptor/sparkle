#![no_std]
#![no_main]
// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(dead_code)]

use embassy_time::{Duration, Timer};
use embassy_executor::Spawner;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_println::println;

mod ble;
mod display;
// mod spark_message;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let timg1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg0.timer0);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    spawner.spawn(ble::run(
        timg0.timer1,
        peripherals.RNG,
        peripherals.RADIO_CLK,
        peripherals.BT
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
        timg1.timer0,       // timer
        peripherals.SPI3,   // spi
    )).unwrap();

    loop {
        println!("Main loop");
        Timer::after(Duration::from_secs(10)).await;
    }
}
