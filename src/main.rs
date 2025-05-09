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

const SPARK_SERVICE_UUID       : BleUuid = BleUuid::Uuid16(0xFFC0);
const SPARK_BLE_WRITE_CHAR_UUID: BleUuid = BleUuid::Uuid16(0xFFC1);
const SPARK_BLE_NOTIF_CHAR_UUID: BleUuid = BleUuid::Uuid16(0xFFC2);
const SPARK_RECV_NOTIF_CHARACTERISTIC: BleUuid = BleUuid::Uuid16(0x2902);

fn main() -> anyhow::Result<(), Box<dyn Error>> {
    esp_idf_sys::link_patches();
    // esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take()?;

    EspLogger::initialize_default();

    let mut my_display = display::Display::new(peripherals)?;
    thread::sleep(Duration::from_millis(10000));

    my_display.display_text("Starting BLE Spark Amp Connector")?;

    thread::sleep(Duration::from_millis(10000));


    my_display.display_text("Starting scan")?;

    // let found_devices = Arc::new(Mutex::new(Vec::new()));

    // block_on(async {
    //     let ble_device = BLEDevice::take();
    //     let mut ble_scan = BLEScan::new();
    //     let found_devices_clone = found_devices.clone();

    //     let dev = ble_scan
    //         .active_scan(true)
    //         .interval(100)
    //         .window(99)
    //         .start(ble_device, 10000, |device, data| {
    //         if data.is_advertising_service(&SPARK_SERVICE_UUID) {
    //             let mut list = found_devices_clone.lock().unwrap();
    //             list.push(*device);

    //             return Some(*device);
    //         }
    //         None
    //     }).await?;

    //     let _ = my_display.display_text("Scan done");
    //     let devices = found_devices.lock().unwrap();
    //     let _ = my_display.display_text(&format!("Found {} amps", devices.len()));
    //     for d in devices.iter() {
    //         let _ = my_display.display_text(&format!("Device {:?}", d));
    //     }

    //     if let Some(dev) = dev {
    //         let _ = my_display.display_text(&format!("Dev {:?}", dev));
    //         let mut client = ble_device.new_client();
    //         client.connect(&dev.addr()).await?;

    //         let service = client.get_service(SPARK_SERVICE_UUID).await?;
    //         let characteristic = service.get_characteristic(SPARK_BLE_NOTIF_CHAR_UUID).await?;
    //         if characteristic.can_notify() {
    //             info!("subscribing");
    //             let data = [0x1, 0x0];
    //             characteristic
    //                 .get_descriptor(SPARK_RECV_NOTIF_CHARACTERISTIC)
    //                 .await?
    //                 .write_value(&data, true)
    //                 .await?;
    //             characteristic.on_notify(|data| {
    //                 let text = core::str::from_utf8(data).unwrap();
    //                 info!("data: {}", text);
    //             }).subscribe_notify(false)
    //             .await?;
    //         }

    //     } else {
    //         info!("No dev");
    //     }

    //     anyhow::Ok(())
    // })?;
    let brk = false;

    loop {
        // let text_style = MonoTextStyle::new(&FONT_9X15, BinaryColor::On);
        // let text = format!("\nHello World");
        // Text::new(&text, Point::zero(), text_style).draw(&mut display).unwrap();
        // display.flush().unwrap();

        // thread::sleep(Duration::from_millis(3000));
        // let clear = Rectangle::new(
        //     Point::new(0, 0),
        //     display.bounding_box().size,
        // )
        // .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off));

        // clear.draw(&mut display).unwrap();
        // display.flush().unwrap();

        my_display.display_text("Loop")?;

        thread::sleep(Duration::from_millis(1000));

        if brk {
            break;
        }
    }

    Ok(())
}
