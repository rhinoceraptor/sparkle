#![no_std]
use display_interface_spi::SPIInterface;

pub struct Display {
    display: Ssd1306Async<
        SPIInterface,
        DisplaySize128x64,
        BufferedGraphicsModeAsync<DisplaySize128x64>,
    >
}

#[derive(Debug)]
pub enum DisplayError {
    ResetFailed,
    InitFailed,
}

impl Display {
    pub async fn new(
        spi: SPIBUS,
        mut reset: Output<'a>,
        mut dc: Output<'a>
) -> Result<Self, DisplayError> {
        let interface = SPIInterface::new(spi_dev, dc);
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
            display,
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

