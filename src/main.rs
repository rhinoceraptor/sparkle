use std::thread;
use std::time::Duration;
use std::error::Error;
use std::sync::{Arc, Mutex};
use log::*;

use esp32_nimble::{BLEDevice, BLEScan};
use esp32_nimble::utilities::BleUuid;
// use esp_idf_sys as _;
// use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::hal::peripherals::Peripherals;

pub mod display;
pub mod spark_message;

const SPARK_SERVICE_UUID       : BleUuid = BleUuid::Uuid16(0xFFC0);
const SPARK_BLE_WRITE_CHAR_UUID: BleUuid = BleUuid::Uuid16(0xFFC1);
const SPARK_BLE_NOTIF_CHAR_UUID: BleUuid = BleUuid::Uuid16(0xFFC2);

fn main() -> anyhow::Result<(), Box<dyn Error>> {
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take()?;
    EspLogger::initialize_default();

    let mut my_display = display::Display::new(peripherals)?;
    my_display.display_text("Starting scan")?;

    let display = Arc::new(Mutex::new(my_display));

    block_on(async {
        let ble_device = BLEDevice::take();
        let mut ble_scan = BLEScan::new();

        let dev = ble_scan
            .active_scan(true)
            .interval(100)
            .window(99)
            .start(ble_device, 10000, |device, data| {
            if data.is_advertising_service(&SPARK_SERVICE_UUID) {
                info!("Found device {:?} {:?}", device, data);

                return Some(*device);
            }
            None
        }).await?;

        if let Some(dev) = dev {
            let mut client = ble_device.new_client();
            client.on_connect(|client| {
                info!("Connected");
                logger().flush();
                client.update_conn_params(120, 120, 0, 60).unwrap();
            });
            client.connect(&dev.addr()).await?;

            let service = client.get_service(SPARK_SERVICE_UUID).await?;
            let characteristic = service.get_characteristic(SPARK_BLE_NOTIF_CHAR_UUID).await?;


            let mut cccd = characteristic.get_descriptor(BleUuid::Uuid16(0x2902)).await?;
            cccd.write_value(&[0x1, 0x0], true).await?;

            if characteristic.can_notify() {
                info!("subscribing");
                logger().flush();
                characteristic.on_notify(move |data: &[u8]| {
                    let decoder = spark_message::SparkMsgDecoder;
                    let msg = decoder.decode(&data);
                    match msg {
                        Some(spark_message::SparkToAppMsg::AmpName { sequence, name }) => {
                            let mut d = display.lock().unwrap();
                            d.display_text(&format!("Connected to:\n{}", name));
                        }
                        _ => {}
                    }
                    ()
                }).subscribe_notify(false)
                .await?;
            }

            info!("Send GetAmpType");
            logger().flush();

            let mut encoder = spark_message::SparkMsgEncoder::new();
            let msg = spark_message::AppToSparkMsg::GetAmpName{};
            let blocks = encoder.encode(msg);


            let write_characteristic = service.get_characteristic(SPARK_BLE_WRITE_CHAR_UUID).await?;

            loop {
                let blocks = encoder.encode(msg);
                for block in blocks {
                    info!("Send block {:02X?}", block);
                    logger().flush();
                    write_characteristic.write_value(&block, false).await?;
                }

                thread::sleep(Duration::from_millis(8000));
            }

        } else {
            info!("No dev");
        }

        anyhow::Ok(())
    })?;

    let brk = false;

    loop {
        info!("loop");
        thread::sleep(Duration::from_millis(1000));

        if brk {
            break;
        }
    }

    Ok(())
}
