use std::{
    ffi::{c_void, CStr, CString},
    io::{Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
};

use const_str::cstr;
use dvdsrccommon::{
    audio_demuxing::{raw_audio_frames_init, AudioFramesInfo},
    bindings::a52,
};
use vapoursynth4_rs::{
    core::CoreRef,
    ffi::{VSAudioChannels, VS_AUDIO_FRAME_SAMPLES},
    frame::{AudioFrame, Frame, FrameContext},
    key,
    map::MapRef,
    node::{ActivationReason, Dependencies, Filter, FilterMode},
    AudioInfo,
};

use crate::{add_audio_props, open_dvd_vobus};

struct FullFilterMutStuff {
    ac3info: AudioFramesInfo,

    a52_decoder: *mut a52::a52_state_t,
    latest_n: i32,

    buffer: Vec<u8>,
}

pub struct FullVtsFilterAc3 {
    ai: AudioInfo,
    //vts: IndexedVts,
    mut_stuff: Arc<Mutex<FullFilterMutStuff>>,
}

impl Filter for FullVtsFilterAc3 {
    type Error = CString;
    type FrameType = AudioFrame;
    type FilterData = ();

    fn create(
        input: MapRef<'_>,
        output: MapRef<'_>,
        _data: Option<Box<Self::FilterData>>,
        mut core: CoreRef,
    ) -> Result<(), Self::Error> {
        unsafe {
            let open_dvd_vobus = open_dvd_vobus(input)?;
            let reader = open_dvd_vobus.reader;
            let audio = input
                .get_int(key!(c"audio"), 0)
                .expect("Failed to get audio");

            let ac3info = raw_audio_frames_init(
                reader,
                open_dvd_vobus.indexed,
                open_dvd_vobus.vobus,
                audio as _,
                false,
            );

            let flags = ac3info.ac3_fingerprint.0 as u32;
            let (num_channels, channel_layout) = if flags == a52::A52_STEREO {
                (2, 3)
            } else if flags == a52::A52_3F2R + a52::A52_LFE {
                (
                    6,
                    (1 << VSAudioChannels::FrontLeft as u32)
                        + (1 << VSAudioChannels::FrontRight as u32)
                        + (1 << VSAudioChannels::FrontCenter as u32)
                        + (1 << VSAudioChannels::LowFrequency as u32)
                        + (1 << VSAudioChannels::SideLeft as u32)
                        + (1 << VSAudioChannels::SideRight as u32),
                )
            } else if flags == a52::A52_3F2R {
                (
                    5,
                    (1 << VSAudioChannels::FrontLeft as u32)
                        + (1 << VSAudioChannels::FrontRight as u32)
                        + (1 << VSAudioChannels::FrontCenter as u32)
                        + (1 << VSAudioChannels::SideLeft as u32)
                        + (1 << VSAudioChannels::SideRight as u32),
                )
            } else {
                unreachable!();
            };
            {
                let num_samples = (ac3info.frame_cnt * 256 * 6) as i64;
                let num_frames = (num_samples + VS_AUDIO_FRAME_SAMPLES as i64 - 1)
                    / VS_AUDIO_FRAME_SAMPLES as i64;

                //assert_eq!(ac3info.frame_cnt as i64, num_frames * 2);

                let ai = AudioInfo {
                    format: vapoursynth4_rs::ffi::VSAudioFormat {
                        sample_type: vapoursynth4_rs::ffi::VSSampleType::Float,
                        bits_per_sample: 32,
                        bytes_per_sample: 4,
                        num_channels,
                        channel_layout,
                    },
                    sample_rate: ac3info.ac3_fingerprint.1,
                    num_samples,
                    num_frames: num_frames as i32,
                };

                let mutdata = FullFilterMutStuff {
                    a52_decoder: a52::a52_init(0),
                    buffer: vec![0u8; ac3info.frame_length as usize],
                    ac3info,
                    latest_n: i32::MAX / 2,
                };
                let filter = FullVtsFilterAc3 {
                    ai: ai.clone(),
                    // vts: indexv,
                    mut_stuff: Arc::new(Mutex::new(mutdata)),
                };

                let deps = [];

                let mut output = output;
                let output = &mut output;

                core.create_audio_filter(
                    output,
                    cstr!("FullVtsAc3"),
                    &ai,
                    filter,
                    Dependencies::new(&deps).unwrap(),
                );

                Ok(())
            }
        }
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
            let mut buffer = stuff.buffer.clone();
            let a52_decoder = stuff.a52_decoder;
            let latest_n = stuff.latest_n;

            let ac3info: &mut AudioFramesInfo = &mut stuff.ac3info;
            let sample_start = n * VS_AUDIO_FRAME_SAMPLES;
            let inner_frame = sample_start as u64 / (256 * 6);
            assert_eq!(VS_AUDIO_FRAME_SAMPLES / (6 * 256), 2);
            assert_eq!(VS_AUDIO_FRAME_SAMPLES % (6 * 256), 0);

            let is_lastt = (n + 1) == self.ai.num_frames;

            let mut level = 1.0;

            let mut newframe = core.new_audio_frame(
                &self.ai.format,
                if is_lastt && (self.ai.num_samples as i64 % VS_AUDIO_FRAME_SAMPLES as i64) != 0 {
                    VS_AUDIO_FRAME_SAMPLES / 2
                } else {
                    VS_AUDIO_FRAME_SAMPLES
                },
                None,
            );
            add_audio_props(n as _, newframe.properties_mut().unwrap(), ac3info);

            let channel_cnt = self.ai.format.num_channels;

            if latest_n + 1 != n && !(n == 0) {
                ac3info
                    .reader
                    .seek(SeekFrom::Start(
                        (inner_frame as u64 - 1) * ac3info.frame_length as u64,
                    ))
                    .unwrap();
                ac3info
                    .reader
                    .read_exact(&mut buffer[0..ac3info.frame_length as usize])
                    .unwrap();
                buffer_block(a52_decoder, buffer.as_mut_ptr(), ac3info, &mut level);
                for _ in 0..6 {
                    a52::a52_block(a52_decoder);
                }
            }

            ac3info
                .reader
                .seek(SeekFrom::Start(
                    inner_frame as u64 * ac3info.frame_length as u64,
                ))
                .unwrap();
            ac3info
                .reader
                .read_exact(&mut buffer[0..ac3info.frame_length as usize])
                .unwrap();

            let samples = a52::a52_samples(a52_decoder);

            buffer_block(a52_decoder, buffer.as_mut_ptr(), ac3info, &mut level);

            let wps: Vec<*mut f32> = (0..channel_cnt)
                .map(|e| newframe.channel_mut(e) as *mut f32)
                .collect();

            for blk in 0..6 {
                a52::a52_block(a52_decoder);
                for ch in 0..channel_cnt {
                    for a1 in 0..256 {
                        *(wps[ch as usize].offset(blk * 256 + a1)) =
                            *samples.offset(a1 as isize + 256 * ch as isize);
                    }
                }
            }

            if !is_lastt {
                ac3info
                    .reader
                    .seek(SeekFrom::Start(
                        (inner_frame as u64 + 1) * ac3info.frame_length as u64,
                    ))
                    .unwrap();
                ac3info
                    .reader
                    .read_exact(&mut buffer[0..ac3info.frame_length as usize])
                    .unwrap();
                buffer_block(a52_decoder, buffer.as_mut_ptr(), ac3info, &mut level);
                for blk in 0..6 {
                    let _ = a52::a52_block(a52_decoder);

                    for ch in 0..channel_cnt {
                        for a1 in 0..256 {
                            *(wps[ch as usize].offset(blk * 256 + a1 + 256 * 6)) =
                                *samples.offset(a1 as isize + 256 * ch as isize);
                        }
                    }
                }
            }
            return Ok(Some(newframe));
        }
    }

    const NAME: &'static CStr = cstr!("FullVtsAc3");
    const ARGS: &'static CStr =
        cstr!("path:data;vts:int;audio:int;ranges:int[]:opt;domain:int:opt");
    const RETURN_TYPE: &'static CStr = cstr!("clip:anode;");
    const FILTER_MODE: FilterMode = FilterMode::Unordered;
}

fn buffer_block(
    a52_decoder: *mut a52::a52_state_t,
    buffer_ptr: *mut u8,
    ac3info: &mut AudioFramesInfo,
    level: &mut f32,
) {
    unsafe {
        let (mut flags, mut sample_rate, mut bit_rate) = (0, 0, 0);

        let syncinforet =
            a52::a52_syncinfo(buffer_ptr, &mut flags, &mut sample_rate, &mut bit_rate);
        assert_eq!(syncinforet, ac3info.frame_length as i32);
        assert_eq!(flags, ac3info.ac3_fingerprint.0);
        assert_eq!(sample_rate, ac3info.ac3_fingerprint.1);
        assert_eq!(bit_rate, ac3info.ac3_fingerprint.2);

        a52::a52_frame(
            a52_decoder,
            buffer_ptr,
            &mut ac3info.ac3_fingerprint.0,
            level,
            0.0,
        );
    }
}
