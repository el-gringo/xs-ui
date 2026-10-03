//! Raw `libjpeg` bindings.
//!
//! Everything here mirrors `jpeglib.h` from libjpeg 8 / libjpeg-turbo 3.x and is
//! loaded at runtime with `dlopen`, following `xs-egl-sys` and `xs-tls`. Two
//! consequences shape the file:
//!
//! * the struct declarations stop at the last field we read and pad the rest,
//!   so the offsets below are the only thing that has to stay true;
//! * the layouts are pinned by the tests at the bottom of this file, which were
//!   generated from a C program compiled against the real headers.
//!
//! libjpeg's error handling is `setjmp`/`longjmp` based and its default
//! `error_exit` calls `exit()`, so [`JpegErrorManager`] is mandatory rather than
//! optional.

use std::os::raw::{c_char, c_int, c_uint, c_void};

/// `boolean` in `jmorecfg.h`.
pub type boolean = c_int;
/// One 8 bit sample.
pub type JSAMPLE = u8;
/// A row of samples.
pub type JSAMPROW = *mut JSAMPLE;
/// A list of row pointers.
pub type JSAMPARRAY = *mut JSAMPROW;
/// `size_t`.
pub type size_t = usize;

/// `JMSG_LENGTH_MAX` from `jerror.h`.
pub const JMSG_LENGTH_MAX: usize = 200;

/// The `version` argument `jpeg_CreateDecompress` checks, from `jconfig.h`.
pub const JPEG_LIB_VERSION: c_int = 80;

/// `sizeof(struct jpeg_decompress_struct)` on the ABI we bind against.
///
/// `jpeg_CreateDecompress` uses this to size the object it allocates for us.
/// The struct is 656 bytes on every 64 bit libjpeg 8 / libjpeg-turbo build; the
/// tests at the bottom of this file assert the field offsets that matter, which
/// is what a wrong value here would actually break.
pub const SIZEOF_JPEG_DECOMPRESS_STRUCT: size_t = 656;

/// `J_COLOR_SPACE` members we name.
pub const JCS_UNKNOWN: c_int = 0;
pub const JCS_GRAYSCALE: c_int = 1;
pub const JCS_RGB: c_int = 2;
pub const JCS_YCbCr: c_int = 3;
pub const JCS_CMYK: c_int = 4;
pub const JCS_YCCK: c_int = 5;

/// `jpeg_read_header` results.
pub const JPEG_HEADER_OK: c_int = 1;
pub const JPEG_HEADER_TABLES_ONLY: c_int = 2;

/// `glibc`'s `jmp_buf`, i.e. `struct __jmp_buf_tag`.
///
/// 200 bytes on glibc: 64 of jump buffer, 8 of saved-signal-mask flag and
/// padding, then a 128 byte `sigset_t`. Pinned by a test below.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JmpBuf(pub [u64; 25]);

unsafe extern "C" {
    /// `setjmp` without the signal mask save, paired with `_longjmp`.
    pub fn _setjmp(env: *mut c_void) -> c_int;
    /// Never returns: unwinds to the matching `_setjmp`.
    pub fn _longjmp(env: *mut c_void, value: c_int) -> !;
}

/// The fields of `struct jpeg_error_mgr` we read or write.
///
/// The message parameter union is never touched, so it is replaced by padding up
/// to the real 168 byte size: `jpeg_std_error` fills the whole struct and the
/// library may write to any of it.
#[repr(C)]
pub struct jpeg_error_mgr {
    pub error_exit: Option<unsafe extern "C" fn(*mut jpeg_common_struct)>,
    pub emit_message: Option<unsafe extern "C" fn(*mut jpeg_common_struct, c_int)>,
    pub output_message: Option<unsafe extern "C" fn(*mut jpeg_common_struct)>,
    pub format_message: Option<unsafe extern "C" fn(*mut jpeg_common_struct, *mut c_char)>,
    pub reset_error_mgr: Option<unsafe extern "C" fn(*mut jpeg_common_struct)>,
    pub msg_code: c_int,
    /// The `msg_parm_*` union and the trace limit, from offset 44 up. Private
    /// because it is padding, not something a caller reads.
    _parameters_and_trace: [u8; SIZEOF_JPEG_ERROR_MGR - 44],
}

