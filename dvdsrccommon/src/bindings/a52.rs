pub const A52_CHANNEL: u32 = 0;
pub const A52_MONO: u32 = 1;
pub const A52_STEREO: u32 = 2;
pub const A52_3F: u32 = 3;
pub const A52_2F1R: u32 = 4;
pub const A52_3F1R: u32 = 5;
pub const A52_2F2R: u32 = 6;
pub const A52_3F2R: u32 = 7;
pub const A52_CHANNEL1: u32 = 8;
pub const A52_CHANNEL2: u32 = 9;
pub const A52_DOLBY: u32 = 10;
pub const A52_CHANNEL_MASK: u32 = 15;
pub const A52_LFE: u32 = 16;
pub const A52_ADJUST_LEVEL: u32 = 32;

pub type sample_t = f32;
pub type level_t = f32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct a52_state_s {
    _unused: [u8; 0],
}
pub type a52_state_t = a52_state_s;

extern "C" {
    pub fn a52_init(mm_accel: u32) -> *mut a52_state_t;
    pub fn a52_samples(state: *mut a52_state_t) -> *mut sample_t;
    pub fn a52_syncinfo(
        buf: *mut u8,
        flags: *mut ::std::os::raw::c_int,
        sample_rate: *mut ::std::os::raw::c_int,
        bit_rate: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
    pub fn a52_frame(
        state: *mut a52_state_t,
        buf: *mut u8,
        flags: *mut ::std::os::raw::c_int,
        level: *mut level_t,
        bias: sample_t,
    ) -> ::std::os::raw::c_int;
    pub fn a52_dynrng(
        state: *mut a52_state_t,
        call: ::std::option::Option<
            unsafe extern "C" fn(arg1: level_t, arg2: *mut ::std::os::raw::c_void) -> level_t,
        >,
        data: *mut ::std::os::raw::c_void,
    );
    pub fn a52_block(state: *mut a52_state_t) -> ::std::os::raw::c_int;
    pub fn a52_free(state: *mut a52_state_t);
}
