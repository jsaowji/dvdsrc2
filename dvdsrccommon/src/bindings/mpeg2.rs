pub const SEQ_FLAG_MPEG2: u32 = 1;
pub const SEQ_FLAG_CONSTRAINED_PARAMETERS: u32 = 2;
pub const SEQ_FLAG_PROGRESSIVE_SEQUENCE: u32 = 4;
pub const SEQ_FLAG_LOW_DELAY: u32 = 8;
pub const SEQ_FLAG_COLOUR_DESCRIPTION: u32 = 16;
pub const SEQ_MASK_VIDEO_FORMAT: u32 = 224;
pub const SEQ_VIDEO_FORMAT_COMPONENT: u32 = 0;
pub const SEQ_VIDEO_FORMAT_PAL: u32 = 32;
pub const SEQ_VIDEO_FORMAT_NTSC: u32 = 64;
pub const SEQ_VIDEO_FORMAT_SECAM: u32 = 96;
pub const SEQ_VIDEO_FORMAT_MAC: u32 = 128;
pub const SEQ_VIDEO_FORMAT_UNSPECIFIED: u32 = 160;

pub const GOP_FLAG_DROP_FRAME: u32 = 1;
pub const GOP_FLAG_BROKEN_LINK: u32 = 2;
pub const GOP_FLAG_CLOSED_GOP: u32 = 4;

pub const PIC_MASK_CODING_TYPE: u32 = 7;
pub const PIC_FLAG_CODING_TYPE_I: u32 = 1;
pub const PIC_FLAG_CODING_TYPE_P: u32 = 2;
pub const PIC_FLAG_CODING_TYPE_B: u32 = 3;
pub const PIC_FLAG_CODING_TYPE_D: u32 = 4;
pub const PIC_FLAG_TOP_FIELD_FIRST: u32 = 8;
pub const PIC_FLAG_PROGRESSIVE_FRAME: u32 = 16;
pub const PIC_FLAG_COMPOSITE_DISPLAY: u32 = 32;
pub const PIC_FLAG_SKIP: u32 = 64;
pub const PIC_FLAG_TAGS: u32 = 128;
pub const PIC_FLAG_REPEAT_FIRST_FIELD: u32 = 256;
pub const PIC_MASK_COMPOSITE_DISPLAY: u32 = 4294963200;

