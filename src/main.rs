use std::thread;
use std::time::Duration;
use std::error::Error;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::units::FromValueType;
use esp_idf_hal::gpio::{AnyInputPin, PinDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use ssd1306::{prelude::*, Ssd1306};
use display_interface_spi::SPIInterface;

use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
    primitives::{PrimitiveStyle, Rectangle},
    Drawable,
};

fn main() -> anyhow::Result<(), Box<dyn Error>> {
    // Initialize peripherals
    let peripherals = Peripherals::take()?;
    let spi = peripherals.spi2;

    let mut rst  = PinDriver::output(peripherals.pins.gpio21)?;
    let dc   = PinDriver::output(peripherals.pins.gpio19)?;
    let sclk = peripherals.pins.gpio18;
    let sda  = peripherals.pins.gpio23;
    let cs   = peripherals.pins.gpio5;

    let config = config::Config::new()
        .baudrate(10.MHz().into());

    // Initialize SPI
    let device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sda,
        Option::<AnyInputPin>::None,
        Some(cs),
        &SpiDriverConfig::new(),
        &config,
    )?;

    let spi_interface = SPIInterface::new(device, dc);

    // Reset display manually
    rst.set_low()?;
    Ets::delay_ms(50);
    rst.set_high()?;
    Ets::delay_ms(50);

    // Initialize display
    let mut display = Ssd1306::new(spi_interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    display.flush().unwrap();

    let fonts = [
        FONT_4X6,
        FONT_5X7,
        FONT_5X8,
        FONT_6X10,
        FONT_6X12,
        FONT_6X13,
        FONT_6X13_BOLD,
        FONT_6X13_ITALIC,
        FONT_6X9,
        FONT_7X13,
        FONT_7X13_BOLD,
        FONT_7X13_ITALIC,
        FONT_7X14,
        FONT_7X14_BOLD,
        FONT_8X13,
        FONT_8X13_BOLD,
        FONT_8X13_ITALIC,
        FONT_9X15,
        FONT_9X15_BOLD,
        FONT_9X18,
        FONT_9X18_BOLD,
        FONT_10X20,
    ];

    let font_names = [
        "FONT_4X6",
        "FONT_5X7",
        "FONT_5X8",
        "FONT_6X10",
        "FONT_6X12",
        "FONT_6X13",
        "FONT_6X13_BOLD",
        "FONT_6X13_ITALIC",
        "FONT_6X9",
        "FONT_7X13",
        "FONT_7X13_BOLD",
        "FONT_7X13_ITALIC",
        "FONT_7X14",
        "FONT_7X14_BOLD",
        "FONT_8X13",
        "FONT_8X13_BOLD",
        "FONT_8X13_ITALIC",
        "FONT_9X15",
        "FONT_9X15_BOLD",
        "FONT_9X18",
        "FONT_9X18_BOLD",
        "FONT_10X20",
    ];

    let mut i = 0;

    loop {
        // Draw "Hello, world!"
        let text_style = MonoTextStyle::new(&fonts[i], BinaryColor::On);
        let text = format!("\n{}\nabcdefghijklmnopqrstuvwxyz\nNABCDEFGHIJKLMNOPQRSTUVWXYZ\n0123456789", &font_names[i]);
        Text::new(&text, Point::zero(), text_style).draw(&mut display).unwrap();
        display.flush().unwrap();
        thread::sleep(Duration::from_millis(1000));
        i += 1;
        if i == fonts.len() {
            i = 0;
        }

        let clear = Rectangle::new(
            Point::new(0, 0),
            display.bounding_box().size,
        )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off));

        clear.draw(&mut display).unwrap();
        display.flush().unwrap();
    }

    Ok(())
}
