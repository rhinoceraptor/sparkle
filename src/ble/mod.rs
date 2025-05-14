extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
mod advertisement;
mod scanner;

use esp_println as _;
use embassy_time::{Duration, Timer};
use esp_hal::timer::timg::Timer as EspTimer;
use esp_hal::peripherals::{RNG, RADIO_CLK, BT};
use bt_hci::param::{AddrKind, BdAddr};
use bt_hci::controller::ExternalController;
use bt_hci::uuid::descriptors::CLIENT_CHARACTERISTIC_CONFIGURATION;
use core::cell::RefCell;
use embassy_futures::select::select;
use embassy_futures::join::{join3,join};
use embassy_futures::select::Either::{First, Second};
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::Sender;
use esp_backtrace as _;
use trouble_host::scan::{LeAdvReportsIter, Scanner};
use trouble_host::connection::{PhySet, ScanConfig};
use trouble_host::{Host, HostResources};
use trouble_host::prelude::*;
use trouble_host::Address;
use esp_wifi::ble::controller::BleConnector;
use super::spark_message;
use advertisement::AdvertisementData;

// Max number of connections
const CONNECTIONS_MAX: usize = 6;
const L2CAP_CHANNELS_MAX: usize = 6;

// We need to send this characteristic UUID 0x1 and 0x0 to get notifications??
pub const MYSTERY_VALUES      : [u8; 2] = [0x1, 0x2];

pub const SPARK_SERVICE_UUID  : u16 = 0xFFC0;
pub const WRITE_CHARACTERISTIC: u16 = 0xFFC1;
pub const NOTIF_CHARACTERISTIC: u16 = 0xFFC2;

