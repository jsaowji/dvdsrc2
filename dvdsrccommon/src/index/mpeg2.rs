use serde_derive::{Deserialize, Serialize};

use crate::mpeg2video::SequenceHeader;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum FrameType {
    I,
    P,
    B,
}

impl FrameType {}

impl Default for FrameType {
    fn default() -> Self {
        FrameType::I
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Frame {
    pub frametype: FrameType,
    pub real_temporal_reference: u8,
    pub temporal_reference: u8,
    pub tff: bool,
    pub rff: bool,
    pub prog: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Gop {
    pub sequence_header: SequenceHeader,
    pub closed: bool,
    pub prog_sequence: bool,
    pub frames: Vec<Frame>, //in decode order
}

impl Gop {
    pub fn debug_print(&self) {
        let sq = &self.sequence_header;
        eprintln!(
            "{}x{}   fr{}/{} ar {}/{} closed {} PROG {}",
            sq.width,
            sq.height,
            sq.fr_num,
            sq.fr_den,
            sq.ar_den,
            sq.ar_num,
            self.closed,
            self.prog_sequence
        );
        eprintln!("Frames: {} (tff rff prog)", self.frames.len());
        for a in 0..self.frames.len() {
            let f = &self.frames[a];
            eprintln!(
                "{} {} {:?} {} {} {}",
                a, f.temporal_reference, f.frametype, f.tff as u8, f.rff as u8, f.prog as u8
            );
        }
    }
}
