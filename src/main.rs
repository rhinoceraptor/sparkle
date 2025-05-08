use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::{Gpio18, Gpio19, Gpio23, Gpio5, Output, PinDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use ssd1306::{prelude::*, Ssd1306};
use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

fn main() -> anyhow::Result<()> {
    // Initialize peripherals
    let peripherals = Peripherals::take()?;
    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio18; // SPI CLK
    let mosi = peripherals.pins.gpio23; // SPI MOSI
    let cs = peripherals.pins.gpio5;    // Chip Select
    let dc = peripherals.pins.gpio19;   // Data/Command
    let rst = peripherals.pins.gpio21;  // Reset (optional)

    // Initialize SPI
    let spi = SpiDeviceDriver::new_single(
        spi,
        sclk,
        mosi,
        None, // MISO not used
        &SpiConfig::new().baudrate(10.MHz().into()),
    )?;

    // Prepare pins
    let mut dc = PinDriver::output(dc)?;
    let mut rst = PinDriver::output(rst)?;
    let cs = PinDriver::output(cs)?;

    // Create SPI interface without automatic CS control
    let spi_interface = SPIInterfaceNoCS::new(spi, dc);

    // Reset display manually
    rst.set_low()?;
    Ets::delay_ms(50);
    rst.set_high()?;
    Ets::delay_ms(50);

    // Initialize display
    let mut display = Ssd1306::new(spi_interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init()?;
    display.flush()?;

    // Draw "Hello, world!"
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("Hello, world!", Point::zero(), text_style).draw(&mut display)?;
    display.flush()?;
}
