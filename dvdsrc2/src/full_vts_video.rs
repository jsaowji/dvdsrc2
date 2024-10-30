use std::{
    collections::HashMap,
    error::Error,
    ffi::{c_void, CStr, CString},
    io::{Cursor, Read, Seek, SeekFrom, Write},
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use const_str::cstr;
use dvdsrccommon::{
    bindings::mpeg2::{mpeg2_accel, mpeg2_init, mpeg2_sequence_t, mpeg2_state_t, mpeg2dec_t},
    byteorder::{ReadBytesExt, BE},
    do_index_dvd::handle_scratch_read,
    dvdio::{cache_seek_reader::CacheSeekReader, proper_dvd_reader::ProperDvdReader},
    index::{FrameType, Vobu},
    mpegps::start_code,
};
use vapoursynth4_rs::{
    core::CoreRef,
    frame::{Frame, FrameContext, VideoFrame},
    key,
    map::{MapMut, MapRef, Value},
    node::{ActivationReason, Dependencies, Filter, FilterMode},
    VideoInfo,
};
use vapoursynth4_sys::{
    VSChromaLocation, VSColorPrimaries, VSFieldBased, VSMapAppendMode, VSMatrixCoefficients,
    VSPresetVideoFormat, VSTransferCharacteristics,
};

use crate::open_dvd_vobus;

pub const STATE_BUFFER: mpeg2_state_t = 0;
pub const STATE_SEQUENCE: mpeg2_state_t = 1;
pub const STATE_SEQUENCE_REPEATED: mpeg2_state_t = 2;
pub const STATE_GOP: mpeg2_state_t = 3;
pub const STATE_PICTURE: mpeg2_state_t = 4;
//pub const STATE_SLICE_1ST: mpeg2_state_t = 5;
//pub const STATE_PICTURE_2ND: mpeg2_state_t = 6;

pub const STATE_SLICE: mpeg2_state_t = 7;
pub const STATE_END: mpeg2_state_t = 8;
pub const STATE_INVALID: mpeg2_state_t = 9;
pub const STATE_INVALID_END: mpeg2_state_t = 10;
pub const STATE_SEQUENCE_MODIFIED: mpeg2_state_t = 11;
///TODO assert thjese

struct FullFilterMutStuff {
    reader: CacheSeekReader<ProperDvdReader>,
    frame_cache: HashMap<i32, VideoFrame>,

    mpeg2_decoder: *mut mpeg2dec_t,
    last_decoded_vobu: usize,
}

pub struct FullVtsFilter {
    vi: VideoInfo,
    //vts: IndexedVts,
    vobus: Vec<Vobu>,
    vobu_lookup: Vec<usize>,
    absolute_frame: Vec<u64>,

    info_frame: VideoFrame,

    mut_stuff: Arc<Mutex<FullFilterMutStuff>>,
}

impl Filter for FullVtsFilter {
    type Error = CString;
    type FrameType = VideoFrame;
    type FilterData = ();

    fn create(
        input: MapRef<'_>,
        //mut output: MapMut<'_>,
        output: MapMut<'_>,
        _data: Option<Box<Self::FilterData>>,
        mut core: CoreRef,
    ) -> Result<(), Self::Error> {
        unsafe {
            let open_dvd_vobus = open_dvd_vobus(input);
            let reader = open_dvd_vobus.reader;
            let vobus = open_dvd_vobus.vobus;

            let vobus: Vec<Vobu> = vobus.into_iter().map(|e| e.v).collect();

            let seq = &vobus[0].gops[0].sequence_header;
            let mut num_frames = 0;

            let mut vobu_lookup = Vec::new();
            let mut absolute_frame = Vec::new();

            for (i, a) in vobus.iter().enumerate() {
                absolute_frame.push(num_frames as u64);
                for g in &a.gops {
                    num_frames += g.frames.len();
                    for _ in 0..g.frames.len() {
                        vobu_lookup.push(i);
                    }
                }
            }

            let vi = VideoInfo {
                format: vapoursynth4_rs::ffi::VSVideoFormat {
                    color_family: vapoursynth4_rs::ffi::VSColorFamily::YUV,
                    sample_type: vapoursynth4_rs::ffi::VSSampleType::Integer,
                    bits_per_sample: 8,
                    bytes_per_sample: 1,
                    sub_sampling_w: 1,
                    sub_sampling_h: 1,
                    num_planes: 3,
                },
                fps_num: seq.fr_num as _,
                fps_den: seq.fr_den as _,
                width: seq.width as _,
                height: seq.height as _,
                num_frames: num_frames as _,
            };

            let info_frame = {
                let mut dada = Vec::with_capacity(vi.num_frames as usize * 4);
                for a in &vobus {
                    for gg in &a.gops {
                        let mut frames = gg.frames.clone();
                        frames.sort_by(|a, b| {
                            a.temporal_reference
                                .partial_cmp(&b.temporal_reference)
                                .unwrap()
                        });
                        for f in frames {
                            let mut status_byte = 0;
                            if f.tff {
                                status_byte += 1
                            };
                            if f.rff {
                                status_byte += 2
                            };
                            if f.prog {
                                status_byte += 4
                            };
                            if gg.prog_sequence {
                                status_byte += 8
                            };
                            let vob0 = (a.vobcell.vob >> 8) as u8;
                            let vob1 = (a.vobcell.vob & 0xFF) as u8;
                            let cel = a.vobcell.cell;
                            dada.write_all(&[status_byte, vob0, vob1, cel]).unwrap();
                        }
                    }
                }
                let format = core.get_video_format_by_id(VSPresetVideoFormat::Gray8 as u32);
                let mut newframe = core.new_video_frame(&format, dada.len() as _, 1, None);

                std::slice::from_raw_parts_mut(newframe.plane_mut(0), newframe.frame_width(0) as _)
                    .copy_from_slice(&dada);
                newframe
            };
            if cfg!(target_os = "windows") {
                mpeg2_accel(0);
            }
            let filter = FullVtsFilter {
                vi: vi.clone(),
                //     vts: indexv,
                vobus,
                vobu_lookup,
                absolute_frame,
                info_frame,
                mut_stuff: Arc::new(Mutex::new(FullFilterMutStuff {
                    frame_cache: HashMap::new(),
                    reader,
                    mpeg2_decoder: mpeg2_init(),
                    last_decoded_vobu: usize::MAX - 14,
                })),
            };

            let deps = [];
            //output.set(
            //    key!("info"),
            //    vapoursynth4_rs::map::Value::VideoFrame(filter.info_frame.clone()),
            //    VSMapAppendMode::Replace,
            //)
            //.unwrap();
            core.create_video_filter(
                output,
                cstr!("FullVts"),
                &vi,
                Box::new(filter),
                Dependencies::new(&deps).unwrap(),
            );

            Ok(())
        }
    }

    fn get_frame(
        &self,
        n: i32,
        _activation_reason: ActivationReason,
        _frame_data: *mut *mut c_void,
        _ctx: FrameContext,
        core: CoreRef,
    ) -> Result<Option<VideoFrame>, Self::Error> {
        use dvdsrccommon::bindings::mpeg2::*;
        unsafe {
            let mut mut_stuff = self.mut_stuff.lock().unwrap();
            {
                if mut_stuff.frame_cache.contains_key(&n) {
                    return Ok(mut_stuff.frame_cache.remove(&n));
                }
            }
            let mpeg2dec = mut_stuff.mpeg2_decoder.clone();
            let ldv = mut_stuff.last_decoded_vobu.clone();
            let mut reader_ref = &mut mut_stuff.reader;

            let info = mpeg2_info(mpeg2dec);

            let target_vobu = self.vobu_lookup[n as usize];

            //TODO: LAST VOBU WITH VIDEO
            let last_vobu = self.vobus.len() - 1 == target_vobu as usize;

            let vobu = &self.vobus[target_vobu];

            let rebuf = !(vobu.gops[0].closed || ldv as i32 == target_vobu as i32 - 1);

            let can_rebuf = target_vobu >= 1;
            let non_closed_gop_but_rebuf = rebuf && !can_rebuf;

            //let did_acually_rebuf = rebuf && can_rebuf;
            if rebuf && can_rebuf {
                //TODO: PREV VOBU WITH VIDEO
                let pre_vobu = &self.vobus[target_vobu - 1];

                let mut buffer = Vec::with_capacity(pre_vobu.mpeg2_video_size as usize + 4);
                reader_ref
                    .seek(SeekFrom::Start(pre_vobu.sector_start as u64 * 2048))
                    .unwrap();
                demux_video(
                    reader_ref.deref_mut(),
                    pre_vobu.mpeg2_video_size as _,
                    &mut buffer,
                )
                .unwrap();
                //buffer.append(&mut [0x00, 0x00, 0x01, 0xB3].to_vec());
                buffer.append(&mut [0x00, 0x00, 0x01, 0xB3].to_vec());
                //buffer.append(&mut [0x00, 0x00, 0x01, 0xB7].to_vec());
                //buffer.append(&mut [0x00, 0x00, 0x01, 0xB7].to_vec());
                mpeg2_reset(mpeg2dec, 1);

                mpeg2_buffer(
                    mpeg2dec,
                    buffer.as_mut_ptr(),
                    buffer.as_mut_ptr().offset(buffer.len() as isize),
                );

                loop {
                    let state = mpeg2_parse(mpeg2dec);
                    if state == STATE_BUFFER
                        || state == STATE_END
                        || state == STATE_INVALID
                        || state == STATE_INVALID_END
                    {
                        break;
                    }
                }
            }

            let mut buffer = Vec::with_capacity(vobu.mpeg2_video_size as usize + 4);
            reader_ref
                .seek(SeekFrom::Start(vobu.sector_start as u64 * 2048))
                .unwrap();
            demux_video(
                reader_ref.deref_mut(),
                vobu.mpeg2_video_size as _,
                &mut buffer,
            )
            .unwrap();
            if !vobu.mpeg2_has_seq_end {
                if last_vobu {
                    buffer.append(&mut [0x00, 0x00, 0x01, 0xB7].to_vec());
                } else {
                    buffer.append(&mut [0x00, 0x00, 0x01, 0xB3].to_vec());
                }
            }
            mpeg2_reset(mpeg2dec, 0);

            mpeg2_buffer(
                mpeg2dec,
                buffer.as_mut_ptr().offset(0),
                buffer.as_mut_ptr().offset(buffer.len() as isize),
            );

            mut_stuff.last_decoded_vobu = target_vobu;

            let mut absolute_decode_order_frame_idx = 0u8;
            let mut pic_idx = 0;
            let mut slice_idx = 0;

            let mut frames = HashMap::<u8, VideoFrame>::new();
            let mut statey = 0;
            let mut gop_number = 0;
            let mut returna = None;
            loop {
                let state = mpeg2_parse(mpeg2dec);
                match state {
                    STATE_SEQUENCE | STATE_SEQUENCE_MODIFIED | STATE_SEQUENCE_REPEATED => {
                        if gop_number == 0 {
                            assert_eq!(statey, 0);
                        } else {
                            assert_eq!(pic_idx, vobu.gops[gop_number - 1].frames.len());
                            assert_eq!(slice_idx, vobu.gops[gop_number - 1].frames.len());
                        }
                        statey = 1;
                        gop_number += 1;
                        pic_idx = 0;
                        slice_idx = 0;
                    }
                    STATE_PICTURE => {
                        assert_eq!(statey, 2);
                        statey = 3;
                        let oy = &vobu.gops[gop_number - 1].frames[pic_idx];
                        assert_eq!(
                            oy.real_temporal_reference,
                            (*(*info).current_picture).temporal_reference as u8
                        );
                        assert_eq!(
                            oy.tff,
                            (((*(*info).current_picture).flags) & PIC_FLAG_TOP_FIELD_FIRST) != 0
                        );
                        assert_eq!(
                            oy.rff,
                            (((*(*info).current_picture).flags) & PIC_FLAG_REPEAT_FIRST_FIELD) != 0
                        );
                        assert_eq!(
                            oy.prog,
                            (((*(*info).current_picture).flags) & PIC_FLAG_PROGRESSIVE_FRAME) != 0
                        );

                        pic_idx += 1;
                    }
                    STATE_GOP => {
                        assert_eq!(statey, 1);
                        statey = 2;
                    }
                    STATE_SLICE | STATE_END => {
                        if slice_idx == vobu.gops[gop_number - 1].frames.len() {
                            break;
                        }
                        let oy = &vobu.gops[gop_number - 1].frames[slice_idx];
                        //if statey == 0 {
                        //    continue;
                        //}
                        assert_eq!(statey, 3);
                        statey = 2;

                        let mut newframe = core.new_video_frame(
                            &self.vi.format,
                            self.vi.width,
                            self.vi.height,
                            None,
                        );
                        tag_frame1(newframe.properties_mut().unwrap(), *(*info).sequence);
                        tag_frame2(
                            newframe.properties_mut().unwrap(),
                            oy.tff,
                            oy.prog,
                            oy.frametype.clone(),
                        );

                        {
                            let mut mm = newframe.properties_mut().unwrap();
                            mm.set(
                                key!("InfoFrame"),
                                vapoursynth4_rs::map::Value::VideoFrame(self.info_frame.clone()),
                                VSMapAppendMode::Replace,
                            )
                            .unwrap();
                        }
                        let mut mark_invalid = false;
                        if non_closed_gop_but_rebuf {
                            let i_frame = &vobu.gops[gop_number - 1].frames[0];
                            assert_eq!(i_frame.frametype, FrameType::I);

                            let target_frame = &vobu.gops[gop_number - 1].frames[slice_idx];
                            if target_frame.temporal_reference < i_frame.temporal_reference {
                                mark_invalid = true;
                            }
                        }
                        if mark_invalid {
                            fille_frame_black(&mut newframe);
                        } else {
                            //assert_eq!(newframe.stride(0), self.vi.width as isize);
                            //assert_eq!(newframe.stride(1), self.vi.width as isize / 2);
                            //assert_eq!(newframe.stride(2), self.vi.width as isize / 2);
                            let fbuf = *(*info).current_fbuf;

                            for i in [0, 1, 2] {
                                //let ll = self.vi.width * self.vi.height;
                                let mut w = self.vi.width as usize;
                                let mut h = self.vi.height as usize;
                                if i != 0 {
                                    w >>= self.vi.format.sub_sampling_w;
                                    h >>= self.vi.format.sub_sampling_h;
                                }

                                let srcptr = fbuf.buf[i as usize];
                                let destptr = newframe.plane_mut(i);

                                let strd_src = w as isize;
                                let strd_target = newframe.stride(i);

                                for a in 0..h as isize {
                                    let target = std::slice::from_raw_parts_mut(
                                        destptr.offset(a * strd_target),
                                        w as _,
                                    );
                                    let src = std::slice::from_raw_parts_mut(
                                        srcptr.offset(a * strd_src),
                                        w as _,
                                    );

                                    target.clone_from_slice(src);
                                }
                            }
                        }
                        frames.insert(absolute_decode_order_frame_idx, newframe);
                        absolute_decode_order_frame_idx += 1;
                        slice_idx += 1;
                    }
                    //   STATE_END => break,
                    STATE_BUFFER => break,
                    //STATE_BUFFER => {
                    //    break;
                    //},
                    e => panic!("{}", e),
                }
            }
            assert_eq!(pic_idx, vobu.gops[gop_number - 1].frames.len());
            assert_eq!(slice_idx, vobu.gops[gop_number - 1].frames.len());

            let temporal_offset_we_looking = n as u64 - self.absolute_frame[target_vobu];

            let mut offset = 0;
            let mut nnframes = Vec::new();
            for g in &vobu.gops {
                nnframes.append(
                    &mut g
                        .frames
                        .iter()
                        .map(|e| {
                            let mut e = e.clone();
                            e.temporal_reference += offset;
                            e
                        })
                        .collect::<Vec<dvdsrccommon::index::Frame>>(),
                );

                offset += g.frames.len() as u8;
            }

            for f in frames {
                let fff = &nnframes[f.0 as usize];

                if fff.temporal_reference == temporal_offset_we_looking as u8 {
                    returna = Some(f.1)
                } else {
                    let kk =
                        self.absolute_frame[target_vobu] as i32 + fff.temporal_reference as i32;

                    if !mut_stuff.frame_cache.contains_key(&kk) {
                        mut_stuff.frame_cache.insert(kk, f.1);
                    }
                }
            }

            if returna.is_none() {
                let mut newframe =
                    core.new_video_frame(&self.vi.format, self.vi.width, self.vi.height, None);
                fille_frame_black(&mut newframe);
                returna = Some(newframe);
            }
            if mut_stuff.frame_cache.keys().len() >= 80 {
                mut_stuff.frame_cache.clear();
            }

            return Ok(returna);
        }
    }

    const NAME: &'static CStr = cstr!("FullVts");
    const ARGS: &'static CStr = cstr!("path:data;vts:int;ranges:int[]:opt;");
    const RETURN_TYPE: &'static CStr = cstr!("clip:vnode;");
    //const RETURN_TYPE: &'static CStr = cstr!("clip:vnode;info:vframe;");
    const FILTER_MODE: FilterMode = FilterMode::Unordered;
}

fn demux_video(
    mut rdd: impl Read,
    sz2: usize,
    video_buffer: &mut Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    if sz2 == 0 {
        return Ok(());
    }
    let mut scratch = [0u8; 2048];
    let mut buffer = [0u8; 2048];
    loop {
        rdd.read_exact(&mut buffer)?;
        let mut b = Cursor::new(&buffer);
        assert_eq!(start_code(&mut b)?, 0xBA);

        b.read_exact(&mut scratch[0..10])?;
        loop {
            let nxt = start_code(&mut b)?;
            let sz = b.read_u16::<BE>()? as usize;

            match nxt {
                0xBF | 0xBD | 0xBE | 0xBB => {
                    handle_scratch_read(&mut b, sz, &mut scratch).unwrap();
                }
                0xE0 => {
                    b.read_exact(&mut scratch[..sz as usize])?;
                    let st = scratch[2] as usize;
                    video_buffer.extend_from_slice(&scratch[3 + st..sz]);
                    assert!(video_buffer.len() <= sz2);
                    if video_buffer.len() == sz2 {
                        return Ok(());
                    }
                }
                e => {
                    panic!("{:X}", e);
                }
            }
            if b.seek(SeekFrom::Current(0))? % 2048 == 0 {
                break;
            }
        }
    }
}

fn fille_frame_black(newframe: &mut VideoFrame) {
    for i in [0, 1, 2] {
        let num = [16u8, 128, 128][i];
        let stride = newframe.stride(i as _);
        let h = newframe.frame_height(i as _);
        unsafe {
            let sliclepar = std::slice::from_raw_parts_mut(
                newframe.plane_mut(i as _),
                (stride as i32 * h as i32) as usize,
            );
            sliclepar.fill(num);
        }
    }
}

fn tag_frame2(mut props: MapMut<'_>, tff: bool, prog: bool, ft: FrameType) {
    props
        .set(
            key!("_FieldBased"),
            Value::Int(if !prog {
                if tff {
                    VSFieldBased::VSC_FIELD_TOP
                } else {
                    VSFieldBased::VSC_FIELD_BOTTOM
                }
            } else {
                VSFieldBased::VSC_FIELD_PROGRESSIVE
            } as _),
            VSMapAppendMode::Replace,
        )
        .unwrap();

    props
        .set(
            key!("_PictType"),
            Value::Utf8(match ft {
                FrameType::I => "I",
                FrameType::P => "P",
                FrameType::B => "B",
            }),
            VSMapAppendMode::Replace,
        )
        .unwrap();
}

fn tag_frame1(mut props: MapMut<'_>, seq: mpeg2_sequence_t) {
    let primaries = match seq.colour_primaries {
        //0 Forbidden
        1 => VSColorPrimaries::VSC_PRIMARIES_BT709,
        //2 => Unspecified Video
        //3 => Reserved
        4 => VSColorPrimaries::VSC_PRIMARIES_BT470_M,
        5 => VSColorPrimaries::VSC_PRIMARIES_BT470_BG,
        6 => VSColorPrimaries::VSC_PRIMARIES_ST170_M,
        7 => VSColorPrimaries::VSC_PRIMARIES_ST240_M,
        // 8-255 Reserved
        _ => VSColorPrimaries::VSC_PRIMARIES_UNSPECIFIED,
    };

    let transfer = match seq.transfer_characteristics {
        //0 Forbidden
        1 => VSTransferCharacteristics::VSC_TRANSFER_BT709,
        //2 => Unspecified Video
        //3 => Reserved
        4 => VSTransferCharacteristics::VSC_TRANSFER_BT470_M,
        5 => VSTransferCharacteristics::VSC_TRANSFER_BT470_BG,
        6 => VSTransferCharacteristics::VSC_TRANSFER_BT601, // SMPTE 170M
        7 => VSTransferCharacteristics::VSC_TRANSFER_ST240_M,
        8 => VSTransferCharacteristics::VSC_TRANSFER_LINEAR,
        //9-255 Reserved
        _ => VSTransferCharacteristics::VSC_TRANSFER_UNSPECIFIED,
    };

    let matrix = match seq.matrix_coefficients {
        //0 Forbidden
        1 => VSMatrixCoefficients::VSC_MATRIX_BT709,
        //2 => Unspecified Video
        //3 => Reserved
        4 => VSMatrixCoefficients::VSC_MATRIX_FCC,
        5 => VSMatrixCoefficients::VSC_MATRIX_BT470_BG,
        6 => VSMatrixCoefficients::VSC_MATRIX_ST170_M,
        7 => VSMatrixCoefficients::VSC_MATRIX_ST240_M,
        //8-255 Reserved
        _ => VSMatrixCoefficients::VSC_MATRIX_UNSPECIFIED,
    };
    props
        .set(
            key!("_Matrix"),
            Value::Int(matrix as _),
            VSMapAppendMode::Replace,
        )
        .unwrap();
    props
        .set(
            key!("_Transfer"),
            Value::Int(transfer as _),
            VSMapAppendMode::Replace,
        )
        .unwrap();
    props
        .set(
            key!("_Primaries"),
            Value::Int(primaries as _),
            VSMapAppendMode::Replace,
        )
        .unwrap();
    props
        .set(
            key!("_ChromaLocation"),
            Value::Int(VSChromaLocation::VSC_CHROMA_LEFT as _),
            VSMapAppendMode::Replace,
        )
        .unwrap();

    //SAR
    if seq.pixel_width > 0 && seq.pixel_height > 0 {
        let mut pw = 0;
        let mut ph = 0;

        let ret =
            unsafe { dvdsrccommon::bindings::mpeg2::mpeg2_guess_aspect(&seq, &mut pw, &mut ph) };

        if ret == 0 {
            pw = seq.pixel_width;
            ph = seq.pixel_height;
        }

        props
            .set(
                key!("_SARNum"),
                Value::Int(pw as _),
                VSMapAppendMode::Replace,
            )
            .unwrap();
        props
            .set(
                key!("_SARDen"),
                Value::Int(ph as _),
                VSMapAppendMode::Replace,
            )
            .unwrap();
    }
}
