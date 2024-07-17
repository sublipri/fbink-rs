use crate::thin::fbink_rota_native_to_canonical;

use std::ffi::CStr;

use fbink_sys as raw;
use fbink_sys::*;
use num_enum::{FromPrimitive, IntoPrimitive};
use strum::{AsRefStr, Display};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FbInkState {
    pub user_hz: ::std::os::raw::c_long,
    pub font_name: String,
    pub view_width: u32,
    pub view_height: u32,
    pub screen_width: u32,
    pub screen_height: u32,
    pub scanline_stride: u32,
    pub bpp: u32,
    pub inverted_grayscale: bool,
    pub device_name: String,
    pub device_codename: String,
    pub device_platform: String,
    pub device_id: DeviceId,
    pub pen_fg_color: u8,
    pub pen_bg_color: u8,
    pub screen_dpi: u16,
    pub font_w: u16,
    pub font_h: u16,
    pub max_cols: u16,
    pub max_rows: u16,
    pub view_hori_origin: u8,
    pub view_vert_origin: u8,
    pub view_vert_offset: u8,
    pub fontsize_mult: u8,
    pub glyph_width: u8,
    pub glyph_height: u8,
    pub is_perfect_fit: bool,
    pub is_mtk: bool,
    pub is_sunxi: bool,
    pub sunxi_has_fbdamage: bool,
    pub sunxi_force_rota: SunxiForceRotation,
    pub is_kindle_legacy: bool,
    pub is_kobo_non_mt: bool,
    pub unreliable_wait_for: bool,
    pub can_wake_epdc: bool,
    pub ntx_boot_rota: u8,
    pub ntx_rota_quirk: NtxRotationQuirk,
    pub rotation_map: [u8; 4usize],
    pub touch_swap_axes: bool,
    pub touch_mirror_x: bool,
    pub touch_mirror_y: bool,
    pub is_ntx_quirky_landscape: bool,
    pub current_rota: u8,
    pub can_rotate: bool,
    pub can_hw_invert: bool,
    pub has_eclipse_wfm: bool,
    pub has_color_panel: bool,
    pub pixel_format: PixelFormat,
    pub can_wait_for_submission: bool,
}

