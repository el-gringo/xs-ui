//! Raw `libjxl` bindings.
//!
//! Loaded at runtime with `dlopen`, following `xs-egl-sys` and `xs-tls`.
//!
//! # Version requirement: 0.11
//!
//! The struct declarations below are 0.11 layouts. `JxlColorEncoding` alone
//! changed shape: in 0.9 and 0.10 `transfer_function` sat at offset 12 and
//! `have_gamma` followed it, while in 0.11 the `double` white point and
//! primaries arrays push it to offset 80. A 0.10 build fed a 0.11 struct would
//! produce plausible-looking garbage rather than fail, so
//! [`JxlDecoder::try_new`](crate::JpegXlDecoder::try_new) rejects anything older
//! by asking `JxlDecoderVersion` first.

#![allow(non_camel_case_types, non_snake_case, dead_code)]

use std::os::raw::{c_int, c_void};

/// `size_t`.
pub type size_t = usize;
/// `uint8_t`.
pub type u8_t = u8;
/// `int32_t`.
pub type i32_t = i32;
/// `uint32_t`.
pub type u32_t = u32;
/// `float`.
pub type f32_t = f32;
/// `double`.
pub type f64_t = f64;

/// `JXL_BOOL`, which `jxl/types.h` documents as being `int`.
pub type JxlBool = c_int;
/// `JXL_TRUE`.
pub const JXL_TRUE: JxlBool = 1;
/// `JXL_FALSE`.
pub const JXL_FALSE: JxlBool = 0;

/// `JxlDecoderStatus`.
pub const JXL_DEC_SUCCESS: c_int = 0;
pub const JXL_DEC_ERROR: c_int = 1;
pub const JXL_DEC_NEED_MORE_INPUT: c_int = 2;
pub const JXL_DEC_NEED_PREVIEW_OUT_BUFFER: c_int = 3;
pub const JXL_DEC_NEED_IMAGE_OUT_BUFFER: c_int = 5;
pub const JXL_DEC_BASIC_INFO: c_int = 0x40;
pub const JXL_DEC_COLOR_ENCODING: c_int = 0x100;
pub const JXL_DEC_FRAME: c_int = 0x400;
pub const JXL_DEC_FULL_IMAGE: c_int = 0x1000;

/// `JxlDataType`.
pub const JXL_TYPE_FLOAT: c_int = 0;
pub const JXL_TYPE_UINT8: c_int = 2;
pub const JXL_TYPE_UINT16: c_int = 3;

/// `JxlEndianness`.
pub const JXL_NATIVE_ENDIAN: c_int = 0;
pub const JXL_LITTLE_ENDIAN: c_int = 1;
pub const JXL_BIG_ENDIAN: c_int = 2;

/// `JxlColorSpace`.
pub const JXL_COLOR_SPACE_RGB: c_int = 0;
pub const JXL_COLOR_SPACE_GRAY: c_int = 1;
pub const JXL_COLOR_SPACE_XYB: c_int = 2;

/// `JxlWhitePoint`.
pub const JXL_WHITE_POINT_D65: c_int = 1;

/// `JxlPrimaries`.
pub const JXL_PRIMARIES_SRGB: c_int = 1;

/// `JxlTransferFunction`. `SRGB` is 13, not 1, as of the 0.11 renumbering.
pub const JXL_TRANSFER_FUNCTION_SRGB: c_int = 13;

/// `JxlRenderingIntent`.
pub const JXL_RENDERING_INTENT_RELATIVE: c_int = 1;

/// `JxlExtraChannelType`.
pub const JXL_CHANNEL_ALPHA: c_int = 0;
pub const JXL_CHANNEL_DEPTH: c_int = 15;

/// The oldest `JxlDecoderVersion` whose layouts [`JxlBasicInfo`] and
/// [`JxlColorEncoding`] match: 0.11.0 is `11 * 1000 + 0`.
pub const MINIMUM_VERSION: u32 = 11_000;

/// `JxlDecoder`, opaque.
#[repr(C)]
pub struct JxlDecoder {
    _private: [u8; 0],
}

