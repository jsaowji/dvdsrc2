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

use bindings::dvdread::{__va_list_tag, dvd_logger_cb, dvd_logger_level_t};
pub use byteorder;

extern "C" fn null_logfn(
    _arg1: *mut ::std::os::raw::c_void,
    _arg2: dvd_logger_level_t,
    _arg3: *const ::std::os::raw::c_char,
    _arg4: *mut __va_list_tag,
) {
}

pub const LOGGER_CB: dvd_logger_cb = dvd_logger_cb {
    pf_log: Some(null_logfn),
};
