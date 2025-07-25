mod full_vts_ac3;
mod full_vts_lpcm;
mod full_vts_video;
mod ifo_file;
mod raw_ac3;
mod raw_vob;

use admap::*;
use dvdsrccommon::{
    audio_demuxing::AudioFramesInfo,
    do_index_dvd::{get_index_vts, DvdBlockReaderDomain, OpenDvdBlockReader},
    dvdio::{cache_seek_reader::CacheSeekReader, proper_dvd_reader::ProperDvdReader},
    index::IndexedVts,
    open_dvd,
    vobu_range::*,
};
use full_vts_ac3::*;
use full_vts_lpcm::*;
use full_vts_video::*;
use ifo_file::*;
use raw_ac3::*;
use raw_vob::*;
mod admap;

use const_str::cstr;
use vapoursynth4_rs::{
    declare_plugin, key,
    map::{MapMut, MapRef},
};
use vapoursynth4_sys::VSMapAppendMode;

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
    (RawVobFilter, None),
    (IfoFile, None),
    (AdmapFilter, None)
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

    let domain = match if let Ok(e) = input.get_int(key!("domain"), 0) {
        e
    } else {
        1
    } {
        0 => DvdBlockReaderDomain::Menu,
        1 => DvdBlockReaderDomain::Title,
        _ => panic!("invalid domain"),
    };
    let indexv = get_index_vts(dvdpath, vts as _, domain);

    let ranges = parse_range(input, &indexv);

    let vobus = VobuRanger::from(&ranges, &indexv);

    let dvd = open_dvd(dvdpath.try_into().unwrap()).unwrap();

    let reader = OpenDvdBlockReader::new(dvd, vts as _, domain).reader;

    OpenDvdVobus {
        indexed: indexv,
        vobus,
        reader,
    }
}

fn add_audio_props(n: i64, mut props: MapMut<'_>, ainfo: &AudioFramesInfo) {
    if n == 0 {
        props
            .set(
                key!("Stuff_Start_PTS"),
                vapoursynth4_rs::map::Value::Int(ainfo.pts_cutoff_start as _),
                VSMapAppendMode::Replace,
            )
            .unwrap();
        props
            .set(
                key!("Stuff_End_PTS"),
                vapoursynth4_rs::map::Value::Int(ainfo.pts_cut_end as _),
                VSMapAppendMode::Replace,
            )
            .unwrap();
    }
}
