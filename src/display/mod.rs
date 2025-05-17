use embassy_time::{Duration, Timer};
use esp_hal::{
    spi::{Mode, master::{Config, Spi}},
    gpio::{AnyPin, Level, Output, OutputConfig},
    time::Rate,
    timer::OneShotTimer,
    peripherals::SPI3,
};
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::Receiver;
use esp_println as _;
use esp_hal::timer::timg::Timer as EspTimer;
use profont::PROFONT_24_POINT;
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use mipidsi::interface::SpiInterface;
use mipidsi::{
    Builder,
    models::ST7796,
    options::{
        ColorInversion,
        Rotation,
        Orientation,
        ColorOrder,
    },
};

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
mod font;

// Display
const H: i32 = 480;
const W: i32 = 320;

#[embassy_executor::task]
pub async fn run(
    sclk:      AnyPin<'static>,
    mosi:      AnyPin<'static>,
    rst:       AnyPin<'static>,
    cs:        AnyPin<'static>,
    dc:        AnyPin<'static>,
    backlight: AnyPin<'static>,
    miso:      AnyPin<'static>,
    timer:     EspTimer<'static>,
    spi:       SPI3<'static>,
    channel: Receiver<'static, CriticalSectionRawMutex, arrayvec::ArrayString<40>, 40>,
) {

    let     rst       = Output::new(rst,       Level::Low, OutputConfig::default());
    let cs            = Output::new(cs,        Level::Low, OutputConfig::default());
    let dc            = Output::new(dc,        Level::Low, OutputConfig::default());
    let mut backlight = Output::new(backlight, Level::Low, OutputConfig::default());

    let config = Config::default().with_frequency(Rate::from_khz(62500)).with_mode(Mode::_0);
    let spi = Spi::new(spi, config)
        .unwrap()
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_miso(miso);
        // .into_async();

    let spi = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, NoCs).unwrap();
    let mut buffer = [0_u8; 512];
    let di = SpiInterface::new(spi, dc, &mut buffer);
    let mut delay = OneShotTimer::new(timer);
    let mut display = Builder::new(ST7796, di)
        .reset_pin(rst)
        .display_size(W as u16, H as u16)
        .color_order(ColorOrder::Bgr)
        .orientation(Orientation::new().rotate(Rotation::Deg90).flip_horizontal())
        .init(&mut delay)
        .unwrap();

    let text_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::WHITE);
    let text = "Hello World ^_^;";
    let text_x: i32 = 100;
    let text_y: i32 = 100;

    backlight.set_level(Level::High);

    display.clear(Rgb565::BLACK).unwrap();
    let right = Text::new(text, Point::new(text_x, text_y), text_style)
        .draw(&mut display)
        .unwrap();

    loop {
        let display_text = channel.receive().await.to_string();
        let text_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::WHITE);
        display.clear(Rgb565::BLACK).unwrap();
        Text::new(&display_text, Point::new(text_x, text_y), text_style)
            .draw(&mut display)
            .unwrap();
    }
}

struct NoCs;

impl embedded_hal::digital::OutputPin for NoCs {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl embedded_hal::digital::ErrorType for NoCs {
    type Error = core::convert::Infallible;
}
