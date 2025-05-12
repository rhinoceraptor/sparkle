mod advertisement;
mod scanner;

use embassy_time::{Duration, Timer};
use esp_hal::timer::timg::Timer as EspTimer;
use esp_hal::peripherals::{RNG, RADIO_CLK, BT};
use esp_println::println;
use esp_wifi::ble::controller::BleConnector;

#[embassy_executor::task]
pub async fn run(
    timer: EspTimer<'static>,
    rng: RNG<'static>,
    clk: RADIO_CLK<'static>,
    bt: BT<'static>
) {
    let init = esp_wifi::init(
        timer,
        esp_hal::rng::Rng::new(rng),
        clk,
    )
    .unwrap();

    let connector = BleConnector::new(&init, bt);

    let (addr_kind, addr) = scanner::run(connector).await.unwrap();
    println!("addr_kind: {:?} addr: {:?}", addr_kind, addr);

    loop {
        println!("BLE loop");
        Timer::after(Duration::from_secs(10)).await;
    }
}
