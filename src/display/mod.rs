use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
use esp_hal::delay::Delay;
use esp_hal::Async;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;

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

pub struct Display<'a> {
    display: Ssd1306Async<
        SPIInterface<Spi<'a, Async>, Output<'a>>,
        DisplaySize128x64,
        BufferedGraphicsModeAsync<DisplaySize128x64>
    >
}

#[derive(Debug)]
pub enum DisplayError {
    ResetFailed,
    InitFailed,
}

impl<'a> Display<'a> {
    pub async fn new(spi: Spi<'a, Async>, mut reset: Output<'a>, mut dc: Output<'a>) -> Result<Display<'a>, DisplayError> {
        let interface = SPIInterface::new(spi, dc);
        let mut display = Ssd1306Async::new(
            interface,
            DisplaySize128x64,
            DisplayRotation::Rotate0
        ).into_buffered_graphics_mode();

        display.reset(&mut reset, &mut embassy_time::Delay {})
            .await
            .map_err(|_| DisplayError::ResetFailed)?;

        display.init().await.map_err(|_| DisplayError::InitFailed)?;

        Ok(Self {
            display
        })
    }

    // pub fn display_text(&mut self, text: &str) -> anyhow::Result<(), Box<dyn Error>> {
    //     info!("{}", text);
    //     let clear = Rectangle::new(
    //         Point::new(0, 0),
    //         self.display.bounding_box().size,
    //     )
    //     .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off));

    //     let text_style = MonoTextStyle::new(&FONT_9X15, BinaryColor::On);
    //     let display_text = format!("\n{}", text);

    //     clear.draw(&mut self.display).unwrap();
    //     Text::new(&display_text, Point::zero(), text_style).draw(&mut self.display).unwrap();
    //     self.display.flush().unwrap();
    //     Ok(())
    // }
}