/// `JxlBasicInfo`, 204 bytes.
///
/// Only `xsize`, `ysize`, `bits_per_sample` and `num_extra_channels` are read,
/// but `JxlDecoderGetBasicInfo` copies the whole struct out of the library, so
/// the size has to be right. `have_container` leading the struct is 0.11: it was
/// added in 0.10 and pushed `xsize` from offset 0 to offset 4.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JxlBasicInfo {
    pub have_container: JxlBool,
    pub xsize: u32_t,
    pub ysize: u32_t,
    pub bits_per_sample: u32_t,
    pub exponent_bits_per_sample: u32_t,
    pub intensity_target: f32_t,
    pub min_nits: f32_t,
    pub relative_to_max_display: JxlBool,
    pub linear_below: f32_t,
    pub uses_original_profile: JxlBool,
    pub have_preview: JxlBool,
    pub have_animation: JxlBool,
    pub orientation: c_int,
    pub num_color_channels: u32_t,
    pub num_extra_channels: u32_t,
    pub alpha_bits: u32_t,
    pub alpha_exponent_bits: u32_t,
    pub alpha_premultiplied: JxlBool,
    pub preview: [u32_t; 2],
    pub animation: [u32_t; 4],
    pub intrinsic_xsize: u32_t,
    pub intrinsic_ysize: u32_t,
    pub padding: [u8_t; 100],
}

/// `JxlPixelFormat`, 24 bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JxlPixelFormat {
    pub num_channels: u32_t,
    pub data_type: c_int,
    pub endianness: c_int,
    pub align: size_t,
}

/// `JxlColorEncoding`, 104 bytes.
///
/// The `double` arrays are why this struct is version sensitive: see the module
/// docs.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JxlColorEncoding {
    pub color_space: c_int,
    pub white_point: c_int,
    pub white_point_xy: [f64_t; 2],
    pub primaries: c_int,
    pub primaries_red_xy: [f64_t; 2],
    pub primaries_green_xy: [f64_t; 2],
    pub primaries_blue_xy: [f64_t; 2],
    pub transfer_function: c_int,
    pub gamma: f64_t,
    pub rendering_intent: c_int,
}

impl JxlColorEncoding {
    /// sRGB, which is what every consumer in this workspace wants.
    ///
    /// The numerical arrays stay zero because the enums above name the built-in
    /// values; libjxl only reads them for `*_CUSTOM`.
    #[must_use]
    pub const fn srgb() -> Self {
        Self {
            color_space: JXL_COLOR_SPACE_RGB,
            white_point: JXL_WHITE_POINT_D65,
            white_point_xy: [0.0; 2],
            primaries: JXL_PRIMARIES_SRGB,
            primaries_red_xy: [0.0; 2],
            primaries_green_xy: [0.0; 2],
            primaries_blue_xy: [0.0; 2],
            transfer_function: JXL_TRANSFER_FUNCTION_SRGB,
            gamma: 0.0,
            rendering_intent: JXL_RENDERING_INTENT_RELATIVE,
        }
    }
}

/// `JxlExtraChannelInfo`, 44 bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JxlExtraChannelInfo {
    pub kind: c_int,
    pub bits_per_sample: u32_t,
    pub exponent_bits_per_sample: u32_t,
    pub dim_shift: u32_t,
    pub name_length: u32_t,
    pub alpha_premultiplied: JxlBool,
    pub spot_color: [f32_t; 4],
    pub cfa_channel: u32_t,
}

