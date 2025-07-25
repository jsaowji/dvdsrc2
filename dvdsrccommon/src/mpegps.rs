use std::io::{Cursor, Read};

use byteorder::ReadBytesExt;

pub const PTS_FLAG: u8 = 0b0010;
pub const PTSDTS_PTS_FLAG: u8 = 0b0011;
pub const PTSDTS_DTS_FLAG: u8 = 0b0001;

pub fn parse_pts(flag: u8, pts_bytesx: &[u8]) -> u32 {
    assert_eq!(pts_bytesx.len(), 5);
    let pts_bytes: Vec<u32> = pts_bytesx.iter().map(|e| *e as u32).collect();

    // 0010XXX1 XXXXXXXX XXXXXXX1 XXXXXXXX XXXXXXX1

    assert!(((pts_bytes[0] & 0b11110000) >> 4) == flag as u32);

    assert!((pts_bytes[0] & 0b00000001) == 1);
    assert!((pts_bytes[2] & 0b00000001) == 1);
    assert!((pts_bytes[4] & 0b00000001) == 1);

    let p0 = (pts_bytes[0] & 0b1110) >> 1;
    let p1 = (pts_bytes[1] << 8) + (pts_bytes[2] & 0b11111110) >> 1;
    let p2 = (pts_bytes[3] << 8) + (pts_bytes[4] & 0b11111110) >> 1;
    let pts = p2 + (p1 << 15) + (p0 << 30);

    return pts;
}

pub fn start_code(mut a: impl Read) -> Result<u8, std::io::Error> {
    let mut buf = [0u8; 4];
    a.read_exact(&mut buf)?;
    assert_eq!(buf[0..3], [0, 0, 1]);
    Ok(buf[3])
}

pub struct Pes {
    pub pts: Option<u32>,
    pub dts: Option<u32>,
    pub inner: Cursor<Vec<u8>>,

    pub has_pes_crc: bool,
}
pub fn pes(mut rdd: impl Read, sz: usize) -> Result<Pes, std::io::Error> {
    let a = rdd.read_u8()?;
    _ = a;
    //assert_eq!(a & 4,0);
    //assert_eq!( (a & 0b11000000) >> 6,2);

    let b = rdd.read_u8()?;
    //assert!(b & 0b00100000 == 0);
    //assert!(b & 0b00010000 == 0);
    let c = rdd.read_u8()?;

    let mut buf = [0; 64];
    rdd.read_exact(&mut buf[0..c as usize])?;
    let buf = &buf[0..c as usize];

    let pts_dts_ind = (b & 0b11000000) >> 6;
    let has_pes_crc = (b & 0b00000010) != 0;

    let mut pts = None;
    let mut dts = None;

    if pts_dts_ind == 0b00 {
        //nothing
    } else if pts_dts_ind == 0b10 {
        //pts only
        let pts_bytesx = &buf[..5];
        pts = Some(parse_pts(PTS_FLAG, pts_bytesx));
    } else if pts_dts_ind == 0b11 {
        //pts dts
        let pts_bytesx = &buf[0..5];
        let dts_bytesx = &buf[5..10];
        pts = Some(parse_pts(PTSDTS_PTS_FLAG, pts_bytesx));
        dts = Some(parse_pts(PTSDTS_DTS_FLAG, dts_bytesx));
    } else {
        unreachable!();
    }

    Ok(Pes {
        dts,
        pts,
        inner: rdd.read_to_cursor(sz - 3 - c as usize)?,
        has_pes_crc: has_pes_crc,
    })
}

pub trait ReadToVec {
    fn read_to_cursor(&mut self, b: usize) -> Result<Cursor<Vec<u8>>, std::io::Error>;
}
impl<T: Read> ReadToVec for T {
    fn read_to_cursor(&mut self, b: usize) -> Result<Cursor<Vec<u8>>, std::io::Error> {
        let mut v = vec![0u8; b];
        self.read_exact(&mut v[0..b])?;
        Ok(Cursor::new(v))
    }
}
