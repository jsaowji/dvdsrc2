pub type __off_t = ::std::os::raw::c_long;
pub type __off64_t = ::std::os::raw::c_long;
pub type off_t = __off_t;

pub type __gnuc_va_list = __builtin_va_list;
pub type va_list = __builtin_va_list;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct dvd_reader_s {
    _unused: [u8; 0],
}
pub type dvd_reader_t = dvd_reader_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct dvd_reader_device_s {
    _unused: [u8; 0],
}
pub type dvd_reader_device_t = dvd_reader_device_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct dvd_file_s {
    _unused: [u8; 0],
}
pub type dvd_file_t = dvd_file_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct dvd_reader_stream_cb {
    pub pf_seek: ::std::option::Option<
        unsafe extern "C" fn(
            p_stream: *mut ::std::os::raw::c_void,
            i_pos: u64,
        ) -> ::std::os::raw::c_int,
    >,
    pub pf_read: ::std::option::Option<
        unsafe extern "C" fn(
            p_stream: *mut ::std::os::raw::c_void,
            buffer: *mut ::std::os::raw::c_void,
            i_read: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub pf_readv: ::std::option::Option<
        unsafe extern "C" fn(
            p_stream: *mut ::std::os::raw::c_void,
            p_iovec: *mut ::std::os::raw::c_void,
            i_blocks: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
}
pub const dvd_logger_level_t_DVD_LOGGER_LEVEL_INFO: dvd_logger_level_t = 0;
pub const dvd_logger_level_t_DVD_LOGGER_LEVEL_ERROR: dvd_logger_level_t = 1;
pub const dvd_logger_level_t_DVD_LOGGER_LEVEL_WARN: dvd_logger_level_t = 2;
pub const dvd_logger_level_t_DVD_LOGGER_LEVEL_DEBUG: dvd_logger_level_t = 3;
pub type dvd_logger_level_t = ::std::os::raw::c_uint;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct dvd_logger_cb {
    pub pf_log: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::std::os::raw::c_void,
            arg2: dvd_logger_level_t,
            arg3: *const ::std::os::raw::c_char,
            arg4: *mut __va_list_tag,
        ),
    >,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct dvd_stat_t {
    pub size: off_t,
    pub nr_parts: ::std::os::raw::c_int,
    pub parts_size: [off_t; 9usize],
}
extern "C" {
    pub fn DVDOpen(arg1: *const ::std::os::raw::c_char) -> *mut dvd_reader_t;
}
extern "C" {
    pub fn DVDOpenStream(
        arg1: *mut ::std::os::raw::c_void,
        arg2: *mut dvd_reader_stream_cb,
    ) -> *mut dvd_reader_t;
}
extern "C" {
    pub fn DVDOpen2(
        arg1: *mut ::std::os::raw::c_void,
        arg2: *const dvd_logger_cb,
        arg3: *const ::std::os::raw::c_char,
    ) -> *mut dvd_reader_t;
}
extern "C" {
    pub fn DVDOpenStream2(
        arg1: *mut ::std::os::raw::c_void,
        arg2: *const dvd_logger_cb,
        arg3: *mut dvd_reader_stream_cb,
    ) -> *mut dvd_reader_t;
}
extern "C" {
    pub fn DVDClose(arg1: *mut dvd_reader_t);
}
pub const dvd_read_domain_t_DVD_READ_INFO_FILE: dvd_read_domain_t = 0;
pub const dvd_read_domain_t_DVD_READ_INFO_BACKUP_FILE: dvd_read_domain_t = 1;
pub const dvd_read_domain_t_DVD_READ_MENU_VOBS: dvd_read_domain_t = 2;
pub const dvd_read_domain_t_DVD_READ_TITLE_VOBS: dvd_read_domain_t = 3;
pub type dvd_read_domain_t = ::std::os::raw::c_uint;
extern "C" {
    pub fn DVDFileStat(
        arg1: *mut dvd_reader_t,
        arg2: ::std::os::raw::c_int,
        arg3: dvd_read_domain_t,
        arg4: *mut dvd_stat_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DVDOpenFile(
        arg1: *mut dvd_reader_t,
        arg2: ::std::os::raw::c_int,
        arg3: dvd_read_domain_t,
    ) -> *mut dvd_file_t;
}
extern "C" {
    pub fn DVDCloseFile(arg1: *mut dvd_file_t);
}
extern "C" {
    pub fn DVDReadBlocks(
        arg1: *mut dvd_file_t,
        arg2: ::std::os::raw::c_int,
        arg3: usize,
        arg4: *mut ::std::os::raw::c_uchar,
    ) -> isize;
}
extern "C" {
    pub fn DVDFileSeek(arg1: *mut dvd_file_t, arg2: i32) -> i32;
}
extern "C" {
    pub fn DVDReadBytes(
        arg1: *mut dvd_file_t,
        arg2: *mut ::std::os::raw::c_void,
        arg3: usize,
    ) -> isize;
}
extern "C" {
    pub fn DVDFileSize(arg1: *mut dvd_file_t) -> isize;
}
extern "C" {
    pub fn DVDDiscID(
        arg1: *mut dvd_reader_t,
        arg2: *mut ::std::os::raw::c_uchar,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DVDUDFVolumeInfo(
        arg1: *mut dvd_reader_t,
        arg2: *mut ::std::os::raw::c_char,
        arg3: ::std::os::raw::c_uint,
        arg4: *mut ::std::os::raw::c_uchar,
        arg5: ::std::os::raw::c_uint,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DVDFileSeekForce(
        arg1: *mut dvd_file_t,
        offset: ::std::os::raw::c_int,
        force_size: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DVDISOVolumeInfo(
        arg1: *mut dvd_reader_t,
        arg2: *mut ::std::os::raw::c_char,
        arg3: ::std::os::raw::c_uint,
        arg4: *mut ::std::os::raw::c_uchar,
        arg5: ::std::os::raw::c_uint,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DVDUDFCacheLevel(
        arg1: *mut dvd_reader_t,
        arg2: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
pub type __builtin_va_list = [__va_list_tag; 1usize];
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __va_list_tag {
    pub gp_offset: ::std::os::raw::c_uint,
    pub fp_offset: ::std::os::raw::c_uint,
    pub overflow_arg_area: *mut ::std::os::raw::c_void,
    pub reg_save_area: *mut ::std::os::raw::c_void,
}
