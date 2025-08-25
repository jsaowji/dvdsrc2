#[allow(dead_code)]
pub mod bindings;

pub mod dvdio;

pub mod audio_demuxing;
pub mod audio_frame_reader;
pub mod do_index_dvd;
pub mod index;
pub mod mpeg2video;
pub mod mpegps;

pub mod vobu_range;
use std::{ffi::CString, ptr::null_mut};

use bindings::dvdread::dvd_reader_t;
pub use byteorder;

use crate::bindings::dvdread::DVDOpen2;

extern "C" fn null_logfn(//    _arg1: *mut ::std::os::raw::c_void,
//    _arg2: dvd_logger_level_t,
//    _arg3: *const ::std::os::raw::c_char,
//    _arg4: *mut __va_list_tag,
) {
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DvdLoggerCb2 {
    pub pf_log: ::std::option::Option<unsafe extern "C" fn()>,
}

pub const LOGGER_CB: DvdLoggerCb2 = DvdLoggerCb2 {
    pf_log: Some(null_logfn as _),
};

pub fn open_dvd(stra: &str) -> Result<*mut dvd_reader_t, ()> {
    let cas = CString::new(stra).unwrap();
    let pa = &LOGGER_CB as *const _;
    let dvd = unsafe { DVDOpen2(null_mut(), pa as *const _, cas.as_ptr()) };
    if dvd.is_null() {
        Err(())
    } else {
        Ok(dvd)
    }
}
