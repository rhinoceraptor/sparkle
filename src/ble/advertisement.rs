#[derive(Clone, Debug)]
pub enum ServiceUuid {
    Uuid16(u16),
    Uuid32(u32),
    Uuid128([u8; 16]),
}

impl From<u16> for ServiceUuid {
    fn from(value: u16) -> Self {
        ServiceUuid::Uuid16(value)
    }
}

impl From<u32> for ServiceUuid {
    fn from(value: u32) -> Self {
        ServiceUuid::Uuid32(value)
    }
}

impl From<[u8; 16]> for ServiceUuid {
    fn from(value: [u8; 16]) -> Self {
        ServiceUuid::Uuid128(value)
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum AdvertisementType {
    Flags = 0x01,
    IncompleteListUuid16 = 0x02,
    CompleteListUuid16 = 0x03,
    IncompleteListUuid32 = 0x04,
    CompleteListUuid32 = 0x05,
    IncompleteListUuid128 = 0x06,
    CompleteListUuid128 = 0x07,
    ShortenedLocalName = 0x08,
    CompleteLocalName = 0x09,
    TxPowerLevel = 0x0a,
    PeripheralConnIntervalRange = 0x12,
    ListSolicitationUuid16 = 0x14,
    ListSolicitationUuid128 = 0x15,
    ServiceDataUuid16 = 0x16,
    PublicTargetAddress = 0x17,
    RandomTargetAddress = 0x18,
    Appearance = 0x19,
    AdvertisingInterval = 0x1a,
    ServiceDataUuid32 = 0x20,
    ServiceDataUuid128 = 0x21,
    URI = 0x24,
    PbADV = 0x29,
    MeshMessage = 0x2a,
    MeshBeacon = 0x2b,
    BroadcastName = 0x30,
    ManufacturerSpecificData = 0xff,
}

#[derive(Clone, Debug)]
pub struct AdvertisementData {
    pub flags: Option<u8>,
    pub service_uuids_16: [u16; 8],
    pub service_uuids_32: [u32; 4],
    pub service_uuids_128: [[u8; 16]; 2],
    pub manufacturer_data: [u8; 16],
    pub other_data: [(u8, [u8; 16]); 4],
}

impl AdvertisementData {
    pub fn new() -> Self {
        Self {
            flags: None,
            service_uuids_16: [0; 8],
            service_uuids_32: [0; 4],
            service_uuids_128: [[0; 16]; 2],
            manufacturer_data: [0; 16],
            other_data: [(0, [0; 16]); 4],
        }
    }

    pub fn new_from_bytes(data: &[u8]) -> Self {
        let mut ad_data = AdvertisementData::new();
        let mut i = 0;
        let mut uuid16_index = 0;
        let mut uuid32_index = 0;
        let mut uuid128_index = 0;
        let mut manufacturer_index = 0;
        let mut other_index = 0;

        while i < data.len() {
            let length = data[i] as usize;
            if length == 0 || i + length >= data.len() {
                break;
            }

            let ad_type = data[i + 1];
            let payload = &data[i + 2..i + length + 1];

            match ad_type {
                0x01 => {
                    if payload.len() == 1 {
                        ad_data.flags = Some(payload[0]);
                    }
                }
                0x02 | 0x03 => { // 16-bit UUIDs
                    for j in (0..payload.len()).step_by(2) {
                        if j + 1 < payload.len() && uuid16_index < 8 {
                            let uuid = u16::from_le_bytes([payload[j], payload[j + 1]]);
                            ad_data.service_uuids_16[uuid16_index] = uuid;
                            uuid16_index += 1;
                        }
                    }
                }
                0x04 | 0x05 => { // 32-bit UUIDs
                    for j in (0..payload.len()).step_by(4) {
                        if j + 3 < payload.len() && uuid32_index < 4 {
                            let uuid = u32::from_le_bytes([payload[j], payload[j + 1], payload[j + 2], payload[j + 3]]);
                            ad_data.service_uuids_32[uuid32_index] = uuid;
                            uuid32_index += 1;
                        }
                    }
                }
                0x06 | 0x07 => { // 128-bit UUIDs
                    for j in (0..payload.len()).step_by(16) {
                        if j + 15 < payload.len() && uuid128_index < 2 {
                            ad_data.service_uuids_128[uuid128_index] = payload[j..j + 16].try_into().unwrap();
                            uuid128_index += 1;
                        }
                    }
                }
                0xFF => { // Manufacturer Specific Data
                    for &byte in payload.iter() {
                        if manufacturer_index < 16 {
                            ad_data.manufacturer_data[manufacturer_index] = byte;
                            manufacturer_index += 1;
                        }
                    }
                }
                _ => {
                    if other_index < 4 {
                        let mut buf = [0u8; 16];
                        let len = payload.len().min(16);
                        buf[..len].copy_from_slice(&payload[..len]);
                        ad_data.other_data[other_index] = (ad_type, buf);
                        other_index += 1;
                    }
                }
            }

            i += length + 1;
        }

        ad_data
    }

    pub fn is_advertising_service(&self, uuid: impl Into<ServiceUuid>) -> bool {
        match uuid.into() {
            ServiceUuid::Uuid16(uuid_16)   =>  self.service_uuids_16.contains(&uuid_16),
            ServiceUuid::Uuid32(uuid_32)   => self.service_uuids_32.contains(&uuid_32),
            ServiceUuid::Uuid128(uuid_128) => self.service_uuids_128.contains(&uuid_128),
        }
    }
}

