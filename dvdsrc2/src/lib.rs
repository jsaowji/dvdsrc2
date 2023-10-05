mod full_vts_ac3;
mod full_vts_lpcm;
mod full_vts_video;
mod ifo_file;
mod raw_ac3;

use std::{ffi::CString, ptr::null_mut};

use dvdsrccommon::{
    bindings::dvdread::DVDOpen2,
    do_index_dvd::{get_index_vts, OpenDvdBlockReader},
    dvdio::{cache_seek_reader::CacheSeekReader, proper_dvd_reader::ProperDvdReader},
    index::IndexedVts,
    vobu_range::*,
    LOGGER_CB,
};
use full_vts_ac3::*;
use full_vts_lpcm::*;
use full_vts_video::*;
use ifo_file::*;
use raw_ac3::*;

use const_str::cstr;
use vapoursynth4_rs::{declare_plugin, key, map::MapRef};

declare_plugin!(
    "com.jsaowji.dvdsrc2c",
    "dvdsrc2",
    "Dvdsrc 2nd tour",
    (1, 0),
    vapoursynth4_rs::VAPOURSYNTH_API_VERSION,
    0,
    (FullVtsFilter, None),
    (FullVtsFilterAc3, None),
    (FullVtsFilterLpcm, None),
    (RawAc3Filter, None),
    (IfoFile, None)
);

pub fn parse_range<'a>(input: MapRef<'a>, indexv: &IndexedVts) -> Vec<VobuRange> {
    if let Some(e) = input.num_elements(key!("ranges")) {
        let mut ranges: Vec<VobuRange> = Vec::new();
        assert_eq!(e % 2, 0);
        for i in 0..e / 2 {
            let from = input
                .get_int(key!("ranges"), i * 2)
                .expect("Failed to ranges clip");
            let to = input
                .get_int(key!("ranges"), i * 2 + 1)
                .expect("Failed to ranges clip");
            for a in [from, to] {
                assert!(a >= 0);
                assert!(a <= indexv.vobus.len() as i64);
            }
            assert!(from <= to);
            ranges.push((from as u32, to as u32));
        }
        ranges
    } else {
        vec![(0 as u32, indexv.vobus.len() as u32 - 1)]
    }
}

pub struct OpenDvdVobus {
    //dvd: *mut dvd_reader_t,
    indexed: IndexedVts,
    vobus: Vec<EVobu>,
    reader: CacheSeekReader<ProperDvdReader>,
}

fn open_dvd_vobus(input: MapRef<'_>) -> OpenDvdVobus {
    let dvdpath = input
        .get_utf8(key!("path"), 0)
        .expect("Failed to get dvdpath");
    let vts = input.get_int(key!("vts"), 0).expect("Failed to get vts");

    let indexv = get_index_vts(dvdpath, vts as _);

    let ranges = parse_range(input, &indexv);

    let vobus = VobuRanger::from(&ranges, &indexv);

    let cas = CString::new(dvdpath).unwrap();
    let dvd = unsafe { DVDOpen2(null_mut(), &LOGGER_CB, cas.as_ptr()) };
    assert!(!dvd.is_null());

    let reader = OpenDvdBlockReader::new(dvd, vts as _).reader;

    OpenDvdVobus {
        indexed: indexv,
        vobus,
        reader,
    }
}