xs_platform_linux::declare_module! {
    LibJxl,
    "libjxl.so",
    "libjxl.so.0.11",
    "libjxl.so.0.10",
    ...
    ...
    pub fn JxlDecoderVersion() -> u32_t,

    pub fn JxlDecoderCreate(*const c_void) -> *mut JxlDecoder,
    pub fn JxlDecoderDestroy(*mut JxlDecoder),
    pub fn JxlDecoderSubscribeEvents(*mut JxlDecoder, c_int) -> c_int,
    pub fn JxlDecoderSetKeepOrientation(*mut JxlDecoder, JxlBool) -> c_int,
    pub fn JxlDecoderSetUnpremultiplyAlpha(*mut JxlDecoder, JxlBool) -> c_int,
    pub fn JxlDecoderSetCoalescing(*mut JxlDecoder, JxlBool) -> c_int,
    pub fn JxlDecoderSetInput(*mut JxlDecoder, *const u8_t, size_t) -> c_int,
    pub fn JxlDecoderCloseInput(*mut JxlDecoder),
    pub fn JxlDecoderProcessInput(*mut JxlDecoder) -> c_int,
    pub fn JxlDecoderReleaseInput(*mut JxlDecoder) -> size_t,
    pub fn JxlDecoderGetBasicInfo(*const JxlDecoder, *mut JxlBasicInfo) -> c_int,
    pub fn JxlDecoderGetExtraChannelInfo(*const JxlDecoder, size_t, *mut JxlExtraChannelInfo) -> c_int,
    pub fn JxlDecoderSetOutputColorProfile(
        *mut JxlDecoder,
        *const JxlColorEncoding,
        *const u8_t,
        size_t,
    ) -> c_int,
    pub fn JxlDecoderImageOutBufferSize(
        *const JxlDecoder,
        *const JxlPixelFormat,
        *mut size_t,
    ) -> c_int,
    pub fn JxlDecoderSetImageOutBuffer(
        *mut JxlDecoder,
        *const JxlPixelFormat,
        *mut c_void,
        size_t,
    ) -> c_int,
    pub fn JxlDecoderSkipCurrentFrame(*mut JxlDecoder) -> c_int,
    ...
    ...
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    // Every offset below was read out of a C program compiled against libjxl
    // 0.11.2's headers. A wrong one here produces a plausible-looking image full
    // of the wrong colours, so these are worth pinning.
    #[test]
    fn jxl_basic_info_matches_the_header() {
        assert_eq!(size_of::<JxlBasicInfo>(), 204);
        assert_eq!(offset_of!(JxlBasicInfo, have_container), 0);
        assert_eq!(offset_of!(JxlBasicInfo, xsize), 4);
        assert_eq!(offset_of!(JxlBasicInfo, ysize), 8);
        assert_eq!(offset_of!(JxlBasicInfo, bits_per_sample), 12);
        assert_eq!(offset_of!(JxlBasicInfo, num_extra_channels), 56);
        assert_eq!(offset_of!(JxlBasicInfo, alpha_bits), 60);
    }

    #[test]
    fn jxl_pixel_format_matches_the_header() {
        assert_eq!(size_of::<JxlPixelFormat>(), 24);
        assert_eq!(align_of::<JxlPixelFormat>(), 8);
        assert_eq!(offset_of!(JxlPixelFormat, num_channels), 0);
        assert_eq!(offset_of!(JxlPixelFormat, data_type), 4);
        assert_eq!(offset_of!(JxlPixelFormat, endianness), 8);
        assert_eq!(offset_of!(JxlPixelFormat, align), 16);
    }

    #[test]
    fn jxl_color_encoding_matches_the_header() {
        assert_eq!(size_of::<JxlColorEncoding>(), 104);
        assert_eq!(align_of::<JxlColorEncoding>(), 8);
        assert_eq!(offset_of!(JxlColorEncoding, color_space), 0);
        assert_eq!(offset_of!(JxlColorEncoding, white_point), 4);
        assert_eq!(offset_of!(JxlColorEncoding, white_point_xy), 8);
        assert_eq!(offset_of!(JxlColorEncoding, primaries), 24);
        assert_eq!(offset_of!(JxlColorEncoding, primaries_red_xy), 32);
        assert_eq!(offset_of!(JxlColorEncoding, primaries_green_xy), 48);
        assert_eq!(offset_of!(JxlColorEncoding, primaries_blue_xy), 64);
        assert_eq!(offset_of!(JxlColorEncoding, transfer_function), 80);
        assert_eq!(offset_of!(JxlColorEncoding, gamma), 88);
        assert_eq!(offset_of!(JxlColorEncoding, rendering_intent), 96);
    }

    #[test]
    fn jxl_extra_channel_info_matches_the_header() {
        assert_eq!(size_of::<JxlExtraChannelInfo>(), 44);
    }

    #[test]
    fn jxl_bool_is_an_int() {
        let actual = size_of::<JxlBool>();
        let expected = size_of::<c_int>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn the_srgb_encoding_names_built_in_values() {
        let srgb = JxlColorEncoding::srgb();
        assert_eq!(srgb.color_space, JXL_COLOR_SPACE_RGB);
        assert_eq!(srgb.white_point, JXL_WHITE_POINT_D65);
        assert_eq!(srgb.primaries, JXL_PRIMARIES_SRGB);
        assert_eq!(srgb.transfer_function, JXL_TRANSFER_FUNCTION_SRGB);
        assert_eq!(srgb.rendering_intent, JXL_RENDERING_INTENT_RELATIVE);
    }

    #[test]
    fn the_srgb_encoding_leaves_the_custom_arrays_at_zero() {
        // libjxl only reads these for the *_CUSTOM enums.
        let srgb = JxlColorEncoding::srgb();
        assert_eq!(srgb.white_point_xy, [0.0; 2]);
        assert_eq!(srgb.primaries_red_xy, [0.0; 2]);
        assert_eq!(srgb.gamma, 0.0);
    }

    #[test]
    fn the_version_gate_is_0_11() {
        let actual = MINIMUM_VERSION;
        let expected = 11_000;
        assert_eq!(actual, expected);
    }
}
