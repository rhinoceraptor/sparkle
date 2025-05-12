use esp_println::println;
use bt_hci::cmd::le::LeSetScanParams;
use bt_hci::controller::{Controller, ControllerCmdSync};
use bt_hci::controller::ExternalController;
use core::cell::RefCell;
use embassy_futures::join::join;
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

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;
const SERVICE_UUID: u32 = 0xFFC0;
const SERVICE_UUID2: u32 = 0xC0FF;

pub async fn run(connector: BleConnector<'_>) {
    // Using a fixed "random" address can be useful for testing. In real scenarios, one would
    // use e.g. the MAC 6 byte array as the address (how to get that varies by the platform).
    let address: Address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);

    println!("Our address = {:02X?}", address);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> = HostResources::new();
    let controller: ExternalController<_, 20> = ExternalController::new(connector);
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);

    let Host {
        central, mut runner, ..
    } = stack.build();

    let printer = Printer {
        seen: RefCell::new(Deque::new()),
    };
    let mut scanner = Scanner::new(central);
    let _ = join(runner.run_with_handler(&printer), async {
        let mut config = ScanConfig::default();
        config.active = true;
        config.phys = PhySet::M1;
        config.interval = Duration::from_secs(1);
        config.window = Duration::from_secs(1);
        let mut _session = scanner.scan(&config).await.unwrap();
        // Scan forever
        loop {
            Timer::after(Duration::from_secs(1)).await;
        }
    })
    .await;
}

struct Printer {
    seen: RefCell<Deque<BdAddr, 128>>,
}

impl EventHandler for Printer {
    fn on_adv_reports(&self, mut it: LeAdvReportsIter<'_>) {
        let mut seen = self.seen.borrow_mut();
        while let Some(Ok(report)) = it.next() {
            if seen.iter().find(|b| b.raw() == report.addr.raw()).is_none() {
                let one = BdAddr::new([0xBA, 0x79, 0x33, 0xED, 0xEB, 0xF7]);
                let two = BdAddr::new([0xF7, 0xEB, 0xED, 0x33, 0x79, 0xBA]);
                if report.addr == one || report.addr == two {
                    println!("discovered: {:02X?}", report.addr);
                    println!("data:\n{:02X?}", report.data);
                    let ad = AdvertisementData::new_from_bytes(report.data);
                    println!("{:02X?}", ad.service_uuids_32);

                    println!("SERVICE_UUID {:02X?}", SERVICE_UUID);
                    println!("SERVICE_UUID2 {:02X?}", SERVICE_UUID2);
                    if ad.is_advertising_service(SERVICE_UUID) || ad.is_advertising_service(SERVICE_UUID2) {
                        println!("ADVERTISING!!!!!!!!!!!");
                    }
                }

                if seen.is_full() {
                    seen.pop_front();
                }
                seen.push_back(report.addr).unwrap();
            }
        }
    }
}