pub const MPEG2_ACCEL_X86_MMX: u32 = 1;
pub const MPEG2_ACCEL_X86_3DNOW: u32 = 2;
pub const MPEG2_ACCEL_X86_MMXEXT: u32 = 4;
pub const MPEG2_ACCEL_X86_SSE2: u32 = 8;
pub const MPEG2_ACCEL_X86_SSE3: u32 = 16;
pub const MPEG2_ACCEL_PPC_ALTIVEC: u32 = 1;
pub const MPEG2_ACCEL_ALPHA: u32 = 1;
pub const MPEG2_ACCEL_ALPHA_MVI: u32 = 2;
pub const MPEG2_ACCEL_SPARC_VIS: u32 = 1;
pub const MPEG2_ACCEL_SPARC_VIS2: u32 = 2;
pub const MPEG2_ACCEL_ARM: u32 = 1;
pub const MPEG2_ACCEL_DETECT: u32 = 2147483648;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mpeg2_sequence_s {
    pub width: ::std::os::raw::c_uint,
    pub height: ::std::os::raw::c_uint,
    pub chroma_width: ::std::os::raw::c_uint,
    pub chroma_height: ::std::os::raw::c_uint,
    pub byte_rate: ::std::os::raw::c_uint,
    pub vbv_buffer_size: ::std::os::raw::c_uint,
    pub flags: u32,
    pub picture_width: ::std::os::raw::c_uint,
    pub picture_height: ::std::os::raw::c_uint,
    pub display_width: ::std::os::raw::c_uint,
    pub display_height: ::std::os::raw::c_uint,
    pub pixel_width: ::std::os::raw::c_uint,
    pub pixel_height: ::std::os::raw::c_uint,
    pub frame_period: ::std::os::raw::c_uint,
    pub profile_level_id: u8,
    pub colour_primaries: u8,
    pub transfer_characteristics: u8,
    pub matrix_coefficients: u8,
}
pub type mpeg2_sequence_t = mpeg2_sequence_s;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mpeg2_gop_s {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub pictures: u8,
    pub flags: u32,
}
pub type mpeg2_gop_t = mpeg2_gop_s;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mpeg2_picture_s {
    pub temporal_reference: ::std::os::raw::c_uint,
    pub nb_fields: ::std::os::raw::c_uint,
    pub tag: u32,
    pub tag2: u32,
    pub flags: u32,
    pub display_offset: [mpeg2_picture_s__bindgen_ty_1; 3usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mpeg2_picture_s__bindgen_ty_1 {
    pub x: ::std::os::raw::c_int,
    pub y: ::std::os::raw::c_int,
}
pub type mpeg2_picture_t = mpeg2_picture_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mpeg2_fbuf_s {
    pub buf: [*mut u8; 3usize],
    pub id: *mut ::std::os::raw::c_void,
}
pub type mpeg2_fbuf_t = mpeg2_fbuf_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mpeg2_info_s {
    pub sequence: *const mpeg2_sequence_t,
    pub gop: *const mpeg2_gop_t,
    pub current_picture: *const mpeg2_picture_t,
    pub current_picture_2nd: *const mpeg2_picture_t,
    pub current_fbuf: *const mpeg2_fbuf_t,
    pub display_picture: *const mpeg2_picture_t,
    pub display_picture_2nd: *const mpeg2_picture_t,
    pub display_fbuf: *const mpeg2_fbuf_t,
    pub discard_fbuf: *const mpeg2_fbuf_t,
    pub user_data: *const u8,
    pub user_data_len: ::std::os::raw::c_uint,
}
pub type mpeg2_info_t = mpeg2_info_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mpeg2dec_s {
    _unused: [u8; 0],
}
pub type mpeg2dec_t = mpeg2dec_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mpeg2_decoder_s {
    _unused: [u8; 0],
}
pub type mpeg2_decoder_t = mpeg2_decoder_s;

pub const mpeg2_state_t_STATE_BUFFER: mpeg2_state_t = 0;
pub const mpeg2_state_t_STATE_SEQUENCE: mpeg2_state_t = 1;
pub const mpeg2_state_t_STATE_SEQUENCE_REPEATED: mpeg2_state_t = 2;
pub const mpeg2_state_t_STATE_GOP: mpeg2_state_t = 3;
pub const mpeg2_state_t_STATE_PICTURE: mpeg2_state_t = 4;
pub const mpeg2_state_t_STATE_SLICE_1ST: mpeg2_state_t = 5;
pub const mpeg2_state_t_STATE_PICTURE_2ND: mpeg2_state_t = 6;
pub const mpeg2_state_t_STATE_SLICE: mpeg2_state_t = 7;
pub const mpeg2_state_t_STATE_END: mpeg2_state_t = 8;
pub const mpeg2_state_t_STATE_INVALID: mpeg2_state_t = 9;
pub const mpeg2_state_t_STATE_INVALID_END: mpeg2_state_t = 10;
pub const mpeg2_state_t_STATE_SEQUENCE_MODIFIED: mpeg2_state_t = 11;
pub type mpeg2_state_t = ::std::os::raw::c_uint;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mpeg2_convert_init_s {
    pub id_size: ::std::os::raw::c_uint,
    pub buf_size: [::std::os::raw::c_uint; 3usize],
    pub start: ::std::option::Option<
        unsafe extern "C" fn(
            id: *mut ::std::os::raw::c_void,
            fbuf: *const mpeg2_fbuf_t,
            picture: *const mpeg2_picture_t,
            gop: *const mpeg2_gop_t,
        ),
    >,
    pub copy: ::std::option::Option<
        unsafe extern "C" fn(
            id: *mut ::std::os::raw::c_void,
            src: *const *mut u8,
            v_offset: ::std::os::raw::c_uint,
        ),
    >,
}
pub type mpeg2_convert_init_t = mpeg2_convert_init_s;
pub const mpeg2_convert_stage_t_MPEG2_CONVERT_SET: mpeg2_convert_stage_t = 0;
pub const mpeg2_convert_stage_t_MPEG2_CONVERT_STRIDE: mpeg2_convert_stage_t = 1;
pub const mpeg2_convert_stage_t_MPEG2_CONVERT_START: mpeg2_convert_stage_t = 2;
pub type mpeg2_convert_stage_t = ::std::os::raw::c_uint;
pub type mpeg2_convert_t = ::std::option::Option<
    unsafe extern "C" fn(
        stage: ::std::os::raw::c_int,
        id: *mut ::std::os::raw::c_void,
        sequence: *const mpeg2_sequence_t,
        stride: ::std::os::raw::c_int,
        accel: u32,
        arg: *mut ::std::os::raw::c_void,
        result: *mut mpeg2_convert_init_t,
    ) -> ::std::os::raw::c_int,
>;
extern "C" {
    pub fn mpeg2_convert(
        mpeg2dec: *mut mpeg2dec_t,
        convert: mpeg2_convert_t,
        arg: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
    pub fn mpeg2_stride(
        mpeg2dec: *mut mpeg2dec_t,
        stride: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
    pub fn mpeg2_set_buf(
        mpeg2dec: *mut mpeg2dec_t,
        buf: *mut *mut u8,
        id: *mut ::std::os::raw::c_void,
    );
    pub fn mpeg2_custom_fbuf(mpeg2dec: *mut mpeg2dec_t, custom_fbuf: ::std::os::raw::c_int);
    pub fn mpeg2_accel(accel: u32) -> u32;
    pub fn mpeg2_init() -> *mut mpeg2dec_t;
    pub fn mpeg2_info(mpeg2dec: *mut mpeg2dec_t) -> *const mpeg2_info_t;
    pub fn mpeg2_close(mpeg2dec: *mut mpeg2dec_t);
    pub fn mpeg2_buffer(mpeg2dec: *mut mpeg2dec_t, start: *mut u8, end: *mut u8);
    pub fn mpeg2_getpos(mpeg2dec: *mut mpeg2dec_t) -> ::std::os::raw::c_int;
    pub fn mpeg2_parse(mpeg2dec: *mut mpeg2dec_t) -> mpeg2_state_t;
    pub fn mpeg2_reset(mpeg2dec: *mut mpeg2dec_t, full_reset: ::std::os::raw::c_int);
    pub fn mpeg2_skip(mpeg2dec: *mut mpeg2dec_t, skip: ::std::os::raw::c_int);
    pub fn mpeg2_slice_region(
        mpeg2dec: *mut mpeg2dec_t,
        start: ::std::os::raw::c_int,
        end: ::std::os::raw::c_int,
    );
    pub fn mpeg2_tag_picture(mpeg2dec: *mut mpeg2dec_t, tag: u32, tag2: u32);
    pub fn mpeg2_init_fbuf(
        decoder: *mut mpeg2_decoder_t,
        current_fbuf: *mut *mut u8,
        forward_fbuf: *mut *mut u8,
        backward_fbuf: *mut *mut u8,
    );
    pub fn mpeg2_slice(
        decoder: *mut mpeg2_decoder_t,
        code: ::std::os::raw::c_int,
        buffer: *const u8,
    );
    pub fn mpeg2_guess_aspect(
        sequence: *const mpeg2_sequence_t,
        pixel_width: *mut ::std::os::raw::c_uint,
        pixel_height: *mut ::std::os::raw::c_uint,
    ) -> ::std::os::raw::c_int;

    pub fn mpeg2_malloc(
        size: ::std::os::raw::c_uint,
        reason: mpeg2_alloc_t,
    ) -> *mut ::std::os::raw::c_void;
    pub fn mpeg2_free(buf: *mut ::std::os::raw::c_void);

    pub fn mpeg2_malloc_hooks(
        malloc: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: ::std::os::raw::c_uint,
                arg2: mpeg2_alloc_t,
            ) -> *mut ::std::os::raw::c_void,
        >,
        free: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void) -> ::std::os::raw::c_int,
        >,
    );

}
pub const mpeg2_alloc_t_MPEG2_ALLOC_MPEG2DEC: mpeg2_alloc_t = 0;
pub const mpeg2_alloc_t_MPEG2_ALLOC_CHUNK: mpeg2_alloc_t = 1;
pub const mpeg2_alloc_t_MPEG2_ALLOC_YUV: mpeg2_alloc_t = 2;
pub const mpeg2_alloc_t_MPEG2_ALLOC_CONVERT_ID: mpeg2_alloc_t = 3;
pub const mpeg2_alloc_t_MPEG2_ALLOC_CONVERTED: mpeg2_alloc_t = 4;
pub type mpeg2_alloc_t = ::std::os::raw::c_uint;
