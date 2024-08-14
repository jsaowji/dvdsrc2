use std::{
    ffi::{c_void, CStr, CString},
    io::{Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
};

use const_str::cstr;
use vapoursynth4_rs::{
    core::CoreRef,
    frame::{FrameContext, VideoFrame},
    map::{MapMut, MapRef},
    node::{ActivationReason, Dependencies, Filter, FilterMode},
    VideoInfo,
};

use crate::{open_dvd_vobus, OpenDvdVobus};

struct FullFilterMutStuff {
    dvdvvobus: OpenDvdVobus,
    seek_lookup_tbl: Vec<u64>,
}

pub struct RawVobFilter {
    vobs_vi: VideoInfo,
    mut_stuff: Arc<Mutex<FullFilterMutStuff>>,
}

impl Filter for RawVobFilter {
    type Error = CString;
    type FrameType = VideoFrame;
    type FilterData = ();

    fn create(
        input: MapRef<'_>,
        output: MapMut<'_>,
        _data: Option<Box<Self::FilterData>>,
        mut core: CoreRef,
    ) -> Result<(), Self::Error> {
        let open_dvd_vobus = open_dvd_vobus(input);

        let total_sz = open_dvd_vobus.indexed.total_size_blocks;
        let mut seek_lookup_tbl: Vec<u64> = Vec::with_capacity(total_sz as usize);

        for v in &open_dvd_vobus.vobus {
            let start = open_dvd_vobus.indexed.vobus[v.i].sector_start;
            let end = if v.i == open_dvd_vobus.indexed.vobus.len() - 1 {
                total_sz
            } else {
                open_dvd_vobus.indexed.vobus[v.i + 1].sector_start
            };
            let sz = end - start;
            for a in 0..sz {
                seek_lookup_tbl.push((start + a) as u64 * 2048);
            }
        }

        let vi = VideoInfo {
            format: vapoursynth4_rs::ffi::VSVideoFormat {
                color_family: vapoursynth4_rs::ffi::VSColorFamily::Gray,
                sample_type: vapoursynth4_rs::ffi::VSSampleType::Integer,
                bits_per_sample: 8,
                bytes_per_sample: 1,
                sub_sampling_w: 0,
                sub_sampling_h: 0,
                num_planes: 1,
            },
            fps_num: 0,
            fps_den: 0,
            width: 2048,
            height: 1,
            num_frames: seek_lookup_tbl.len() as _,
        };

        let mutdata = FullFilterMutStuff {
            dvdvvobus: open_dvd_vobus,
            seek_lookup_tbl,
        };

        let filter = RawVobFilter {
            vobs_vi: vi.clone(),
            mut_stuff: Arc::new(Mutex::new(mutdata)),
        };

        let deps = [];

        core.create_video_filter(
            output,
            cstr!("RawVob"),
            &vi,
            Box::new(filter),
            Dependencies::new(&deps).unwrap(),
        );

        Ok(())
    }

    fn get_frame(
        &self,
        n: i32,
        _activation_reason: ActivationReason,
        _frame_data: *mut *mut c_void,
        _ctx: FrameContext,
        core: CoreRef,
    ) -> Result<Option<VideoFrame>, Self::Error> {
        unsafe {
            let fl = 2048;
            let mut newframe = core.new_video_frame(&self.vobs_vi.format, fl as _, 1, None);
            assert_eq!(newframe.stride(0), fl);
            let buffer = std::slice::from_raw_parts_mut(newframe.plane_mut(0), fl as _);

            let mut stuff = self.mut_stuff.lock().unwrap();
            let ind = stuff.seek_lookup_tbl[n as usize];
            stuff.dvdvvobus.reader.seek(SeekFrom::Start(ind)).unwrap();
            stuff.dvdvvobus.reader.read_exact(&mut buffer[0..]).unwrap();

            return Ok(Some(newframe));
        }
    }

    const NAME: &'static CStr = cstr!("RawVob");
    const ARGS: &'static CStr = cstr!("path:data;vts:int;ranges:int[]:opt;");
    const RETURN_TYPE: &'static CStr = cstr!("clip:vnode;");
    const FILTER_MODE: FilterMode = FilterMode::Unordered;
}
