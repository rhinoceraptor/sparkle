use std::thread;
use std::time::Duration;
use std::error::Error;
use std::sync::{Arc, Mutex};
use log::*;

// use uuid::Uuid;

use esp32_nimble::{BLEDevice, BLEScan};
use esp32_nimble::utilities::BleUuid;
use esp_idf_hal::task::block_on;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys as _;
// use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::log::EspLogger;

pub mod display;
pub mod spark_message;

const SPARK_SERVICE_UUID       : BleUuid = BleUuid::Uuid16(0xFFC0);
const SPARK_BLE_WRITE_CHAR_UUID: BleUuid = BleUuid::Uuid16(0xFFC1);
const SPARK_BLE_NOTIF_CHAR_UUID: BleUuid = BleUuid::Uuid16(0xFFC2);

fn main() -> anyhow::Result<(), Box<dyn Error>> {
    esp_idf_sys::link_patches();
    let peripherals = Peripherals::take()?;
    EspLogger::initialize_default();

    let mut my_display = display::Display::new(peripherals)?;
    my_display.display_text("Starting scan")?;

    let found_devices = Arc::new(Mutex::new(Vec::new()));

    block_on(async {
        let ble_device = BLEDevice::take();
        let mut ble_scan = BLEScan::new();
        let found_devices_clone = found_devices.clone();

        let dev = ble_scan
            .active_scan(true)
            .interval(100)
            .window(99)
            .start(ble_device, 10000, |device, data| {
            if data.is_advertising_service(&SPARK_SERVICE_UUID) {
                info!("Found device {:?} {:?}", device, data);
                let mut list = found_devices_clone.lock().unwrap();
                list.push(*device);

                return Some(*device);
            }
            None
        }).await?;

        if let Some(dev) = dev {
            let mut client = ble_device.new_client();
            client.on_connect(|client| {
                info!("Connected");
                client.update_conn_params(120, 120, 0, 60).unwrap();
            });
            client.connect(&dev.addr()).await?;

            let service = client.get_service(SPARK_SERVICE_UUID).await?;
            let characteristic = service.get_characteristic(SPARK_BLE_NOTIF_CHAR_UUID).await?;
            info!("characteristic: {:?}", characteristic);
            if characteristic.can_notify() {
                info!("subscribing");
                characteristic.on_notify(|data| {
                    let text = core::str::from_utf8(data).unwrap();
                    info!("data: {}", text);
                }).subscribe_notify(true)
                .await?;
            }

        } else {
            info!("No dev");
        }

        anyhow::Ok(())
    })?;

    let brk = false;

    loop {
        thread::sleep(Duration::from_millis(1000));

        if brk {
            break;
        }
    }

    Ok(())
}
