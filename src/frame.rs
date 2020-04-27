use std::io;

#[derive(Debug)]
pub struct Frame {
    frame_type: FrameType,
    is_final_frame: bool,
    payload: Vec<u8>,
    mask: Option<[u8; 4]>,
}



impl Frame {
    pub fn from_payload(frame_type: FrameType, payload: Vec<u8>) -> Frame {
        Frame { frame_type, is_final_frame: true, payload, mask: None }
    }

    pub fn encode(&self) -> Vec<u8> {
        let len = self.payload.len();
        let len_descriptor = PayloadLength::from_len(len);

        let mut ret = vec![
            if self.is_final_frame { 0b_1000_0000 } else { 0b_0000_000 }
                | self.frame_type as u8,
            if self.mask.is_some() { 0b_1000_0000 } else { 0b_0000_000 }
                | len_descriptor.to_byte()
        ];

        match len_descriptor {
            PayloadLength::Small(_) => {},
            PayloadLength::Extended =>
                ret.extend_from_slice(&(len as u64).to_be_bytes()[6..]),
            PayloadLength::ExtraExtended =>
                ret.extend_from_slice(&(len as u64).to_be_bytes()),
        }

        match self.mask {
            Some(mask) => {
                ret.extend_from_slice(&mask);
                let masked_payload = xor_mask(&self.payload, &mask);
                ret.extend_from_slice(&masked_payload);
            },
            None => ret.extend_from_slice(&self.payload),
        }

        ret
    }

    pub fn decode(buf: &[u8]) -> Result<Frame, FrameError> {
        let get = |i| buf.get(i).copied().ok_or(FrameError::FrameTooSmall);
        let range = |r| buf.get(r).ok_or(FrameError::FrameTooSmall);

        let is_final_frame = (get(0)? >> 7) == 1;

        let frame_type = match FrameType::from_number(get(0)? & 0b1111) {
            Some(frame_type) => frame_type,
            None => return Err(FrameError::InvalidOpcode),
        };

        let is_masked = (get(1)? >> 7) == 1;

        let (payload_length, masking_key_offset) =
            match get(1)? & 0b1111111 {
                n @ 0..=125=> {
                    // regular old payload length
                    (n as usize, 2)
                },
                126 => {
                    // extended payload length
                    let n = (get(2)? as u16) << 8 | (get(3)? as u16);
                    (n as usize, 4)
                },
                127 => {
                    // extended payload length continued
                    let mut n: u64 = 0;
                    let mut o = 56;
                    for byte in 0..8 {
                        n |= (get(2+byte)? as u64) << o;
                        o -= 8;
                    }
                    (n as usize, 10)
                },
                _ => unreachable!("bit mask doesnt allow for greater"),
            };

        let raw_payload = &range(masking_key_offset+4..masking_key_offset+4+payload_length)?;

        let mut mask = None;

        let decoded_data =
            if is_masked {
                let key = &range(masking_key_offset..masking_key_offset+4)?;
                mask = Some([key[0], key[1], key[2], key[3]]); // know key has length 4
                xor_mask(raw_payload, key)
            } else {
                raw_payload.to_vec()
            };


        Ok(Frame { frame_type, is_final_frame, payload: decoded_data, mask })
    }

    pub fn is_final_frame(&self) -> bool {
        self.is_final_frame
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FrameType {
    Continue = 0x0,
    Text = 0x1,
    Binary = 0x2,
    RequestClose = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl FrameType {
    pub fn from_number(opcode: u8) -> Option<FrameType> {
        Some(match opcode {
            0x0 => FrameType::Continue,
            0x1 => FrameType::Text,
            0x2 => FrameType::Binary,
            0x8 => FrameType::RequestClose,
            0x9 => FrameType::Ping,
            0xA => FrameType::Pong,
            _ => return None,
        })
    }
}

fn xor_mask(encoded: &[u8], mask: &[u8]) -> Vec<u8> {
    encoded.iter().zip(mask.iter().cycle())
        .map(|(&byte, &mask)| byte ^ mask)
        .collect()
}

#[derive(Debug)]
pub enum FrameError {
    IoError(io::Error),
    InvalidOpcode,
    FrameTooSmall,
}

impl FrameError {
    pub fn should_retry(&self) -> bool {
        match *self {
            FrameError::FrameTooSmall => true, // we can solve by pulling larger frames !
            FrameError::InvalidOpcode => false,
            FrameError::IoError(..) => false,
        }
    }
}

impl From<io::Error> for FrameError {
    fn from(error: io::Error) -> FrameError {
        FrameError::IoError(error)
    }
}

#[derive(Copy, Clone)]
enum PayloadLength {
    Small(u8),
    Extended,
    ExtraExtended,
}

impl PayloadLength {
    fn from_len(len: usize) -> PayloadLength {
        match len {
            0..=125 => PayloadLength::Small(len as u8),
            126 ..= 65535 => PayloadLength::Extended,
            _ => PayloadLength::ExtraExtended,
        }
    }

    fn to_byte(self) -> u8 {
        match self {
            PayloadLength::Small(len) => len,
            PayloadLength::Extended => 126,
            PayloadLength::ExtraExtended => 127,
        }
    }
}