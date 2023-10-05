use std::{
    ffi::{c_void, CStr, CString},
    io::{Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
};

use const_str::cstr;
use dvdsrccommon::audio_demuxing::{raw_audio_frames_init, AudioFramesInfo};
use vapoursynth4_rs::{
    core::CoreRef,
    frame::{AudioFrame, FrameContext},
    key,
    map::{MapMut, MapRef},
    node::{ActivationReason, Dependencies, Filter, FilterMode},
    AudioInfo,
};
use vapoursynth4_sys::VS_AUDIO_FRAME_SAMPLES;

use crate::open_dvd_vobus;

struct FullFilterMutStuff {
    ac3info: AudioFramesInfo,
}

pub struct FullVtsFilterLpcm {
    ai: AudioInfo,
    mut_stuff: Arc<Mutex<FullFilterMutStuff>>,
}

impl Filter for FullVtsFilterLpcm {
    type Error = CString;
    type FrameType = AudioFrame;
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

        let lpcminfo = raw_audio_frames_init(
            reader,
            open_dvd_vobus.indexed.clone(),
            open_dvd_vobus.vobus,
            audio as _,
            true,
        );
        let num_samples = lpcminfo.samples_per_frame as i64 * lpcminfo.frame_cnt as i64;

        let num_frames =
            (num_samples + VS_AUDIO_FRAME_SAMPLES as i64 - 1) / VS_AUDIO_FRAME_SAMPLES as i64;

        assert_eq!(lpcminfo.lpcm_channels, 2);

        let ai = AudioInfo {
            format: vapoursynth4_rs::ffi::VSAudioFormat {
                sample_type: vapoursynth4_rs::ffi::VSSampleType::Integer,
                bits_per_sample: 16,
                bytes_per_sample: 2,
                num_channels: lpcminfo.lpcm_channels as _,
                channel_layout: 3,
            },
            sample_rate: lpcminfo.lpcm_sample_rate as _,
            num_samples,
            num_frames: num_frames as i32,
        };

        let mutdata = FullFilterMutStuff { ac3info: lpcminfo };
        let filter = FullVtsFilterLpcm {
            ai: ai.clone(),
            mut_stuff: Arc::new(Mutex::new(mutdata)),
        };

        let deps = [];

        core.create_audio_filter(
            output,
            cstr!("RawAc3"),
            &ai,
            filter,
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
    ) -> Result<Option<AudioFrame>, Self::Error> {
        unsafe {
            let mut stuff = self.mut_stuff.lock().unwrap();
            let sample_offset = n * VS_AUDIO_FRAME_SAMPLES;
            let num_samples_want = (VS_AUDIO_FRAME_SAMPLES as i64).min(
                (self.ai.num_samples as i64 - n as i64 * VS_AUDIO_FRAME_SAMPLES as i64) as i64,
            );

            let mut newframe = core.new_audio_frame(&self.ai.format, num_samples_want as _, None);

            let byte_per_src_sample_chblock =
                (stuff.ac3info.lpcm_quant * stuff.ac3info.lpcm_channels) / 8;

            let sk_pos = (sample_offset as u64) * byte_per_src_sample_chblock as u64;

            let bytes_total_read = num_samples_want as usize * byte_per_src_sample_chblock as usize;
            let mut buffer = vec![0u8; bytes_total_read as usize];
            stuff
                .ac3info
                .reader
                .seek(SeekFrom::Start(sk_pos as u64))
                .unwrap();
            let bfr = &mut buffer[0..bytes_total_read];
            stuff.ac3info.reader.read_exact(bfr).unwrap();

            if stuff.ac3info.lpcm_quant == 16 {
                for i in 0..bytes_total_read as usize / 2 {
                    let a = bfr[i * 2 + 0];
                    let b = bfr[i * 2 + 1];

                    bfr[i * 2 + 0] = b;
                    bfr[i * 2 + 1] = a;
                }
                let c0 = newframe.channel_mut(0) as *mut u16;
                let c1 = newframe.channel_mut(1) as *mut u16;
                let src = bfr.as_ptr() as *mut u16;
                for i in 0..num_samples_want {
                    *c0.offset(i as isize) = *src.offset(i as isize * 2 + 0);
                    *c1.offset(i as isize) = *src.offset(i as isize * 2 + 1);
                }
            } else {
                unreachable!();
            }

            return Ok(Some(newframe));
        }
    }

    const NAME: &'static CStr = cstr!("FullVtsLpcm");
    const ARGS: &'static CStr = cstr!("path:data;vts:int;audio:int;ranges:int[]:opt;");
    const RETURN_TYPE: &'static CStr = cstr!("clip:anode;");
    const FILTER_MODE: FilterMode = FilterMode::Unordered;
}
