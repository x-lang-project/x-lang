use std::ffi::c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    pub(crate) fn GetFileInformationByHandle(
        handle: *mut c_void,
        info: *mut FileInformation,
    ) -> i32;
    pub(crate) fn GetFileInformationByHandleEx(
        handle: *mut c_void,
        class: u32,
        info: *mut c_void,
        size: u32,
    ) -> i32;
}
#[repr(C)]
#[derive(Default)]
pub(crate) struct FileInformation {
    pub(crate) attributes: u32,
    pub(crate) created: [u32; 2],
    pub(crate) accessed: [u32; 2],
    pub(crate) modified: [u32; 2],
    pub(crate) volume: u32,
    pub(crate) size_high: u32,
    pub(crate) size_low: u32,
    pub(crate) links: u32,
    pub(crate) file_identifier_high: u32,
    pub(crate) file_identifier_low: u32,
}

#[repr(C)]
#[derive(Default)]
pub(crate) struct BasicInformation {
    pub(crate) created: i64,
    pub(crate) accessed: i64,
    pub(crate) modified: i64,
    pub(crate) changed: i64,
    pub(crate) attributes: u32,
}
