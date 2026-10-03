// The C names in this crate are libjpeg's, spelled as `jpeglib.h` spells them so
// they can be looked up directly. That trips a few naming lints, including at the
// constant's use site in a pattern, so they are silenced crate wide rather than
// per module.
#![allow(non_camel_case_types, non_snake_case, dead_code)]
#![allow(clippy::upper_case_acronyms, non_upper_case_globals)]

mod jpeg;
mod libjpeg_sys;

pub use jpeg::JpegDecoder;
pub use libjpeg_sys::LibJpeg;
