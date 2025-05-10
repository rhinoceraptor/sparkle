#![no_std]
#![no_main]
extern crate alloc;

use bleps::{
    asynch::Ble,
};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    clock::CpuClock,
    timer::timg::TimerGroup,
    time,
};
use esp_println::println;
use esp_wifi::{
    EspWifiController,
    ble::controller::BleConnector,
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub mod spark_message;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.3.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let esp_wifi_ctrl = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let connector = BleConnector::new(&esp_wifi_ctrl, peripherals.BT);
    let now = || time::Instant::now().duration_since_epoch().as_millis();
    let mut ble = Ble::new(connector, now);
    println!("{:?}", ble.init().await);

    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        println!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}
