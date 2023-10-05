use std::io::{Read, Seek};

use super::dvd_reader::DvdReader;

pub struct ProperDvdReader {
    inner: DvdReader,

    buffer: [u8; 2048],
    current_pos: u64,
}

impl ProperDvdReader {
    pub fn new(inner: DvdReader) -> ProperDvdReader {
        ProperDvdReader {
            buffer: [0; 2048],
            current_pos: 0,
            inner,
        }
    }
}

impl Read for ProperDvdReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bufsz = buf.len();
        if self.current_pos % 2048 == 0 && bufsz >= 2048 {
            let inner_read = self.inner.read(
                self.current_pos / 2048,
                &mut buf[0..2048 * (bufsz as usize / 2048)],
            );

            if inner_read.is_none() {
                return Ok(0);
            }
            let read_cnt = inner_read.unwrap();
            self.current_pos += read_cnt as u64 * 2048;
            Ok(read_cnt as usize * 2048)
        } else {
            let offset = self.current_pos % 2048;
            self.inner
                .read(self.current_pos / 2048, &mut self.buffer)
                .unwrap();
            let wanted = (2048 - offset).min(buf.len() as u64);
            buf[0..wanted as usize]
                .copy_from_slice(&self.buffer[offset as usize..offset as usize + wanted as usize]);
            self.current_pos += wanted as u64;
            Ok(wanted as usize)
        }
    }
}

impl Seek for ProperDvdReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            std::io::SeekFrom::Start(e) => self.current_pos = e,
            std::io::SeekFrom::End(_) => todo!(),
            std::io::SeekFrom::Current(e) => {
                self.current_pos = (self.current_pos as i64 + e) as u64
            }
        }
        return Ok(self.current_pos);
    }
}
