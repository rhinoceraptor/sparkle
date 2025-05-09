use std::error::Error;
use esp_idf_hal::gpio::{AnyInputPin, Gpio19, Output, PinDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::units::FromValueType;
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

pub struct Display<'a> {
    display: Ssd1306<
        SPIInterface<
            SpiDeviceDriver<'a, SpiDriver<'a>>,
            PinDriver<'a, Gpio19, Output>
        >,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >
}

impl<'a> Display<'a> {
    pub fn new(peripherals: Peripherals) -> anyhow::Result<Display<'a>, Box<dyn Error>> {
        let spi = peripherals.spi2;
        let mut rst  = PinDriver::output(peripherals.pins.gpio21)?;
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

        rst.set_low()?;
        Ets::delay_ms(50);
        rst.set_high()?;
        Ets::delay_ms(50);

        let mut display = Display {
            display: Ssd1306::new(spi_interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode(),
        };

        display.display.init().unwrap();
        display.display.flush().unwrap();

        // let clear = Rectangle::new(
        //     Point::new(0, 0),
        //     display.display.bounding_box().size,
        // )
        // .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off));

        // clear.draw(&mut display.display).unwrap();
        // display.display.flush().unwrap();

        let text_style = MonoTextStyle::new(&FONT_9X15, BinaryColor::On);
        info!("hello world!");
        let display_text = format!("\nhello world!!");
        Text::new(&display_text, Point::zero(), text_style).draw(&mut display.display).unwrap();
        display.display.flush().unwrap();

        Ok(display)
    }

    pub fn display_text(&mut self, text: &str) -> anyhow::Result<(), Box<dyn Error>> {
        info!("{}", text);
        // let clear = Rectangle::new(
        //     Point::new(0, 0),
        //     self.display.bounding_box().size,
        // )
        // .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off));

        // clear.draw(&mut self.display).unwrap();
        // self.display.flush().unwrap();

        let text_style = MonoTextStyle::new(&FONT_9X15, BinaryColor::On);
        let display_text = format!("\n{}", text);
        Text::new(&display_text, Point::zero(), text_style).draw(&mut self.display).unwrap();
        self.display.flush().unwrap();
        Ok(())
    }
}

