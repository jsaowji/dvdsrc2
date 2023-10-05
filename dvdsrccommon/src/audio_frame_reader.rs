use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

use byteorder::{ReadBytesExt, BE};

use crate::{
    audio_demuxing::XAudioFrameVobuInfo,
    dvdio::{cache_seek_reader::CacheSeekReader, proper_dvd_reader::ProperDvdReader},
    mpegps::{pes, start_code},
};

pub struct AudioFrameReader {
    pub reader: CacheSeekReader<ProperDvdReader>,
    pub frame_vobu_info: Vec<XAudioFrameVobuInfo>,
    pub stream_id: u8,
    pub seek_offset: u64,
}

impl AudioFrameReader {
    pub fn ready(&mut self, mut offset: u32, mut size: u64, buf: &mut [u8]) -> u64 {
        let orgi_size = size;
        let mut cursor = Cursor::new(buf);
        for a in &self.frame_vobu_info {
            if offset >= a.demuxed_absolute_offset
                && a.demuxed_size + a.demuxed_absolute_offset > offset
            {
                let local_offset = offset - a.demuxed_absolute_offset;

                let mut audio_data_buffer = Vec::with_capacity(a.real_demuxed_size as usize);

                self.reader
                    .seek(SeekFrom::Start(a.sector_start as u64 * 2048))
                    .unwrap();
                demux_audio(
                    &mut self.reader,
                    a.real_real_demuxed_size as _,
                    &mut audio_data_buffer,
                    self.stream_id,
                    true,
                )
                .unwrap();
                let audio_data_buffer = &audio_data_buffer[a.cut_front..a.cut_back];
                let audio_data_buffer = &audio_data_buffer[local_offset as usize..];
                let read_size = size.min(audio_data_buffer.len() as u64);
                cursor
                    .write_all(&audio_data_buffer[0..read_size as usize])
                    .unwrap();
                size -= read_size as u64;
                offset += read_size as u32;
            }
        }
        return orgi_size - size;
    }
}
impl Read for AudioFrameReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let rd = self.ready(self.seek_offset as u32, buf.len() as _, buf);
        self.seek_offset += rd as u64;
        Ok(rd as usize)
    }
}
impl Seek for AudioFrameReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(e) => self.seek_offset = e,
            SeekFrom::End(_) => todo!(),
            SeekFrom::Current(_) => todo!(),
        };
        Ok(self.seek_offset)
    }
}

pub fn demux_audio(
    mut rdd: impl Read,
    sz2: usize,
    audio_data_buffer: &mut Vec<u8>,
    audio_stream_id: u8,
    strip_pcm: bool,
) -> Result<(), Box<dyn Error>> {
    let mut packets_already = 0;

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
                0xE0 | 0xBF | 0xBE | 0xBB => {
                    b.read_exact(&mut scratch[0..sz as usize])?;
                }
                0xBD => {
                    let pes = pes(&mut b, sz)?;
                    let pes = pes.inner.into_inner();
                    if pes[0] == audio_stream_id {
                        audio_data_buffer.extend_from_slice(
                            &pes[4 + if strip_pcm
                                && (audio_stream_id >= 0xA0 && audio_stream_id <= 0xa7)
                            {
                                3
                            } else {
                                0
                            }..],
                        );
                    }
                    packets_already += 1;

                    let adjeln = sz2 - if strip_pcm { 3 * packets_already } else { 0 };

                    assert!(audio_data_buffer.len() <= adjeln);
                    if audio_data_buffer.len() == adjeln {
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
