use std::ffi::{c_void, CStr, CString};

use const_str::cstr;
use dvdsrccommon::{
    bindings::dvdread::DVDClose,
    do_index_dvd::get_ifo_file,
    //bindings::dvdread::{
    //    dvd_read_domain_t_DVD_READ_INFO_FILE, DVDClose, DVDCloseFile, DVDFileStat, DVDOpenFile,
    //    DVDReadBytes, dvd_reader_t,
    //},
    open_dvd,
};
use vapoursynth4_rs::{
    core::CoreRef,
    frame::{FrameContext, VideoFrame},
    key,
    map::MapRef,
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
        mut output: MapRef<'_>,
        _data: Option<Box<Self::FilterData>>,
        _core: CoreRef,
    ) -> Result<(), Self::Error> {
        let dvdpath = input
            .get_utf8(key!(c"path"), 0)
            .map_err(|_| c"No Path specifiied")?;
        let ifo = input
            .get_int(key!(c"ifo"), 0)
            .map_err(|_| c"No Ifo specifiied")?;

        let dvd = open_dvd(dvdpath.try_into().unwrap()).map_err(|_| c"Couldn't open dvd")?;

        let buffer = get_ifo_file(dvd, ifo as _);

        output
            .set(
                key!(c"file_data"),
                vapoursynth4_rs::map::Value::Data(&buffer),
                VSMapAppendMode::Replace,
            )
            .unwrap();

        unsafe {
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
