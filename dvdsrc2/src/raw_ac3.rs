use std::{
    ffi::{c_void, CStr, CString},
    io::{Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
};

use const_str::cstr;
use dvdsrccommon::audio_demuxing::{raw_audio_frames_init, AudioFramesInfo};
use vapoursynth4_rs::{
    core::CoreRef,
    frame::{Frame, FrameContext, VideoFrame},
    key,
    map::{MapMut, MapRef},
    node::{ActivationReason, Dependencies, Filter, FilterMode},
    VideoInfo,
};

use crate::{add_audio_props, open_dvd_vobus};

struct FullFilterMutStuff {
    ac3info: AudioFramesInfo,
}

pub struct RawAc3Filter {
    ai: VideoInfo,
    mut_stuff: Arc<Mutex<FullFilterMutStuff>>,
}

impl Filter for RawAc3Filter {
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
        let reader = open_dvd_vobus.reader;
        let audio = input
            .get_int(key!("audio"), 0)
            .expect("Failed to get audio");

        let ac3info = raw_audio_frames_init(
            reader,
            open_dvd_vobus.indexed.clone(),
            open_dvd_vobus.vobus,
            audio as _,
            false,
        );

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
            width: ac3info.frame_length as _,
            height: 1,
            num_frames: ac3info.frame_cnt as _,
        };

        let mutdata = FullFilterMutStuff { ac3info };
        let filter = RawAc3Filter {
            ai: vi.clone(),
            mut_stuff: Arc::new(Mutex::new(mutdata)),
        };

        let deps = [];

        core.create_video_filter(
            output,
            cstr!("RawAc3"),
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
            let mut stuff = self.mut_stuff.lock().unwrap();

            let fl = stuff.ac3info.frame_length;
            let mut newframe = core.new_video_frame(&self.ai.format, fl as _, 1, None);

            add_audio_props(n as _, newframe.properties_mut().unwrap(), &stuff.ac3info);

            let buffer = std::slice::from_raw_parts_mut(newframe.plane_mut(0), fl as _);

            stuff
                .ac3info
                .reader
                .seek(SeekFrom::Start((n as u64) * fl as u64))
                .unwrap();
            stuff.ac3info.reader.read_exact(&mut buffer[0..]).unwrap();

            // dbg!(&wps);
            return Ok(Some(newframe));
        }
    }

    const NAME: &'static CStr = cstr!("RawAc3");
    const ARGS: &'static CStr =
        cstr!("path:data;vts:int;audio:int;ranges:int[]:opt;domain:int:opt");
    const RETURN_TYPE: &'static CStr = cstr!("clip:vnode;");
    const FILTER_MODE: FilterMode = FilterMode::Unordered;
}
