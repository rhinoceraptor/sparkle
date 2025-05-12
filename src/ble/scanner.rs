use esp_println::println;
use bt_hci::cmd::le::LeSetScanParams;
use bt_hci::controller::{Controller, ControllerCmdSync};
use bt_hci::controller::ExternalController;
use core::cell::RefCell;
use embassy_futures::select::select;
use embassy_futures::select::Either::Second;
use embassy_time::{Duration, Timer};
use heapless::Deque;
use esp_backtrace as _;
use trouble_host::scan::{LeAdvReportsIter, Scanner};
use trouble_host::connection::{PhySet, ScanConfig};
use trouble_host::{Host, HostResources};
use trouble_host::prelude::*;
use trouble_host::Address;
use esp_wifi::ble::controller::BleConnector;
// use trouble_host::packet_pool::DefaultPacketPool;
use super::advertisement::AdvertisementData;

// Max number of connections
const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;
const SERVICE_UUID: u16 = 0xFFC0;

pub async fn run(connector: BleConnector<'_>) -> Option<BdAddr> {
    // Using a fixed "random" address can be useful for testing. In real scenarios, one would
    // use e.g. the MAC 6 byte array as the address (how to get that varies by the platform).
    let address: Address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> = HostResources::new();
    let controller: ExternalController<_, 20> = ExternalController::new(connector);
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);

    let Host {
        central, mut runner, ..
    } = stack.build();

    let handler = Handler::new();

    let mut scanner = Scanner::new(central);
    let addr = select(runner.run_with_handler(&handler), async {
        let mut config = ScanConfig::default();
        config.active = true;
        config.phys = PhySet::M1;
        config.interval = Duration::from_secs(1);
        config.window = Duration::from_secs(1);
        let mut _session = scanner.scan(&config).await.unwrap();

        while !handler.found_device() {
            Timer::after(Duration::from_secs(1)).await;
        }

        handler.get_addr().unwrap()
    })
    .await;

    match addr {
        Second(a) => Some(a),
        _ => None,
    }
}

struct Handler {
    device: RefCell<Option<BdAddr>>
}

impl Handler {
    fn new() -> Self {
        Self {
            device: RefCell::new(None)
        }
    }

    fn get_addr(&self) -> Option<BdAddr> {
        self.device.borrow().clone()
    }

    fn found_device(&self) -> bool {
        self.get_addr().is_some()
    }
}

impl EventHandler for Handler {
    fn on_adv_reports(&self, mut it: LeAdvReportsIter<'_>) {
        while let Some(Ok(report)) = it.next() {
            let ad = AdvertisementData::new_from_bytes(report.data);
            if ad.is_advertising_service(SERVICE_UUID) {
                println!("Found address {:?} advertising {:02X?}", report.addr, SERVICE_UUID);
                let mut device = self.device.borrow_mut();
                *device = Some(report.addr);
            }
        }
    }
}