fn get_string(ptr: *const ::std::os::raw::c_char) -> String {
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

impl FbInkState {
    pub fn device_details(&self) -> String {
        let name = match self.device_id {
            DeviceId::Unknown(_) => &self.device_name,
            _ => self.device_id.as_ref(),
        };
        format!(
            "{} ({} => {} @ {})",
            name,
            u16::from(self.device_id),
            self.device_platform,
            self.device_codename
        )
    }
    pub fn from_raw(s: raw::FBInkState) -> Self {
        Self {
            user_hz: s.user_hz,
            font_name: get_string(s.font_name),
            view_width: s.view_width,
            view_height: s.view_height,
            screen_width: s.screen_width,
            screen_height: s.screen_height,
            scanline_stride: s.scanline_stride,
            can_wake_epdc: s.can_wake_epdc,
            bpp: s.bpp,
            inverted_grayscale: s.inverted_grayscale,
            device_name: get_string(s.device_name.as_ptr()),
            device_codename: get_string(s.device_codename.as_ptr()),
            device_platform: get_string(s.device_platform.as_ptr()),
            device_id: s.device_id.into(),
            pen_fg_color: s.pen_fg_color,
            pen_bg_color: s.pen_bg_color,
            screen_dpi: s.screen_dpi,
            font_w: s.font_w,
            font_h: s.font_h,
            max_cols: s.max_cols,
            max_rows: s.max_rows,
            view_hori_origin: s.view_hori_origin,
            view_vert_origin: s.view_vert_origin,
            view_vert_offset: s.view_vert_offset,
            fontsize_mult: s.fontsize_mult,
            glyph_width: s.glyph_width,
            glyph_height: s.glyph_height,
            is_perfect_fit: s.is_perfect_fit,
            is_mtk: s.is_mtk,
            is_sunxi: s.is_sunxi,
            sunxi_has_fbdamage: s.sunxi_has_fbdamage,
            sunxi_force_rota: s.sunxi_force_rota.into(),
            is_kindle_legacy: s.is_kindle_legacy,
            is_kobo_non_mt: s.is_kobo_non_mt,
            unreliable_wait_for: s.unreliable_wait_for,
            ntx_boot_rota: s.ntx_boot_rota,
            ntx_rota_quirk: s.ntx_rota_quirk.into(),
            rotation_map: s.rotation_map,
            touch_swap_axes: s.touch_swap_axes,
            touch_mirror_x: s.touch_mirror_x,
            touch_mirror_y: s.touch_mirror_y,
            is_ntx_quirky_landscape: s.is_ntx_quirky_landscape,
            current_rota: s.current_rota,
            can_rotate: s.can_rotate,
            can_hw_invert: s.can_hw_invert,
            has_eclipse_wfm: s.has_eclipse_wfm,
            has_color_panel: s.has_color_panel,
            pixel_format: s.pixel_format.into(),
            can_wait_for_submission: s.can_wait_for_submission,
        }
    }

    /// The current canonical rotation of the device. For non-Kobo devices, this
    /// assumes that the framebuffer's reported rotation value is canonical.
    pub fn canonical_rotation(&self) -> CanonicalRotation {
        match fbink_rota_native_to_canonical(self.current_rota.into()) {
            Ok(rotation) => CanonicalRotation::from_primitive(rotation),
            Err(_) => CanonicalRotation::from_primitive(self.current_rota),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum CanonicalRotation {
    #[default]
    Upright = 0,
    Clockwise = 1,
    UpsideDown = 2,
    CounterClockwise = 3,
}

impl std::fmt::Display for CanonicalRotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Upright => f.write_str("upright"),
            Self::Clockwise => f.write_str("clockwise"),
            Self::UpsideDown => f.write_str("upside down"),
            Self::CounterClockwise => f.write_str("counterclockwise"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum NtxRotationQuirk {
    #[default]
    Straight = NTX_ROTA_INDEX_E_NTX_ROTA_STRAIGHT,
    AllInverted = NTX_ROTA_INDEX_E_NTX_ROTA_ALL_INVERTED,
    OddInverted = NTX_ROTA_INDEX_E_NTX_ROTA_ODD_INVERTED,
    Sane = NTX_ROTA_INDEX_E_NTX_ROTA_SANE,
    Sunxi = NTX_ROTA_INDEX_E_NTX_ROTA_SUNXI,
    CwTouch = NTX_ROTA_INDEX_E_NTX_ROTA_CW_TOUCH,
    CcwTouch = NTX_ROTA_INDEX_E_NTX_ROTA_CCW_TOUCH,
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(i8)]
pub enum SunxiForceRotation {
    #[default]
    NotSupp = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_NOTSUP,
    CurrentRota = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_CURRENT_ROTA,
    CurrentLayout = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_CURRENT_LAYOUT,
    Portrait = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_PORTRAIT,
    Landscape = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_LANDSCAPE,
    Gyro = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_GYRO,
    Ur = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_UR,
    Cw = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_CW,
    Ud = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_UD,
    Ccw = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_CCW,
    Workbuf = SUNXI_FORCE_ROTA_INDEX_E_FORCE_ROTA_WORKBUF,
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum MtkSwipeDirection {
    Down = MTK_SWIPE_DIRECTION_INDEX_E_MTK_SWIPE_DIR_DOWN,
    Up = MTK_SWIPE_DIRECTION_INDEX_E_MTK_SWIPE_DIR_UP,
    #[default]
    Left = MTK_SWIPE_DIRECTION_INDEX_E_MTK_SWIPE_DIR_LEFT,
    Right = MTK_SWIPE_DIRECTION_INDEX_E_MTK_SWIPE_DIR_RIGHT,
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum MtkHalftoneMode {
    #[default]
    Disabled = MTK_HALFTONE_MODE_INDEX_E_MTK_HALFTONE_DISABLED,
    DefaultCheckerSize = MTK_HALFTONE_MODE_INDEX_E_MTK_HALFTONE_DEFAULT_CHECKER_SIZE,
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum PixelFormat {
    #[default]
    Unknown = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_UNKNOWN,
    Y4 = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_Y4,
    Y8 = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_Y8,
    Bgr565 = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_BGR565,
    Rgb565 = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_RGB565,
    Bgr24 = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_BGR24,
    Rgb24 = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_RGB24,
    Bgra = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_BGRA,
    Rgba = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_RGBA,
    Bgr32 = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_BGR32,
    Rgb32 = FBINK_PXFMT_INDEX_E_FBINK_PXFMT_RGB32,
}

#[derive(Debug, Display, AsRefStr, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u16)]
#[strum(serialize_all = "title_case")]
pub enum DeviceId {
    // Cervantes
    CervantesTouch = CERVANTES_DEVICE_ID_E_DEVICE_CERVANTES_TOUCH,
    CervantesTouchlight = CERVANTES_DEVICE_ID_E_DEVICE_CERVANTES_TOUCHLIGHT,
    Cervantes2013 = CERVANTES_DEVICE_ID_E_DEVICE_CERVANTES_2013,
    Cervantes3 = CERVANTES_DEVICE_ID_E_DEVICE_CERVANTES_3,
    Cervantes4 = CERVANTES_DEVICE_ID_E_DEVICE_CERVANTES_4,
    // Kobo
    KoboTouchA = KOBO_DEVICE_ID_E_DEVICE_KOBO_TOUCH_A,
    KoboTouchB = KOBO_DEVICE_ID_E_DEVICE_KOBO_TOUCH_B,
    KoboTouchC = KOBO_DEVICE_ID_E_DEVICE_KOBO_TOUCH_C,
    KoboMini = KOBO_DEVICE_ID_E_DEVICE_KOBO_MINI,
    KoboGlo = KOBO_DEVICE_ID_E_DEVICE_KOBO_GLO,
    KoboGloHd = KOBO_DEVICE_ID_E_DEVICE_KOBO_GLO_HD,
    TolinoShine2Hd = KOBO_DEVICE_ID_E_DEVICE_TOLINO_SHINE_2HD,
    KoboTouch2 = KOBO_DEVICE_ID_E_DEVICE_KOBO_TOUCH_2,
    KoboAura = KOBO_DEVICE_ID_E_DEVICE_KOBO_AURA,
    KoboAuraHd = KOBO_DEVICE_ID_E_DEVICE_KOBO_AURA_HD,
    KoboAuraH2o = KOBO_DEVICE_ID_E_DEVICE_KOBO_AURA_H2O,
    KoboAuraH2o2 = KOBO_DEVICE_ID_E_DEVICE_KOBO_AURA_H2O_2,
    KoboAuraH2o2R2 = KOBO_DEVICE_ID_E_DEVICE_KOBO_AURA_H2O_2_R2,
    KoboAuraOne = KOBO_DEVICE_ID_E_DEVICE_KOBO_AURA_ONE,
    KoboAuraOneLe = KOBO_DEVICE_ID_E_DEVICE_KOBO_AURA_ONE_LE,
    KoboAuraSe = KOBO_DEVICE_ID_E_DEVICE_KOBO_AURA_SE,
    TolinoVision = KOBO_DEVICE_ID_E_DEVICE_TOLINO_VISION,
    KoboAuraSeR2 = KOBO_DEVICE_ID_E_DEVICE_KOBO_AURA_SE_R2,
    KoboClaraHd = KOBO_DEVICE_ID_E_DEVICE_KOBO_CLARA_HD,
    TolinoShine3 = KOBO_DEVICE_ID_E_DEVICE_TOLINO_SHINE_3,
    KoboForma = KOBO_DEVICE_ID_E_DEVICE_KOBO_FORMA,
    TolinoEpos2 = KOBO_DEVICE_ID_E_DEVICE_TOLINO_EPOS_2,
    KoboForma32 = KOBO_DEVICE_ID_E_DEVICE_KOBO_FORMA_32GB,
    KoboLibraH2o = KOBO_DEVICE_ID_E_DEVICE_KOBO_LIBRA_H2O,
    TolinoVision5 = KOBO_DEVICE_ID_E_DEVICE_TOLINO_VISION_5,
    KoboNia = KOBO_DEVICE_ID_E_DEVICE_KOBO_NIA,
    KoboElipsa = KOBO_DEVICE_ID_E_DEVICE_KOBO_ELIPSA,
    KoboLibra2 = KOBO_DEVICE_ID_E_DEVICE_KOBO_LIBRA_2,
    KoboSage = KOBO_DEVICE_ID_E_DEVICE_KOBO_SAGE,
    TolinoEpos3 = KOBO_DEVICE_ID_E_DEVICE_TOLINO_EPOS_3,
    KoboClara2e = KOBO_DEVICE_ID_E_DEVICE_KOBO_CLARA_2E,
    KoboElipsa2e = KOBO_DEVICE_ID_E_DEVICE_KOBO_ELIPSA_2E,
    KoboLibraColour = KOBO_DEVICE_ID_E_DEVICE_KOBO_LIBRA_COLOUR,
    TolinoVisionColor = KOBO_DEVICE_ID_E_DEVICE_TOLINO_VISION_COLOR,
    KoboClaraBw = KOBO_DEVICE_ID_E_DEVICE_KOBO_CLARA_BW,
    TolinoShineBw = KOBO_DEVICE_ID_E_DEVICE_TOLINO_SHINE_BW,
    KoboClaraColour = KOBO_DEVICE_ID_E_DEVICE_KOBO_CLARA_COLOUR,
    TolinoShineColor = KOBO_DEVICE_ID_E_DEVICE_TOLINO_SHINE_COLOR,
    // Mainline
    MainlineTolinoShine2hd = MAINLINE_DEVICE_ID_E_DEVICE_MAINLINE_TOLINO_SHINE_2HD,
    MainlineTolinoShine3 = MAINLINE_DEVICE_ID_E_DEVICE_MAINLINE_TOLINO_SHINE_3,
    MainlineTolinoVision = MAINLINE_DEVICE_ID_E_DEVICE_MAINLINE_TOLINO_VISION,
    MainlineTolinoVision5 = MAINLINE_DEVICE_ID_E_DEVICE_MAINLINE_TOLINO_VISION_5,
    GenericImx5 = MAINLINE_DEVICE_ID_E_DEVICE_MAINLINE_GENERIC_IMX5,
    GenericImx6 = MAINLINE_DEVICE_ID_E_DEVICE_MAINLINE_GENERIC_IMX6,
    GenericSunxiB300 = MAINLINE_DEVICE_ID_E_DEVICE_MAINLINE_GENERIC_SUNXI_B300,
    // Remarkable
    Remarkable1 = REMARKABLE_DEVICE_ID_E_DEVICE_REMARKABLE_1,
    Remarkable2 = REMARKABLE_DEVICE_ID_E_DEVICE_REMARKABLE_2,
    // Pocketbook
    PocketbookMini = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_MINI,
    Pocketbook606 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_606,
    Pocketbook611 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_611,
    Pocketbook613 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_613,
    Pocketbook614 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_614,
    Pocketbook615 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_615,
    Pocketbook616 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_616,
    Pocketbook617 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_617,
    Pocketbook618 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_618,
    PocketbookTouch = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_TOUCH,
    PocketbookLux = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_LUX,
    PocketbookBasicTouch = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_BASIC_TOUCH,
    PocketbookBasicTouch2 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_BASIC_TOUCH_2,
    PocketbookLux3 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_LUX_3,
    PocketbookLux4 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_LUX_4,
    PocketbookLux5 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_LUX_5,
    PocketbookVerse = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_VERSE,
    PocketbookSense = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_SENSE,
    PocketbookTouchHd = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_TOUCH_HD,
    PocketbookTouchHdPlus = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_TOUCH_HD_PLUS,
    PocketbookColor = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_COLOR,
    PocketbookVersePro = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_VERSE_PRO,
    PocketbookAqua = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_AQUA,
    PocketbookAqua2 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_AQUA2,
    PocketbookUltra = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_ULTRA,
    PocketbookEra = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_ERA,
    PocketbookEraColor = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_ERA_COLOR,
    PocketbookInkpad3 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_INKPAD_3,
    PocketbookInkpad3Pro = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_INKPAD_3_PRO,
    PocketbookInkpadColor = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_INKPAD_COLOR,
    PocketbookInkpadColor2 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_INKPAD_COLOR_2,
    PocketbookInkpadColor3 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_INKPAD_COLOR_3,
    PocketbookInkpad = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_INKPAD,
    PocketbookInkpadX = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_INKPAD_X,
    // clashes with INKPAD_COLOR_2
    // PocketbookInkpad4 = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_INKPAD_4,
    PocketbookColorLux = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_COLOR_LUX,
    PocketbookInkpadLite = POCKETBOOK_DEVICE_ID_E_DEVICE_POCKETBOOK_INKPAD_LITE,
    #[num_enum(catch_all)]
    Unknown(u16),
}