/// `sizeof(struct jpeg_error_mgr)`.
pub const SIZEOF_JPEG_ERROR_MGR: usize = 168;

/// The fields of `struct jpeg_common_struct`, i.e. the `jpeg_common_fields`
/// macro both master structs start with.
#[repr(C)]
pub struct jpeg_common_struct {
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut c_void,
    pub progress: *mut c_void,
    pub client_data: *mut c_void,
    pub is_decompressor: boolean,
    pub global_state: c_int,
}

/// `struct jpeg_decompress_struct` up to and including `colormap`, padded out to
/// the real 656 bytes.
///
/// Unlike most libjpeg structs this one is allocated by the *caller*: the
/// canonical usage is a `struct jpeg_decompress_struct` on the stack, handed to
/// `jpeg_CreateDecompress` together with its `sizeof`. libjpeg then initialises
/// all 656 bytes, so the declaration has to be that big or the write runs off the
/// end. Only the fields above `colormap` are ever read, and their offsets are
/// what the tests below pin.
#[repr(C)]
pub struct jpeg_decompress_struct {
    // jpeg_common_fields
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut c_void,
    pub progress: *mut c_void,
    pub client_data: *mut c_void,
    pub is_decompressor: boolean,
    pub global_state: c_int,
    pub src: *mut c_void,
    pub image_width: c_uint,
    pub image_height: c_uint,
    pub num_components: c_int,
    pub jpeg_color_space: c_int,
    pub out_color_space: c_int,
    pub scale_num: c_uint,
    pub scale_denom: c_uint,
    pub output_gamma: f64,
    pub buffered_image: boolean,
    pub raw_data_out: boolean,
    pub dct_method: c_int,
    pub do_fancy_upsampling: boolean,
    pub do_block_smoothing: boolean,
    pub quantize_colors: boolean,
    pub dither_mode: c_int,
    pub two_pass_quantize: boolean,
    pub desired_number_of_colors: c_int,
    pub enable_1pass_quant: boolean,
    pub enable_external_quant: boolean,
    pub enable_2pass_quant: boolean,
    pub output_width: c_uint,
    pub output_height: c_uint,
    pub out_color_components: c_int,
    pub output_components: c_int,
    pub rec_outbuf_height: c_int,
    pub actual_number_of_colors: c_int,
    pub colormap: JSAMPARRAY,
    /// `output_scanline` onwards: never read, but libjpeg initialises it. Private
    /// because it is padding, not something a caller reads.
    _tail: [u8; SIZEOF_JPEG_DECOMPRESS_STRUCT - 168],
}

xs_platform_linux::declare_module! {
    LibJpeg,
    "libjpeg.so",
    "libjpeg.so.8",
    "libjpeg.so.62",
    "libturbojpeg.so",
    ...
    ...
    pub fn jpeg_std_error(*mut jpeg_error_mgr) -> *mut jpeg_error_mgr,

    pub fn jpeg_CreateDecompress(*mut jpeg_decompress_struct, c_int, size_t),
    pub fn jpeg_destroy_decompress(*mut jpeg_decompress_struct),
    pub fn jpeg_mem_src(*mut jpeg_decompress_struct, *const u8, size_t),
    pub fn jpeg_read_header(*mut jpeg_decompress_struct, c_int) -> c_int,
    pub fn jpeg_calc_output_dimensions(*mut jpeg_decompress_struct),
    pub fn jpeg_start_decompress(*mut jpeg_decompress_struct) -> boolean,
    pub fn jpeg_read_scanlines(*mut jpeg_decompress_struct, JSAMPARRAY, c_uint) -> c_uint,
    pub fn jpeg_finish_decompress(*mut jpeg_decompress_struct) -> boolean,
    ...
    ...
}

