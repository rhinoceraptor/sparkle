use std::error::Error;
use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::*;
use esp_idf_svc::hal::units::FromValueType;
use log::*;

use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
    primitives::{PrimitiveStyle, Rectangle},
    Drawable,
};

use ssd1306::{prelude::*, Ssd1306};
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::size::DisplaySize128x64;
use display_interface_spi::{SPIInterface};

pub struct Display {
    // all other pins are moved into SpiDeviceDriver, but not RST so we need to own it
    rst: PinDriver<'static, Gpio21, Output>,
    display: Ssd1306<
        SPIInterface<
            SpiDeviceDriver<'static, SpiDriver<'static>>,
            PinDriver<'static, Gpio19, Output>
        >,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >
}

impl Display {
    pub fn new(peripherals: Peripherals) -> anyhow::Result<Display, Box<dyn Error>> {
        let spi  = peripherals.spi2;
        let rst  = PinDriver::output(peripherals.pins.gpio21)?;
        let dc   = PinDriver::output(peripherals.pins.gpio19)?;
        let sclk = peripherals.pins.gpio18;
        let sda  = peripherals.pins.gpio23;
        let cs   = peripherals.pins.gpio5;

        let config = config::Config::new()
            .baudrate(10.MHz().into());

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
        let mut display = Display {
            rst: rst,
            display: Ssd1306::new(spi_interface, DisplaySize128x64, DisplayRotation::Rotate0)
                .into_buffered_graphics_mode(),
        };

        let mut delay: Delay = Default::default();
        display.display.reset(&mut display.rst, &mut delay).unwrap();
        display.display.init().unwrap();

        Ok(display)
    }

    pub fn display_text(&mut self, text: &str) -> anyhow::Result<(), Box<dyn Error>> {
        info!("{}", text);
        let clear = Rectangle::new(
            Point::new(0, 0),
            self.display.bounding_box().size,
        )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off));

        let text_style = MonoTextStyle::new(&FONT_9X15, BinaryColor::On);
        let display_text = format!("\n{}", text);

        clear.draw(&mut self.display).unwrap();
        Text::new(&display_text, Point::zero(), text_style).draw(&mut self.display).unwrap();
        self.display.flush().unwrap();
        Ok(())
    }
}

