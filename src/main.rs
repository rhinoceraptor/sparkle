#![allow(warnings)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

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
use alloc::vec::Vec;
use alloc::string::String;

pub mod spark_message;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

// {0xB4, 0xE6, 0x2D, 0xB2, 0x1B, 0x36}
// Define our custom BLE scanner struct
pub struct BleScanner<'a> {
    ble: Ble<'a>,
    target_service_uuid: u16,
    discovered_devices: Vec<DiscoveredDevice>,
}

// Structure to hold discovered device information
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    address: [u8; 6],
    address_type: PeerAddressType,
    rssi: i8,
    has_target_service: bool,
    ad_data: Vec<u8>,
    local_name: Option<String>,
}

impl<'a> BleScanner<'a> {
    pub fn new(connector: &'a dyn HciConnection, target_service_uuid: u16) -> Self {
        BleScanner {
            ble: Ble::new(connector),
            target_service_uuid,
            discovered_devices: Vec::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), Error> {
        // Initialize the BLE controller
        self.ble.init()?;

        // Read the Bluetooth address (optional, but useful for debugging)
        match self.ble.cmd_read_br_addr() {
            Ok(addr) => {
                println!("Local Bluetooth address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    addr[5], addr[4], addr[3], addr[2], addr[1], addr[0]);
            }
            Err(e) => {
                println!("Failed to read Bluetooth address: {:?}", e);
            }
        }

        Ok(())
    }

    // pub fn start_scanning(&mut self) -> Result<(), Error> {
    //     // Set scan parameters
    //     self.set_scan_parameters(true, 0x0010, 0x0010, false, false)?;

    //     // Enable scanning
    //     self.set_scan_enable(true, false)?;

    //     println!("Scanning started for service UUID 0x{:04X}...", self.target_service_uuid);
    //     Ok(())
    // }

    // pub fn stop_scanning(&mut self) -> Result<(), Error> {
    //     // Disable scanning
    //     self.set_scan_enable(false, false)?;

    //     println!("Scanning stopped");
    //     Ok(())
    // }

    // pub fn set_scan_parameters(
    //     &mut self,
    //     active_scanning: bool,
    //     scan_interval: u16,
    //     scan_window: u16,
    //     own_address_random: bool,
    //     filter_policy: bool,
    // ) -> Result<EventType, Error> {
    //     self.ble.write_bytes(
    //         Command::LeSetScanParameters {
    //             active: active_scanning,
    //             interval: scan_interval,
    //             window: scan_window,
    //             own_address_type: if own_address_random { 1 } else { 0 },
    //             filter_policy: if filter_policy { 1 } else { 0 },
    //         }
    //         .encode()
    //         .as_slice(),
    //     );

    //     self.ble
    //         .wait_for_command_complete(LE_OGF, SET_SCAN_PARAMETERS_OCF)?
    //         .check_command_completed()
    // }

    // pub fn set_scan_enable(
    //     &mut self,
    //     enable: bool,
    //     filter_duplicates: bool,
    // ) -> Result<EventType, Error> {
    //     self.ble.write_bytes(
    //         Command::LeSetScanEnable {
    //             enable,
    //             filter_duplicates,
    //         }
    //         .encode()
    //         .as_slice(),
    //     );

    //     self.ble
    //         .wait_for_command_complete(LE_OGF, SET_SCAN_ENABLE_OCF)?
    //         .check_command_completed()
    // }
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
                AdStructure::CompleteLocalName(),
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