/// `jpeg_error_mgr` plus the jump target and message buffer our `error_exit`
/// needs.
///
/// Must start with the `jpeg_error_mgr` because `cinfo->err` points at it. Only
/// function pointers live in the first field, so the whole thing is `Send`
/// without an `unsafe impl`.
#[repr(C)]
pub struct JpegErrorManager {
    pub error_mgr: jpeg_error_mgr,
    pub jump_buffer: JmpBuf,
    /// Raw bytes, NUL terminated. Kept as bytes so reading it back needs no sign
    /// cast; cast once at the `format_message` call.
    pub message: [u8; JMSG_LENGTH_MAX],
}

impl JpegErrorManager {
    /// A manager with libjpeg's default handlers installed and ours layered on
    /// top.
    ///
    /// `output_message` is replaced with a no-op so that a corrupt file fetched
    /// from the network cannot write to the process stderr; the formatted
    /// message is kept in [`Self::message`] instead and surfaced as an
    /// [`ImageError`](xs_image_core::ImageError).
    #[must_use]
    pub fn new(jpeg: &LibJpeg) -> Self {
        let mut manager = Self {
            error_mgr: jpeg_error_mgr {
                error_exit: None,
                emit_message: None,
                output_message: None,
                format_message: None,
                reset_error_mgr: None,
                msg_code: 0,
                _parameters_and_trace: [0; SIZEOF_JPEG_ERROR_MGR - 44],
            },
            jump_buffer: JmpBuf([0; 25]),
            message: [0; JMSG_LENGTH_MAX],
        };
        unsafe {
            (jpeg.jpeg_std_error)(&raw mut manager.error_mgr);
            manager.error_mgr.error_exit = Some(error_exit);
            manager.error_mgr.output_message = Some(silence);
        }
        manager
    }

    /// Installs this manager into `cinfo`. Must run before
    /// `jpeg_CreateDecompress`, which writes the common fields.
    pub fn install(&mut self, cinfo: *mut jpeg_decompress_struct) {
        unsafe {
            (*cinfo).err = (&raw mut self.error_mgr).cast::<jpeg_error_mgr>();
        }
    }

    /// The jump target handed to `_setjmp`.
    pub fn jump_buffer(&mut self) -> *mut c_void {
        (&raw mut self.jump_buffer).cast::<c_void>()
    }

