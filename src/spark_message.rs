use zerocopy::{transmute, FromBytes, IntoBytes, Unaligned, Immutable};
use zerocopy::byteorder::{U16, U32, LittleEndian};

// The four‑byte magic value at the start of every block.
pub const BLOCK_MAGIC: U32<LittleEndian> = transmute!(0x01FE_0000);

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ToSpark   = 0x53FE,
    FromSpark = 0x41FF,
}

// 16‑byte block header (little‑endian fields):
//  - magic (u32)
//  - direction (u16)
//  - size (u8)
//  - reserved (9 bytes)
#[repr(C)]
#[derive(IntoBytes, FromBytes, Immutable, Unaligned, Debug)]
pub struct BlockHeader {
    pub magic:     U32<LittleEndian>,
    pub direction: U16<LittleEndian>,
    pub size:      u8,
    pub _reserved: [u8; 9],
}


// 6‑byte SysEx chunk header (all u8, so naturally Unaligned):
#[repr(C)]
#[derive(IntoBytes, FromBytes, Unaligned, Immutable, Debug)]
pub struct ChunkHeader {
    pub start:       u8,
    pub sysex_id:    u8,
    pub sequence:    u8,
    pub checksum:    u8,
    pub command:     u8,
    pub sub_command: u8,
}

fn decode_7bit(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < input.len() {
        let mask = input[i];
        i += 1;
        // up to 7 bytes follow
        for bit in 0..7 {
            if i >= input.len() { break; }
            let b = input[i];
            let full = if (mask >> bit) & 1 == 1 {
                b | 0x80
            } else {
                b
            };
            out.push(full);
            i += 1;
        }
    }
    out
}

fn decode_block(buf: &[u8]) -> Option<(u8, u8, u8, &[u8])> {
    // Must be at least header + chunk header + trailer
    if buf.len() < 16 + 6 + 1 { return None; }

    let (hdr, body)   = BlockHeader::read_from_prefix(buf).ok()?;
    if hdr.magic     != BLOCK_MAGIC { return None; }
    if hdr.direction != Direction::FromSpark as u16 { return None; }

    let (chunk_hdr, chunk_body) = ChunkHeader::read_from_prefix(&body).ok()?;
    if chunk_hdr.start != 0xF0 || chunk_hdr.sysex_id != 0x01 { return None; }

    Some((chunk_hdr.sequence, chunk_hdr.command, chunk_hdr.sub_command, chunk_body))
}

#[derive(Clone, Copy)]
pub enum AppToSparkMsg {
    GetAmpName,
}

impl AppToSparkMsg {
    fn opcode(&self) -> (u8, u8) {
        match self {
            AppToSparkMsg::GetAmpName => (0x02, 0x11),
        }
    }

    fn encode_payload(&self) -> Vec<u8> {
        use byteorder::{BigEndian, WriteBytesExt};
        let mut buf = Vec::new();

        match self {
            AppToSparkMsg::GetAmpName => {
                // no payload
            }
        }

        buf
    }
}

pub struct SparkMsgEncoder {
    next_sequence: u8,
}

impl SparkMsgEncoder {
    pub fn new() -> Self {
        SparkMsgEncoder { next_sequence: 0 }
    }

    fn encode_7bit(input: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < input.len() {
            let chunk_len = ((input.len() - i).min(7)) as usize;
            let mut mask = 0u8;
            out.push(0); // placeholder
            let mask_index = out.len() - 1;

            for bit in 0..chunk_len {
                let b = input[i + bit];
                if b & 0x80 != 0 {
                    mask |= 1 << bit;
                }
                out.push(b & 0x7F);
            }

            out[mask_index] = mask;
            i += chunk_len;
        }
        out
    }

    fn encode(&mut self, msg: AppToSparkMsg) -> Vec<u8> {
        let (command, sub_command) = msg.opcode();
        let raw = msg.encode_payload();
        let packed = Self::encode_7bit(&raw);

        let seq = self.next_sequence;
        self.next_sequence = seq.wrapping_add(1);

        let checksum = packed.iter().fold(0u8, |acc, &b| acc ^ b);

        let chunk_hdr = ChunkHeader {
            start:       0xF0,
            sysex_id:    0x01,
            sequence:    seq,
            checksum,
            command,
            sub_command,
        };

        let mut chunks = Vec::new();
        chunks.extend_from_slice(chunk_hdr.as_bytes());
        chunks.extend(packed);
        chunks.push(0xF7);

        let block_hdr = BlockHeader {
            magic:     BLOCK_MAGIC,
            direction: U16::new(Direction::ToSpark as u16),
            size:      (16 + chunks.len()) as u8,
            _reserved: [0; 9],
        };

        let mut block = Vec::with_capacity(16 + chunks.len());
        block.extend_from_slice(block_hdr.as_bytes());
        block.extend(&chunks);

        block
    }
}

// Parses incoming blocks from the amp.
pub struct SparkMessageDecoder;

#[derive(Clone)]
pub enum SparkToAppMsg {
    AmpName { sequence: u8, name: String },
}

impl SparkMessageDecoder {
    pub fn decode(&self, block: &[u8]) -> Option<SparkToAppMsg> {
        let (sequence, command, subcommand, payload) = decode_block(block)?;

        let raw = decode_7bit(payload);

        match (command, subcommand) {
            // GetAmpName
            (0x03, 0x11) => {
                if raw.len() < 1 { return None; }
                let name_len = raw[0] as usize;
                if raw.len() < 1 + name_len { return None; }
                let name_bytes = &raw[1..1 + name_len];
                let name = String::from_utf8(name_bytes.to_vec()).ok()?;
                Some(SparkToAppMsg::AmpName {
                    sequence,
                    name,
                })
            }
            _ => None
        }
    }
}

