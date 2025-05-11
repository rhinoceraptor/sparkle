#![allow(dead_code)]
#![allow(unused_imports)]
#![no_std]
#![no_main]

// use core::alloc::Layout;
// use alloc::vec::Vec;
// use alloc::string::String;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{
    Input,
    InputConfig,
    Level,
    Output,
    OutputConfig,
    Pin,
};
use esp_hal::spi::{
    Mode,
    master::{
        Config,
        Spi,
    },
};
use esp_hal::time::Rate;
use esp_hal::{
    Async,
    clock::CpuClock,
    timer::timg::TimerGroup,
};
use esp_println as _;
use static_cell::StaticCell;

mod ble;
mod display;
mod spark_message;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let timer_group = TimerGroup::new(peripherals.TIMG0);

    let init = esp_wifi::init(
        timer_group.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    esp_hal_embassy::init(timer_group.timer1);

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

    let sclk      = peripherals.GPIO18;
    let mosi      = peripherals.GPIO19;
    let rst       = peripherals.GPIO4;
    let _dc        = peripherals.GPIO2;
    let mut dc    = Output::new(_dc, Level::Low, OutputConfig::default());
    let mut reset = Output::new(rst, Level::Low, OutputConfig::default());

    let config = Config::default().with_frequency(Rate::from_khz(100)).with_mode(Mode::_0);
    let spi = Spi::new(peripherals.SPI3, config)
        .unwrap()
        .with_sck(sclk)
        .with_mosi(mosi)
        .into_async();

    let display = display::Display::new(
        spi,
        reset,
        dc,
    ).await.unwrap();

    // spawner.spawn(display::controller::start(display)).unwrap();
    spawner.spawn(ble::start(peripherals.BT, init)).unwrap();
}