    /// The formatted message, trimmed of its terminating null.
    #[must_use]
    pub fn message(&self) -> String {
        let bytes: Vec<u8> = self
            .message
            .iter()
            .take_while(|byte| **byte != 0)
            .copied()
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

/// Replaces libjpeg's `exit()`-on-error handler with a jump back to the
/// `_setjmp` in [`JpegDecoder::decode`](crate::JpegDecoder::decode).
///
/// libjpeg frees the decompression object before it gets here, so the caller
/// must not call `jpeg_destroy_decompress` after a jump.
unsafe extern "C" fn error_exit(cinfo: *mut jpeg_common_struct) {
    let error_mgr = unsafe { (*cinfo).err };
    let manager = error_mgr.cast::<JpegErrorManager>();
    if let Some(format_message) = unsafe { (*error_mgr).format_message } {
        unsafe { format_message(cinfo, (*manager).message.as_mut_ptr().cast::<c_char>()) };
    }
    let jump_buffer = unsafe { (&raw mut (*manager).jump_buffer).cast::<c_void>() };
    unsafe { _longjmp(jump_buffer, 1) }
}

/// Swallows libjpeg's warning chatter, which would otherwise go to stderr for
/// every malformed file we try to decode.
unsafe extern "C" fn silence(_cinfo: *mut jpeg_common_struct) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::offset_of;

    // Every offset below was read out of a C program compiled against
    // libjpeg-turbo 3.1.3's jpeglib.h. A change in the vendored ABI should
    // break here rather than silently decode into the wrong samples.
    #[test]
    fn jpeg_common_struct_matches_the_header() {
        assert_eq!(offset_of!(jpeg_common_struct, err), 0);
        assert_eq!(offset_of!(jpeg_common_struct, mem), 8);
        assert_eq!(offset_of!(jpeg_common_struct, progress), 16);
        assert_eq!(offset_of!(jpeg_common_struct, client_data), 24);
        assert_eq!(offset_of!(jpeg_common_struct, is_decompressor), 32);
        assert_eq!(offset_of!(jpeg_common_struct, global_state), 36);
    }

    #[test]
    fn jpeg_error_mgr_matches_the_header() {
        assert_eq!(offset_of!(jpeg_error_mgr, error_exit), 0);
        assert_eq!(offset_of!(jpeg_error_mgr, emit_message), 8);
        assert_eq!(offset_of!(jpeg_error_mgr, output_message), 16);
        assert_eq!(offset_of!(jpeg_error_mgr, format_message), 24);
        assert_eq!(offset_of!(jpeg_error_mgr, reset_error_mgr), 32);
        assert_eq!(offset_of!(jpeg_error_mgr, msg_code), 40);
    }

    #[test]
    fn the_error_manager_starts_with_the_error_mgr() {
        let actual = offset_of!(JpegErrorManager, error_mgr);
        let expected = 0;
        assert_eq!(actual, expected);
    }

    #[test]
    fn jpeg_decompress_struct_matches_the_header() {
        assert_eq!(offset_of!(jpeg_decompress_struct, err), 0);
        assert_eq!(offset_of!(jpeg_decompress_struct, src), 40);
        assert_eq!(offset_of!(jpeg_decompress_struct, image_width), 48);
        assert_eq!(offset_of!(jpeg_decompress_struct, image_height), 52);
        assert_eq!(offset_of!(jpeg_decompress_struct, num_components), 56);
        assert_eq!(offset_of!(jpeg_decompress_struct, jpeg_color_space), 60);
        assert_eq!(offset_of!(jpeg_decompress_struct, out_color_space), 64);
        assert_eq!(offset_of!(jpeg_decompress_struct, scale_num), 68);
        assert_eq!(offset_of!(jpeg_decompress_struct, scale_denom), 72);
        assert_eq!(offset_of!(jpeg_decompress_struct, output_gamma), 80);
        assert_eq!(offset_of!(jpeg_decompress_struct, buffered_image), 88);
        assert_eq!(offset_of!(jpeg_decompress_struct, raw_data_out), 92);
        assert_eq!(offset_of!(jpeg_decompress_struct, dct_method), 96);
        assert_eq!(offset_of!(jpeg_decompress_struct, do_fancy_upsampling), 100);
        assert_eq!(offset_of!(jpeg_decompress_struct, do_block_smoothing), 104);
        assert_eq!(offset_of!(jpeg_decompress_struct, quantize_colors), 108);
        assert_eq!(offset_of!(jpeg_decompress_struct, output_width), 136);
        assert_eq!(offset_of!(jpeg_decompress_struct, output_height), 140);
        assert_eq!(
            offset_of!(jpeg_decompress_struct, out_color_components),
            144
        );
        assert_eq!(offset_of!(jpeg_decompress_struct, output_components), 148);
        assert_eq!(offset_of!(jpeg_decompress_struct, rec_outbuf_height), 152);
        assert_eq!(offset_of!(jpeg_decompress_struct, colormap), 160);
    }

    #[test]
    fn the_decompress_struct_is_exactly_as_big_as_the_library_ones() {
        // The caller allocates this one, so it has to match byte for byte or
        // jpeg_CreateDecompress writes past the end of it.
        let actual = size_of::<jpeg_decompress_struct>();
        let expected = SIZEOF_JPEG_DECOMPRESS_STRUCT;
        assert_eq!(actual, expected);
    }

    #[test]
    fn jpeg_error_mgr_is_the_size_the_header_reports() {
        let actual = size_of::<jpeg_error_mgr>();
        let expected = SIZEOF_JPEG_ERROR_MGR;
        assert_eq!(actual, expected);
    }

    #[test]
    fn jmp_buf_is_glibc_s_size() {
        let actual = size_of::<JmpBuf>();
        let expected = 200;
        assert_eq!(actual, expected);
    }

    #[test]
    fn the_message_stops_at_the_first_null() {
        let jpeg = LibJpeg::try_load().expect("libjpeg.so");
        let mut manager = JpegErrorManager::new(&jpeg);
        manager.message[0] = b'a';
        manager.message[1] = b'b';
        manager.message[2] = 0;
        let actual = manager.message();
        let expected = "ab";
        assert_eq!(actual, expected);
    }
}
