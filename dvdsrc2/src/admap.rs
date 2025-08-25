use std::ffi::{c_void, CStr, CString};

use const_str::cstr;
use dvdsrccommon::do_index_dvd::{get_index_vts, DvdBlockReaderDomain};
use vapoursynth4_rs::{
    core::CoreRef,
    frame::{FrameContext, VideoFrame},
    key,
    map::MapRef,
    node::{ActivationReason, Filter, FilterMode},
};

pub struct AdmapFilter {}

impl Filter for AdmapFilter {
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
            .expect("Failed to get dvdpath");
        let vts = input.get_int(key!(c"vts"), 0).expect("Failed to get vts");
        let indexv = get_index_vts(dvdpath, vts as _, DvdBlockReaderDomain::Title);
        let vava: Vec<i64> = indexv.vobus.iter().map(|e| e.sector_start as i64).collect();
        output.set_int_array(key!(c"admap"), &vava).unwrap();
        Ok(())
    }

    fn get_frame(
        &self,
        _n: i32,
        _activation_reason: ActivationReason,
        _frame_data: *mut *mut c_void,
        _ctx: FrameContext,
        _core: CoreRef,
    ) -> Result<Option<VideoFrame>, Self::Error> {
        unreachable!();
    }

    const NAME: &'static CStr = cstr!("Admap");
    const ARGS: &'static CStr = cstr!("path:data;vts:int;");
    const RETURN_TYPE: &'static CStr = cstr!("admap:int[];");
    const FILTER_MODE: FilterMode = FilterMode::Unordered;
}
