use crate::bindings::dvdread::*;

pub struct DvdReader {
    dvd_file: *mut dvd_file_t,
    pub block_cnt: u64,
}

impl DvdReader {
    pub fn new(dvd_file: *mut dvd_file_t) -> DvdReader {
        unsafe {
            let sz = DVDFileSize(dvd_file);
            DvdReader {
                dvd_file,
                block_cnt: sz as _,
            }
        }
    }

    pub fn read(&mut self, block_offset: u64, buf: &mut [u8]) -> Option<u64> {
        assert_eq!(buf.len() % 2048, 0);

        let read_cnt = (buf.len() as u64 / 2048).min(self.block_cnt - block_offset);
        if read_cnt > 0 {
            unsafe {
                DVDReadBlocks(
                    self.dvd_file,
                    block_offset as _,
                    read_cnt as _,
                    buf.as_mut_ptr(),
                );
            }
            Some(read_cnt)
        } else {
            None
        }
    }
}

impl Drop for DvdReader {
    fn drop(&mut self) {
        unsafe {
            DVDCloseFile(self.dvd_file);
        }
    }
}
