use embassy_time::{Duration, Timer};
use esp_hal::{
    spi::{Mode, master::{Config, Spi}},
    gpio::{GpioPin, Level, Output, OutputConfig},
    time::Rate,
    timer::OneShotTimer,
    peripherals::SPI3,
};
use esp_hal::timer::timg::Timer as EspTimer;
use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
    primitives::{PrimitiveStyle, Rectangle},
    Drawable,
};
use ssd1306::size::DisplaySize128x64;
use ssd1306::prelude::*;
use ssd1306::Ssd1306Async;
extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
mod font;

#[embassy_executor::task]
pub async fn run(
    sclk:  GpioPin<'static, 18>,
    mosi:  GpioPin<'static, 19>,
    rst:   GpioPin<'static, 4>,
    cs:    GpioPin<'static, 5>,
    dc:    GpioPin<'static, 2>,
    timer: EspTimer<'static>,
    spi:   SPI3<'static>,
) {

    let mut rst = Output::new(rst, Level::Low, OutputConfig::default());
    let cs      = Output::new(cs, Level::Low, OutputConfig::default());
    let dc      = Output::new(dc, Level::Low, OutputConfig::default());

    let config = Config::default().with_frequency(Rate::from_khz(100)).with_mode(Mode::_0);
    let spi = Spi::new(spi, config)
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

    let mut delay = OneShotTimer::new(timer).into_async();
    display.reset(&mut rst, &mut delay).await.unwrap();

    display.init().await.unwrap();

    let clear = Rectangle::new(
        Point::new(0, 0),
        display.bounding_box().size,
    )
    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off));

    // let text_style = MonoTextStyle::new(&FONT_9X15, BinaryColor::On);
    // let display_text = "\nHello world";

    clear.draw(&mut display).unwrap();
    // Text::new(&display_text, Point::zero(), text_style).draw(&mut display).unwrap();
    display.flush().await.unwrap();

    // let mut counter: u8 = 0;

    Image::new(&ImageRaw::<BinaryColor>::new(&font::H,     5), Point::new(16, 16)).draw(&mut display).unwrap();
    Image::new(&ImageRaw::<BinaryColor>::new(&font::e,     5), Point::new(32, 16)).draw(&mut display).unwrap();
    Image::new(&ImageRaw::<BinaryColor>::new(&font::l,     5), Point::new(48, 16)).draw(&mut display).unwrap();
    // Image::new(&ImageRaw::<BinaryColor>::new(&font::l,     5), Point::new(29, 16)).draw(&mut display).unwrap();
    // Image::new(&ImageRaw::<BinaryColor>::new(&font::o,     5), Point::new(34, 16)).draw(&mut display).unwrap();
    // Image::new(&ImageRaw::<BinaryColor>::new(&font::space, 5), Point::new(39, 16)).draw(&mut display).unwrap();
    // Image::new(&ImageRaw::<BinaryColor>::new(&font::W,     5), Point::new(44, 16)).draw(&mut display).unwrap();
    // Image::new(&ImageRaw::<BinaryColor>::new(&font::o,     5), Point::new(49, 16)).draw(&mut display).unwrap();
    // Image::new(&ImageRaw::<BinaryColor>::new(&font::r,     5), Point::new(54, 16)).draw(&mut display).unwrap();
    // Image::new(&ImageRaw::<BinaryColor>::new(&font::l,     5), Point::new(59, 16)).draw(&mut display).unwrap();
    // Image::new(&ImageRaw::<BinaryColor>::new(&font::d,     5), Point::new(64, 16)).draw(&mut display).unwrap();

    display.flush().await.unwrap();

    loop {
        Timer::after(Duration::from_secs(1)).await;
        // let text_style = MonoTextStyle::new(&FONT_9X15, BinaryColor::On);

        // clear.draw(&mut display).unwrap();
        // Text::new(&display_text, Point::zero(), text_style).draw(&mut display).unwrap();
        // let mut display_text = String::from("\nHello world\n");
        // display_text.push_str(&counter.to_string());

        // Text::new(&display_text, Point::zero(), text_style).draw(&mut display).unwrap();
        // display.flush().await.unwrap();

        // if counter == 10 {
        //     counter = 0;
        // } else {
        //     counter += 1;
        // }
    }
}
