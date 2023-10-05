use std::io::{Read, Seek};

pub struct CacheSeekReader<A> {
    inner: A,

    buffer: Vec<u8>,
    buffer_pos: u64,
    buffer_len: usize,

    current_pos: u64,
}

impl<A: Read + Seek> CacheSeekReader<A> {
    pub fn new(inner: A) -> CacheSeekReader<A> {
        CacheSeekReader {
            inner,
            buffer: vec![0u8; 2048 * 12],
            buffer_pos: u64::MAX,
            buffer_len: 0,
            current_pos: 0,
        }
    }
}
impl<A: Read + Seek> Read for CacheSeekReader<A> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.current_pos >= self.buffer_pos
            && self.current_pos < self.buffer_pos + self.buffer.len() as u64
        {
            //read sth from buffer
            let buffer_offset = self.current_pos - self.buffer_pos;
            let buffer_left = self.buffer.len() as u64 - buffer_offset;
            let read_size = buffer_left
                .min(buf.len() as _)
                .min(self.buffer_len as u64 - buffer_offset);
            buf[0..read_size as usize].copy_from_slice(
                &self.buffer[buffer_offset as usize..buffer_offset as usize + read_size as usize],
            );
            self.current_pos += read_size;
            return Ok(read_size as _);
        }

        self.inner
            .seek(std::io::SeekFrom::Start(self.current_pos))
            .unwrap();
        self.buffer_len = self.inner.read(&mut self.buffer[0..]).unwrap();

        self.buffer_pos = self.current_pos;
        return self.read(buf);
    }
}

impl<A: Read + Seek> Seek for CacheSeekReader<A> {
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
