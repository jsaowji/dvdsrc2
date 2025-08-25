mod full_vts_ac3;
mod full_vts_lpcm;
mod full_vts_video;
mod ifo_file;
mod raw_ac3;
mod raw_vob;

use std::ffi::CString;

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

use vapoursynth4_rs::{declare_plugin, key, map::MapRef};
use vapoursynth4_sys::VSMapAppendMode;

declare_plugin!(
    c"com.jsaowji.dvdsrc2c",
    c"dvdsrc2",
    c"Dvdsrc 2nd tour",
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

pub fn parse_range<'a>(input: MapRef<'a>, indexv: &IndexedVts) -> Result<Vec<VobuRange>, ()> {
    Ok(if let Some(e) = input.num_elements(key!(c"ranges")) {
        let mut ranges: Vec<VobuRange> = Vec::new();
        assert_eq!(e % 2, 0);
        for i in 0..e / 2 {
            let from = input.get_int(key!(c"ranges"), i * 2).map_err(|_| ())?;
            let to = input.get_int(key!(c"ranges"), i * 2 + 1).map_err(|_| ())?;
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
    })
}

pub struct OpenDvdVobus {
    //dvd: *mut dvd_reader_t,
    indexed: IndexedVts,
    vobus: Vec<EVobu>,
    reader: CacheSeekReader<ProperDvdReader>,
}

fn open_dvd_vobus(input: MapRef<'_>) -> Result<OpenDvdVobus, CString> {
    let dvdpath = input
        .get_utf8(key!(c"path"), 0)
        .map_err(|_| c"Failed to get dvdpath")?;
    let vts = input
        .get_int(key!(c"vts"), 0)
        .map_err(|_| c"Failed to get vts")?;

    let domain = match if let Ok(e) = input.get_int(key!(c"domain"), 0) {
        e
    } else {
        1
    } {
        0 => DvdBlockReaderDomain::Menu,
        1 => DvdBlockReaderDomain::Title,
        _ => return Err(c"invalid domain".into()),
    };
    let indexv = get_index_vts(dvdpath, vts as _, domain);

    let ranges = parse_range(input, &indexv).map_err(|_| c"Failed to parse ranges")?;

    let vobus = VobuRanger::from(&ranges, &indexv);

    let dvd = open_dvd(dvdpath.try_into().unwrap()).unwrap();

    let reader = OpenDvdBlockReader::new(dvd, vts as _, domain).reader;

    Ok(OpenDvdVobus {
        indexed: indexv,
        vobus,
        reader,
    })
}

fn add_audio_props(n: i64, mut props: MapRef<'_>, ainfo: &AudioFramesInfo) {
    if n == 0 {
        props
            .set(
                key!(c"Stuff_Start_PTS"),
                vapoursynth4_rs::map::Value::Int(ainfo.pts_cutoff_start as _),
                VSMapAppendMode::Replace,
            )
            .unwrap();
        props
            .set(
                key!(c"Stuff_End_PTS"),
                vapoursynth4_rs::map::Value::Int(ainfo.pts_cut_end as _),
                VSMapAppendMode::Replace,
            )
            .unwrap();
    }
}
