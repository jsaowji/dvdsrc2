use std::{
    ffi::{c_void, CStr, CString},
    ptr::null_mut,
};

use const_str::cstr;
use dvdsrccommon::{
    bindings::dvdread::{
        dvd_read_domain_t_DVD_READ_INFO_FILE, DVDClose, DVDCloseFile, DVDFileStat, DVDOpen2,
        DVDOpenFile, DVDReadBytes,
    },
    LOGGER_CB,
};
use vapoursynth4_rs::{
    core::CoreRef,
    frame::{FrameContext, VideoFrame},
    key,
    map::{MapMut, MapRef},
    node::Filter,
};
use vapoursynth4_sys::VSMapAppendMode;

pub struct IfoFile {}

impl Filter for IfoFile {
    type Error = CString;
    type FrameType = VideoFrame;
    type FilterData = ();

    fn create(
        input: MapRef<'_>,
        mut output: MapMut<'_>,
        _data: Option<Box<Self::FilterData>>,
        _core: CoreRef,
    ) -> Result<(), Self::Error> {
        let dvdpath = input.get_utf8(key!("path"), 0).expect("Failed to get clip");
        let ifo = input.get_int(key!("ifo"), 0).expect("Failed to get clip");

        let cas = CString::new(dvdpath).unwrap();
        let dvd = unsafe { DVDOpen2(null_mut(), &LOGGER_CB, cas.as_ptr()) };
        assert!(!dvd.is_null());

        unsafe {
            let mut stat = std::mem::zeroed();

            DVDFileStat(
                dvd,
                ifo as _,
                dvd_read_domain_t_DVD_READ_INFO_FILE,
                &mut stat,
            );
            let ifofilesize = stat.size;
            let file = DVDOpenFile(dvd, ifo as _, dvd_read_domain_t_DVD_READ_INFO_FILE);

            let mut buffer = vec![0u8; ifofilesize as usize];

            let rdrd = DVDReadBytes(file, buffer.as_mut_ptr() as _, ifofilesize as _);
            assert_eq!(rdrd as usize, buffer.len());

            output
                .set(
                    key!("file_data"),
                    vapoursynth4_rs::map::Value::Data(&buffer),
                    VSMapAppendMode::Replace,
                )
                .unwrap();

            DVDCloseFile(file);
            DVDClose(dvd);
        }

        Ok(())
    }

    const NAME: &'static CStr = cstr!("Ifo");
    const ARGS: &'static CStr = cstr!("path:data;ifo:int;");
    const RETURN_TYPE: &'static CStr = cstr!("file_data:data;");

    fn get_frame(
        &self,
        _n: i32,
        _activation_reason: vapoursynth4_sys::VSActivationReason,
        _frame_data: *mut *mut c_void,
        _frame_ctx: FrameContext,
        _core: CoreRef,
    ) -> Result<Option<Self::FrameType>, Self::Error> {
        todo!()
    }
}
