#![no_std]
#![no_main]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

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
use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
    primitives::{PrimitiveStyle, Rectangle},
    Drawable,
};
use ssd1306::mode::BufferedGraphicsModeAsync;
use ssd1306::size::DisplaySize128x64;
use ssd1306::prelude::*;
use ssd1306::Ssd1306Async;


mod ble;
// mod display;
// mod spark_message;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    Timer::after(Duration::from_millis(100)).await;

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
    let mut rst   = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let cs        = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let dc        = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    let config = Config::default().with_frequency(Rate::from_khz(100)).with_mode(Mode::_0);
    let spi = Spi::new(peripherals.SPI3, config)
        .unwrap()
        .with_sck(sclk)
        .with_mosi(mosi)
        .into_async();

    let spi = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let spi = display_interface_spi::SPIInterface::new(spi, dc);

    let mut display = Ssd1306Async::new(
        spi,
        DisplaySize128x64,
        DisplayRotation::Rotate0
    ).into_buffered_graphics_mode();

    let timg1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    let mut delay = OneShotTimer::new(timg1.timer0).into_async();
    display.reset(&mut rst, &mut delay).await.unwrap();

    display.init().await.unwrap();
    let clear = Rectangle::new(
        Point::new(0, 0),
        display.bounding_box().size,
    )
    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off));

    let text_style = MonoTextStyle::new(&FONT_9X15, BinaryColor::On);
    let display_text = "\nHello world";

    clear.draw(&mut display).unwrap();
    Text::new(&display_text, Point::zero(), text_style).draw(&mut display).unwrap();
    display.flush().await.unwrap();

    spawner.spawn(ble::run(timg0.timer1, peripherals.RNG, peripherals.RADIO_CLK, peripherals.BT)).unwrap();

    // spawner.spawn(display::controller::start(display)).unwrap();
    // spawner.spawn(ble::start(peripherals.BT, init)).unwrap();

    loop {
        println!("Hello world");
        Timer::after(Duration::from_secs(1)).await;
    }
}
