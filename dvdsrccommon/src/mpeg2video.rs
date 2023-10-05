use std::io::Read;

use byteorder::ReadBytesExt;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Gop {
    // time_code: u32,
    pub closed: bool,
    pub broken: bool,
}

pub fn group_of_pictures_header(mut a: impl Read) -> Result<Gop, std::io::Error> {
    let mut b = [0u8; 4];
    a.read_exact(&mut b)?;

    Ok(Gop {
        //time_code: (b[0] as u32) << 8 + (b[1] as u32) << 16 + (b[2] as u32) << 24 + ((b[3] as u32) & 0b10000000) >> 7,
        closed: (b[3] & 0b01000000) != 0,
        broken: (b[3] & 0b00100000) != 0,
    })
}

#[derive(Debug)]
pub struct Picture {
    pub temporal_reference: u16,
    pub picture_type: u8,
}

pub fn picture_header(mut a: impl Read) -> Result<Picture, std::io::Error> {
    let mut b = [0u8; 2];
    a.read_exact(&mut b)?;

    let temporal_ref = ((b[0] as u16) << 2) + (((b[1] & 0b11000000) as u16) >> 6);

    Ok(Picture {
        temporal_reference: temporal_ref,
        picture_type: (b[1] >> 3) & 0b111,
    })
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct SequenceHeader {
    pub width: u16,
    pub height: u16,

    pub ar_num: u16,
    pub ar_den: u16,

    pub fr_num: u16,
    pub fr_den: u16,
}

pub fn sequence_header(mut a: impl Read) -> Result<SequenceHeader, std::io::Error> {
    let mut data = [0u8; 8 + 64 * 2];
    a.read_exact(&mut data[0..])?;

    let ar = (data[3] & 0b11110000) >> 4;
    let fr = (data[3] & 0b00001111) >> 0;

    let (ar_num, ar_den) = match ar {
        0b0000 => unreachable!(),
        0b0001 => (1, 1),
        0b0010 => (3, 4),
        0b0011 => (9, 16),
        0b0100 => (100, 212),
        _ => unreachable!(),
    };
    let (fr_num, fr_den) = match fr {
        0b0000 => unreachable!(),
        0b0001 => (24000, 1001),
        0b0010 => (24, 1),
        0b0011 => (25, 1),
        0b0100 => (30000, 1001),
        0b0101 => (30, 1),
        0b0110 => (50, 1),
        0b0111 => (60000, 1001),
        0b1000 => (60, 1),
        _ => unreachable!(),
    };

    let seq = SequenceHeader {
        width: ((data[0] as u16) << 4) + ((data[1] as u16 & 0b11110000) >> 4),
        height: (data[2] as u16) + ((data[1] as u16 & 0b00001111) << 8),
        ar_den,
        ar_num,
        fr_den,
        fr_num,
    };

    Ok(seq)
}

#[derive(Debug)]
pub enum Extension {
    Sequence { progressive_sequence: bool },
    Picture { tff: bool, rff: bool, prog: bool },
}

pub fn extension_header(mut a: impl Read) -> Result<Option<Extension>, std::io::Error> {
    let st = a.read_u8()?;
    match (st & 0b11110000) >> 4 {
        0x1 => {
            let data1 = a.read_u8()?;
            let progressive_sequence = (data1 & (1 << 3)) != 0;
            return Ok(Some(Extension::Sequence {
                progressive_sequence,
            }));
        }
        0x8 => {
            let mut buf = [0u8; 10];
            buf[0] = st;
            let _left = a.read(&mut buf[1..])?;

            let picture_structure = buf[2] & 0b11;
            let top_field_first = buf[3] & (1 << 7);
            let repeat_first_field = buf[3] & (1 << 1);
            let progressive_frame = buf[4] & (1 << 7);

            for a in [0, 1, 2] {
                assert_ne!(picture_structure, a);
            }

            return Ok(Some(Extension::Picture {
                tff: top_field_first != 0,
                rff: repeat_first_field != 0,
                prog: progressive_frame != 0,
            }));
        }
        _ => Ok(None),
    }
}
