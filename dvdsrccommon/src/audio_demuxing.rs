use std::io::{Seek, SeekFrom};

use crate::{
    audio_frame_reader::{demux_audio, AudioFrameReader},
    bindings::a52,
    dvdio::{cache_seek_reader::CacheSeekReader, proper_dvd_reader::ProperDvdReader},
    index::IndexedVts,
    vobu_range::EVobu,
};

#[derive(Debug)]
pub struct XAudioFrameVobuInfo {
    pub sector_start: u32,
    pub demuxed_absolute_offset: u32,
    pub real_real_demuxed_size: u32,
    pub real_demuxed_size: u32,
    pub demuxed_size: u32,
    pub cut_front: usize,
    pub cut_back: usize,
}

pub struct AudioFramesInfo {
    pub reader: CacheSeekReader<AudioFrameReader>,
    pub ac3_fingerprint: (i32, i32, i32),
    pub frame_cnt: u32,
    pub frame_length: u32,
    pub pts_cutoff_start: u32,
    pub pts_cut_end: u32,

    pub samples_per_frame: u32,

    pub lpcm_quant: u32,
    pub lpcm_sample_rate: u32,
    pub lpcm_channels: u32,
}

pub fn raw_audio_frames_init(
    mut reader: CacheSeekReader<ProperDvdReader>,
    index: IndexedVts,
    mut vobus: Vec<EVobu>,
    audio: u8,
    is_lpcm: bool,
) -> AudioFramesInfo {
    let is_ac3 = !is_lpcm;
    let codec_packet_length = if is_ac3 { 2880 } else { 150 };

    let mut pts_cut_end = 0;

    let real_stream_idx = audio as u8 + if is_ac3 { 0x80 } else { 0xA0 };

    //TODO: ENDS WITH LAST VOBU WITH AUDIO
    let ends_with_last_vobu = vobus[vobus.len() - 1].i == index.vobus.len() - 1;
    if !ends_with_last_vobu {
        let after_i = vobus[vobus.len() - 1].i + 1;
        let after_vobu = &index.vobus[after_i];

        let next_vobu_start = after_vobu.first_ptm;
        let strmm = after_vobu
            .streams
            .iter()
            .find(|e| e.id == real_stream_idx)
            .unwrap();

        if strmm.packets.know_units[0].pts < next_vobu_start {
            let mut our_cntt = 0;
            let mut cut_pts = 0;
            'outer: for (_, a) in strmm.packets.know_units.iter().enumerate() {
                for j in 0..a.frame_cnt as u32 {
                    let frame_length = codec_packet_length;
                    let frame_start = a.pts + frame_length * (j);
                    // let frame_end = a.pts + frame_length * (j + 1);

                    if frame_start >= next_vobu_start {
                        break 'outer;
                    } else {
                        cut_pts = next_vobu_start - frame_start
                    }
                    our_cntt += 1;
                }
            }
            let their_packets_left = strmm
                .packets
                .know_units
                .iter()
                .fold(0, |a, b| a + b.frame_cnt) as u32
                - our_cntt;
            pts_cut_end = their_packets_left * codec_packet_length - cut_pts;
            //pts_cut_end

            vobus.push(EVobu {
                i: after_i,
                v: after_vobu.clone(),
            });
        }
    }

    let vobu0strems = vobus[0]
        .v
        .streams
        .iter()
        .find(|e| e.id == real_stream_idx)
        .unwrap();

    let mut stream_buffer = Vec::with_capacity(vobu0strems.packets.total_bytes as _);

    reader
        .seek(SeekFrom::Start(vobus[0].v.sector_start as u64 * 2048))
        .unwrap();
    demux_audio(
        &mut reader,
        vobu0strems.packets.total_bytes as _,
        &mut stream_buffer,
        real_stream_idx as _,
        false,
    )
    .unwrap();

    let mut packet_lenght = 13333338;

    //  let mut cutable_sample_length = 28282;

    let mut ac3_fingerprint = (0, 0, 0);

    let mut samples_per_frame = 6 * 256;

    let mut lpcm_quant = 0;
    let mut lpcm_samplerate = 0;
    let mut lpcm_channels = 0;

    if is_ac3 {
        let ss = determine_packet_length_ac3(
            &mut stream_buffer[vobu0strems.packets.know_units[0].offset as usize..],
        );
        packet_lenght = ss.0;
        //   cutable_sample_length  = ss.0;
        ac3_fingerprint = ss.1;
    } else if is_lpcm {
        let ib = stream_buffer[1];
        lpcm_quant = match (ib & 0b11000000) >> 6 {
            0 => 16,
            1 => 20,
            2 => 24,
            _ => unreachable!(),
        };
        assert!(lpcm_quant == 16 || lpcm_quant == 24);
        lpcm_samplerate = match (ib & 0b00110000) >> 4 {
            0 => 48_000,
            1 => 96_000,
            _ => unreachable!(),
        } as u64;
        lpcm_channels = ((ib & 0b11) + 1) as u64;
        //Frame size in bytes = (sample rate)*(quantization)*(number of channels)/4800.
        samples_per_frame = lpcm_samplerate / 600;

        let frame_size = samples_per_frame * lpcm_quant * lpcm_channels;
        assert_eq!(frame_size % 8, 0);
        let frame_size = frame_size / 8;

        packet_lenght = frame_size as usize;
        //dbg!(packet_lenght);
        //cutable_sample_length = (quant * channels)  as usize / 8;
    }

    //let mut start_unit = 0;
    let mut start_byte_offset = 0;
    let mut start_offset_pts = 0;

    'outer: for (_, a) in vobu0strems.packets.know_units.iter().enumerate() {
        //dbg!(a.pts, vobus[0].first_ptm);
        for j in 0..a.frame_cnt as u32 {
            let frame_length = codec_packet_length;
            let frame_start = a.pts + frame_length * (j);
            let frame_end = a.pts + frame_length * (j + 1);
            if frame_end >= vobus[0].v.first_ptm {
                //TODO: this can be improved
                //if is_ac3 {
                start_offset_pts = vobus[0].v.first_ptm as i32 - frame_start as i32;
                start_byte_offset =
                    a.offset + j * packet_lenght as u32 - if is_lpcm { 3 } else { 0 };
                //} else if is_lpcm{
                //    start_offset_pts = 0;
                //    start_byte_offset = a.offset + ;
                //}

                break 'outer;
            }
        }
    }
    dbg!(start_byte_offset, start_offset_pts);

    let mut audioframevobus = Vec::new();

    //TODO: LAST WITH AUDIO
    let mut sz = 0;
    let last_i = vobus.len() - 1;
    for v in vobus.iter().enumerate() {
        let is_last = v.0 == last_i;
        let aua =
            v.1.v
                .streams
                .iter()
                .find(|e| e.id == real_stream_idx)
                .unwrap();

        // dbg!(aua.packets.abs_packet_cnt);
        let real_demuxed_size = aua.packets.total_bytes
            - if is_lpcm {
                aua.packets.abs_packet_cnt * 3
            } else {
                0
            };

        let mut demuxed_size = real_demuxed_size;
        let cut_front = if v.0 == 0 { start_byte_offset } else { 0 };
        let mut cut_back = real_demuxed_size;
        demuxed_size -= cut_front;

        if is_last {
            let endpos = sz + demuxed_size;
            let tomuch = endpos % packet_lenght as u32;
            demuxed_size -= tomuch;
            cut_back -= tomuch;
        }

        audioframevobus.push(XAudioFrameVobuInfo {
            sector_start: v.1.v.sector_start,
            demuxed_absolute_offset: sz,
            real_real_demuxed_size: aua.packets.total_bytes,
            real_demuxed_size,
            demuxed_size,
            cut_front: cut_front as _,
            cut_back: cut_back as _,
        });
        sz += demuxed_size;
    }
    let total_stream_size = audioframevobus.iter().fold(0, |a, b| a + b.demuxed_size);
    assert_eq!(total_stream_size % packet_lenght as u32, 0);
    let frame_cnt = total_stream_size / packet_lenght as u32;

    AudioFramesInfo {
        reader: CacheSeekReader::new(AudioFrameReader {
            reader,
            frame_vobu_info: audioframevobus,
            stream_id: real_stream_idx,
            seek_offset: 0,
        }),
        ac3_fingerprint,
        frame_length: packet_lenght as _,
        frame_cnt,
        pts_cutoff_start: start_offset_pts as u32,
        pts_cut_end: pts_cut_end as _,

        samples_per_frame: samples_per_frame as _,
        lpcm_quant: lpcm_quant as _,
        lpcm_sample_rate: lpcm_samplerate as _,
        lpcm_channels: lpcm_channels as _,
    }
}

fn determine_packet_length_ac3(audioshi: &mut [u8]) -> (usize, (i32, i32, i32)) {
    unsafe {
        let (mut flagso, mut sample_rate, mut bit_rate) = (0, 0, 0);
        let packet_lenght = a52::a52_syncinfo(
            audioshi.as_mut_ptr(),
            &mut flagso,
            &mut sample_rate,
            &mut bit_rate,
        );
        assert_ne!(packet_lenght, 0);
        (packet_lenght as _, (flagso, sample_rate, bit_rate))
    }
}
