use serde_derive::{Deserialize, Serialize};

use crate::index::Gop;

#[derive(Default, Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct KnowUnit {
    pub offset: u32,
    pub pts: u32,
    pub frame_cnt: u8,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct AudioStreamPackets {
    pub know_units: Vec<KnowUnit>,
    pub total_bytes: u32,
    pub starting_frames: u32,
    pub abs_packet_cnt: u32,
}

//frame times
//ac3 2880 ticks of the 90KHz 32ms
//dts 10.67ms in length, or 960 ticks of the 90KHz clock
//LPCM frames are 150 ticks of the 90KHz clock long (1.67ms),

#[derive(Default, Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Stream {
    pub id: u8,
    pub first_ptm: u32,

    pub packets: AudioStreamPackets,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct VobCell {
    pub vob: u16,
    pub cell: u8,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Vobu {
    pub sector_start: u32,
    pub gops: Vec<Gop>,
    pub vobcell: VobCell,
    pub first_ptm: u32,
    pub last_ptm: u32,
    pub streams: Vec<Stream>,
    pub mpeg2_video_size: u32,
    pub mpeg2_has_seq_end: bool,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, bincode::Decode, bincode::Encode)]
pub struct IndexedVts {
    pub vobus: Vec<Vobu>,
    pub total_size_blocks: u32,
}
