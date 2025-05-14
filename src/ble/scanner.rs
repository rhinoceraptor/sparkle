use defmt;
use bt_hci::param::{AddrKind, BdAddr};
use bt_hci::controller::ExternalController;
use core::cell::RefCell;
use embassy_futures::select::select;
use embassy_futures::select::Either::Second;
use embassy_time::{Duration, Timer};
use esp_println as _;
// use esp_backtrace as _;
use trouble_host::scan::{LeAdvReportsIter, Scanner};
use trouble_host::connection::{PhySet, ScanConfig};
use trouble_host::{Host, HostResources};
use trouble_host::prelude::*;
use trouble_host::Address;
use esp_wifi::ble::controller::BleConnector;
use super::advertisement::AdvertisementData;

use super::SPARK_SERVICE_UUID;

pub struct ScanHandler {
    device: RefCell<Option<(AddrKind, BdAddr)>>
}

impl ScanHandler {
    pub fn new() -> Self {
        Self {
            device: RefCell::new(None)
        }
    }

    pub fn get_device(&self) -> Option<(AddrKind, BdAddr)> {
        self.device.borrow().clone()
    }

    pub fn found_device(&self) -> bool {
        self.get_device().is_some()
    }
}

impl EventHandler for ScanHandler {
    fn on_adv_reports(&self, mut it: LeAdvReportsIter<'_>) {
        while let Some(Ok(report)) = it.next() {
            let ad = AdvertisementData::new_from_bytes(report.data);
            if ad.is_advertising_service(SPARK_SERVICE_UUID) {
                defmt::info!("Found address");
                // defmt::info!("Found address {:?} advertising {:02X?}", report.addr, SPARK_SERVICE_UUID);
                let mut device = self.device.borrow_mut();
                *device = Some((report.addr_kind, report.addr));
            }
        }
    }
}