#[embassy_executor::task]
pub async fn run(
    timer: EspTimer<'static>,
    rng: RNG<'static>,
    clk: RADIO_CLK<'static>,
    bt: BT<'static>,
    channel: Sender<'static, CriticalSectionRawMutex, arrayvec::ArrayString<40>, 40>,
) {
    let init = esp_wifi::init(
        timer,
        esp_hal::rng::Rng::new(rng),
        clk,
    )
    .unwrap();

    // Using a fixed "random" address can be useful for testing. In real scenarios, one would
    // use e.g. the MAC 6 byte array as the address (how to get that varies by the platform).
    let address: Address = Address::random([0xB4, 0xE6, 0x2D, 0xB2, 0x1B, 0x36]);
    let connector = BleConnector::new(&init, bt);
    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> = HostResources::new();
    let controller: ExternalController<_, 20> = ExternalController::new(connector);
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);

    // let Host {
    //     central, mut runner, ..
    // } = stack.build();


    // // defmt::info!("Our address = {:02X?}", address);

    // let handler = scanner::ScanHandler::new();

    // let mut scanner = Scanner::new(central);
    // let addr = select(runner.run_with_handler(&handler), async {
    //     let mut config = ScanConfig::default();
    //     config.active = true;
    //     config.phys = PhySet::M1;
    //     config.interval = Duration::from_secs(1);
    //     config.window = Duration::from_secs(1);
    //     let mut _session = scanner.scan(&config).await.unwrap();

    //     while !handler.found_device() {
    //         Timer::after(Duration::from_secs(1)).await;
    //     }

    //     handler.get_device().unwrap()
    // })
    // .await;


    // let (addr_kind, addr) = match addr {
    //     First(a) => todo!(),
    //     Second(a) => Some(a).unwrap(),
    // };
    // defmt::info!("addr_kind: {:?} addr: {:?}", addr_kind, addr);


    // let params = ConnectParams {
    //     min_connection_interface: 120,
    //     max_connection_interval: 120,
    //     max_latency: 0,
    //     timeout: 60
    // };

    let addr_kind = AddrKind::RANDOM;
    let addr = BdAddr::new([0xBA, 0x79, 0x33, 0xED, 0xEB, 0xF7]);

    defmt::info!("Got addr, {}, {}", addr_kind, addr);

    let config = ConnectConfig {
        connect_params: ConnectParams {
            min_connection_interval: Duration::from_millis(40),
            max_connection_interval: Duration::from_millis(40),
            max_latency: 5,
            event_length: Duration::from_millis(0),
            supervision_timeout: Duration::from_secs(10),
        },
        scan_config: ScanConfig {
            // active: true,
            filter_accept_list: &[(addr_kind, &addr)],
            // phys: PhySet:M1,
            // interval: Duration::from_secs(2),
            // window: Duration::from_secs(2),
            // timeout: Duration::from_secs(2),
            ..Default::default()
        },
    };

    let Host {
        mut central, mut runner, ..
    } = stack.build();

    defmt::info!("Scanning for peripheral...");
    let _ = join(runner.run(), async {
        defmt::info!("Connecting");
        let mut s = arrayvec::ArrayString::<40>::from("Connecting...").unwrap();
        channel.send(s).await;


        let conn = central.connect(&config).await.unwrap();

        let client = GattClient::<_, DefaultPacketPool, 10>::new(&stack, &conn)
            .await
            .unwrap();

        let mut s = arrayvec::ArrayString::<40>::from("Connected!").unwrap();
        channel.send(s).await;

        let _ = join(client.task(), async {
            let services = client.services_by_uuid(&Uuid::new_short(SPARK_SERVICE_UUID)).await.unwrap();
            let service = services.first().unwrap().clone();

            let read_characteristic: Characteristic<u8> = client
                .characteristic_by_uuid(&service, &Uuid::from(NOTIF_CHARACTERISTIC))
                .await
                .unwrap();


            let res = client.write_characteristic(&read_characteristic, &MYSTERY_VALUES).await;

            let write_characteristic: Characteristic<u8> = client
                .characteristic_by_uuid(&service, &Uuid::new_short(WRITE_CHARACTERISTIC))
                .await
                .unwrap();


            let mut listener = client.subscribe(&read_characteristic, false).await.unwrap();

            let _ = join3(
                async {
                    loop {
                        let data = listener.next().await;
                        defmt::info!("Got notification:\n{:X} (val: {:X})", data.as_ref(), data.as_ref()[0]);
                        let decoder = spark_message::SparkMsgDecoder;
                        let msg = decoder.decode(&data.as_ref());
                        match msg {
                            Some(spark_message::SparkToAppMsg::AmpName { sequence, name }) => {
                                defmt::info!("Connected to {}, seq: {}", name.as_str(), sequence);
                                let s = arrayvec::ArrayString::<40>::from(&name).unwrap();
                                channel.send(s).await;
                            },
                            _ => {}
                        }
                    }
                },
                async {
                    let mut encoder = spark_message::SparkMsgEncoder::new();
                    let msg = spark_message::AppToSparkMsg::GetAmpName{};
                    let mut blocks = encoder.encode(msg);

                    for block in &mut blocks {
                        defmt::info!("write characteristic\n{:X}", block[..]);
                        client.write_characteristic(&write_characteristic, &block).await.unwrap();
                    }
                },
                async {
                    Timer::after(Duration::from_secs(4)).await;
                    loop {
                        let mut encoder = spark_message::SparkMsgEncoder::new();
                        let msg = spark_message::AppToSparkMsg::SetHardwarePreset(1);
                        let mut blocks = encoder.encode(msg);

                        let mut s = arrayvec::ArrayString::<40>::from("Set Hardware\npreset: 1").unwrap();
                        channel.send(s).await;
                        for block in &mut blocks {
                            defmt::info!("write characteristic\n{:X}", block[..]);
                            client.write_characteristic(&write_characteristic, &block).await.unwrap();
                        }
                        Timer::after(Duration::from_secs(2)).await;

                        let msg = spark_message::AppToSparkMsg::SetHardwarePreset(2);
                        let mut blocks = encoder.encode(msg);

                        let mut s = arrayvec::ArrayString::<40>::from("Set Hardware\npreset: 2").unwrap();
                        channel.send(s).await;
                        for block in &mut blocks {
                            defmt::info!("write characteristic\n{:X}", block[..]);
                            client.write_characteristic(&write_characteristic, &block).await.unwrap();
                        }
                        Timer::after(Duration::from_secs(2)).await;
                        let msg = spark_message::AppToSparkMsg::SetHardwarePreset(3);
                        let mut blocks = encoder.encode(msg);

                        let mut s = arrayvec::ArrayString::<40>::from("Set Hardware\npreset: 3").unwrap();
                        channel.send(s).await;
                        for block in &mut blocks {
                            defmt::info!("write characteristic\n{:X}", block[..]);
                            client.write_characteristic(&write_characteristic, &block).await.unwrap();
                        }
                        Timer::after(Duration::from_secs(2)).await;
                        let msg = spark_message::AppToSparkMsg::SetHardwarePreset(4);
                        let mut blocks = encoder.encode(msg);

                        let mut s = arrayvec::ArrayString::<40>::from("Set Hardware\npreset: 4").unwrap();
                        channel.send(s).await;
                        for block in &mut blocks {
                            defmt::info!("write characteristic\n{:X}", block[..]);
                            client.write_characteristic(&write_characteristic, &block).await.unwrap();
                        }
                        Timer::after(Duration::from_secs(2)).await;
                    }
                },
            )
            .await;
        })
        .await;
    })
    .await;

    loop {
        defmt::info!("BLE loop");
        Timer::after(Duration::from_secs(10)).await;
    }
}
