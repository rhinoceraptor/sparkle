#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use bleps::{
    Ble,
    HciConnector,
    ad_structure::{
        AdStructure,
        BR_EDR_NOT_SUPPORTED,
        LE_GENERAL_DISCOVERABLE,
        create_advertising_data,
    },
    attribute_server::{AttributeServer, NotificationData, WorkResult},
    gatt,
    att::Uuid,
};
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_time::{Duration, Timer};
use esp_hal::{
    clock::CpuClock,
    timer::timg::TimerGroup,
    time,
};
use esp_println::{
    println,
};
use esp_wifi::{
    EspWifiController,
    ble::controller::BleConnector,
};
use esp_alloc::EspHeap;
use core::alloc::Layout;

pub mod spark_message;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

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
    let hci = HciConnector::new(connector, now);
    let mut ble = Ble::new(&hci);
    println!("{:?}", ble.init());

    // TODO: Spawn some tasks
    let _ = spawner;

    println!("{:?}", ble.cmd_set_le_advertising_parameters());
    println!(
        "{:?}",
        ble.cmd_set_le_advertising_data(
            create_advertising_data(&[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
                AdStructure::CompleteLocalName(esp_hal::chip!()),
            ])
            .unwrap()
        )
    );
    println!("{:?}", ble.cmd_set_le_advertise_enable(true));

    println!("started advertising");

    let mut rf = |_offset: usize, data: &mut [u8]| {
        println!("RECEIVED: {} {:?}", _offset, data);
        data[..20].copy_from_slice(&b"Hello Bare-Metal BLE"[..]);
        17
    };
    let mut wf = |offset: usize, data: &[u8]| {
        println!("RECEIVED: {} {:?}", offset, data);
    };

    let mut wf2 = |offset: usize, data: &[u8]| {
        println!("RECEIVED: {} {:?}", offset, data);
    };

    let mut rf3 = |_offset: usize, data: &mut [u8]| {
        println!("RECEIVED: {} {:?}", _offset, data);
        data[..5].copy_from_slice(&b"Hola!"[..]);
        5
    };
    let mut wf3 = |offset: usize, data: &[u8]| {
        println!("RECEIVED: {} {:?}", offset, data);
        println!("RECEIVED: Offset {}, data {:?}", offset, data);
    };

    gatt!([service {
        uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
        characteristics: [
            characteristic {
                uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
                read: rf,
                write: wf,
            },
            characteristic {
                uuid: "957312e0-2354-11eb-9f10-fbc30a62cf38",
                write: wf2,
            },
            characteristic {
                name: "my_characteristic",
                uuid: "987312e0-2354-11eb-9f10-fbc30a62cf38",
                notify: true,
                read: rf3,
                write: wf3,
            },
        ],
    },]);

    let mut rng = bleps::no_rng::NoRng;
    let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut rng);

    loop {
        yield_now().await;
        match srv.do_work() {
            Ok(WorkResult::DidWork) => println!("DidWork"),
            Ok(WorkResult::GotDisconnected) => println!("BLE Disconnected!"),
            Err(err) => println!("Err: {:?}", err),
        };
        println!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
