#![no_std]
#![no_main]
// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(dead_code)]

use embassy_time::{Duration, Timer};
use embassy_executor::Spawner;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    Async,
    spi::{Mode, master::{Config, Spi}},
    gpio::{Level, Input, InputConfig, Output, OutputConfig, Pull},
    clock::CpuClock,
    time::Rate,
    timer::OneShotTimer,
    peripherals::Peripherals,
};
use esp_println::println;
use esp_wifi::{
    init,
    ble::controller::BleConnector,
};

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

    // Initialize SPI
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

    let sclk  = peripherals.GPIO18;
    let mosi  = peripherals.GPIO19;
    let rst   = peripherals.GPIO4;
    let cs    = peripherals.GPIO5;
    let dc    = peripherals.GPIO2;
    let timer = timg1.timer0;
    let spi   = peripherals.SPI3;

    spawner.spawn(display::run(
        sclk,
        mosi,
        rst,
        cs,
        dc,
        timer,
        spi,
    )).unwrap();

    loop {
        println!("Hello world");
        Timer::after(Duration::from_secs(1)).await;
    }
}
