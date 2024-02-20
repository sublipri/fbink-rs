use fbink_sys as raw;
use fbink_sys::*;
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FbInkConfig {
    pub row: i16,
    pub col: i16,
    pub fontmult: u8,
    pub font: Font,
    pub is_inverted: bool,
    pub is_flashing: bool,
    pub is_cleared: bool,
    pub is_centered: bool,
    pub hoffset: i16,
    pub voffset: i16,
    pub is_halfway: bool,
    pub is_padded: bool,
    pub is_rpadded: bool,
    pub fg_color: ForegroundColor,
    pub bg_color: BackgroundColor,
    pub is_overlay: bool,
    pub is_bgless: bool,
    pub is_fgless: bool,
    pub no_viewport: bool,
    pub is_verbose: bool,
    pub is_quiet: bool,
    pub ignore_alpha: bool,
    pub halign: Alignment,
    pub valign: Alignment,
    pub scaled_width: i16,
    pub scaled_height: i16,
    pub wfm_mode: WaveformMode,
    pub dithering_mode: HardwareDitherMode,
    pub sw_dithering: bool,
    pub is_nightmode: bool,
    pub no_refresh: bool,
    pub no_merge: bool,
    pub is_animated: bool,
    pub to_syslog: bool,
}

impl From<FbInkConfig> for raw::FBInkConfig {
    fn from(c: FbInkConfig) -> Self {
        Self {
            row: c.row,
            col: c.col,
            fontmult: c.fontmult,
            fontname: c.font.into(),
            is_inverted: c.is_inverted,
            is_flashing: c.is_flashing,
            is_cleared: c.is_cleared,
            is_centered: c.is_centered,
            hoffset: c.hoffset,
            voffset: c.voffset,
            is_halfway: c.is_halfway,
            is_padded: c.is_padded,
            is_rpadded: c.is_rpadded,
            fg_color: c.fg_color.into(),
            bg_color: c.bg_color.into(),
            is_overlay: c.is_overlay,
            is_bgless: c.is_bgless,
            is_fgless: c.is_fgless,
            no_viewport: c.no_viewport,
            is_verbose: c.is_verbose,
            is_quiet: c.is_quiet,
            ignore_alpha: c.ignore_alpha,
            halign: c.halign.into(),
            valign: c.valign.into(),
            scaled_width: c.scaled_width,
            scaled_height: c.scaled_height,
            wfm_mode: c.wfm_mode.into(),
            dithering_mode: c.dithering_mode.into(),
            sw_dithering: c.sw_dithering,
            is_nightmode: c.is_nightmode,
            no_refresh: c.no_refresh,
            no_merge: c.no_merge,
            is_animated: c.is_animated,
            to_syslog: c.to_syslog,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Font {
    #[default]
    Ibm = FONT_INDEX_E_IBM,
    Unscii = FONT_INDEX_E_UNSCII,
    UnsciiAlt = FONT_INDEX_E_UNSCII_ALT,
    UnsciiThin = FONT_INDEX_E_UNSCII_THIN,
    UnsciiFantasy = FONT_INDEX_E_UNSCII_FANTASY,
    UnsciiMcr = FONT_INDEX_E_UNSCII_MCR,
    UnsciiTall = FONT_INDEX_E_UNSCII_TALL,
    Block = FONT_INDEX_E_BLOCK,
    Leggie = FONT_INDEX_E_LEGGIE,
    Veggie = FONT_INDEX_E_VEGGIE,
    Kates = FONT_INDEX_E_KATES,
    Fkp = FONT_INDEX_E_FKP,
    Ctrld = FONT_INDEX_E_CTRLD,
    Orp = FONT_INDEX_E_ORP,
    OrpBold = FONT_INDEX_E_ORPB,
    OrpItalic = FONT_INDEX_E_ORPI,
    Scientifica = FONT_INDEX_E_SCIENTIFICA,
    ScientificaBold = FONT_INDEX_E_SCIENTIFICAB,
    ScientificaItalic = FONT_INDEX_E_SCIENTIFICAI,
    Terminus = FONT_INDEX_E_TERMINUS,
    TerminusBold = FONT_INDEX_E_TERMINUSB,
    Fatty = FONT_INDEX_E_FATTY,
    Spleen = FONT_INDEX_E_SPLEEN,
    Tewi = FONT_INDEX_E_TEWI,
    TewiBold = FONT_INDEX_E_TEWIB,
    Topaz = FONT_INDEX_E_TOPAZ,
    Microknight = FONT_INDEX_E_MICROKNIGHT,
    Vga = FONT_INDEX_E_VGA,
    Unifont = FONT_INDEX_E_UNIFONT,
    UnifontDouble = FONT_INDEX_E_UNIFONTDW,
    Cozette = FONT_INDEX_E_COZETTE,
    Max = FONT_INDEX_E_FONT_MAX,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum FontStyle {
    #[default]
    Regular = FONT_STYLE_E_FNT_REGULAR,
    Italic = FONT_STYLE_E_FNT_ITALIC,
    Bold = FONT_STYLE_E_FNT_BOLD,
    BoldItalic = FONT_STYLE_E_FNT_BOLD_ITALIC,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Alignment {
    #[default]
    None = ALIGN_INDEX_E_NONE,
    Center = ALIGN_INDEX_E_CENTER,
    Edge = ALIGN_INDEX_E_EDGE,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum PaddingIndex {
    #[default]
    NoPadding = PADDING_INDEX_E_NO_PADDING,
    HorizontalPadding = PADDING_INDEX_E_HORI_PADDING,
    VerticalPadding = PADDING_INDEX_E_VERT_PADDING,
    FullPadding = PADDING_INDEX_E_FULL_PADDING,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum ForegroundColor {
    #[default]
    Black = FG_COLOR_INDEX_E_FG_BLACK,
    Gray1 = FG_COLOR_INDEX_E_FG_GRAY1,
    Gray2 = FG_COLOR_INDEX_E_FG_GRAY2,
    Gray3 = FG_COLOR_INDEX_E_FG_GRAY3,
    Gray4 = FG_COLOR_INDEX_E_FG_GRAY4,
    Gray5 = FG_COLOR_INDEX_E_FG_GRAY5,
    Gray6 = FG_COLOR_INDEX_E_FG_GRAY6,
    Gray7 = FG_COLOR_INDEX_E_FG_GRAY7,
    Gray8 = FG_COLOR_INDEX_E_FG_GRAY8,
    Gray9 = FG_COLOR_INDEX_E_FG_GRAY9,
    GrayA = FG_COLOR_INDEX_E_FG_GRAYA,
    GrayB = FG_COLOR_INDEX_E_FG_GRAYB,
    GrayC = FG_COLOR_INDEX_E_FG_GRAYC,
    GrayD = FG_COLOR_INDEX_E_FG_GRAYD,
    GrayE = FG_COLOR_INDEX_E_FG_GRAYE,
    White = FG_COLOR_INDEX_E_FG_WHITE,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum BackgroundColor {
    #[default]
    White = BG_COLOR_INDEX_E_BG_WHITE,
    GrayE = BG_COLOR_INDEX_E_BG_GRAYE,
    GrayD = BG_COLOR_INDEX_E_BG_GRAYD,
    GrayC = BG_COLOR_INDEX_E_BG_GRAYC,
    GrayB = BG_COLOR_INDEX_E_BG_GRAYB,
    GrayA = BG_COLOR_INDEX_E_BG_GRAYA,
    Gray9 = BG_COLOR_INDEX_E_BG_GRAY9,
    Gray8 = BG_COLOR_INDEX_E_BG_GRAY8,
    Gray7 = BG_COLOR_INDEX_E_BG_GRAY7,
    Gray6 = BG_COLOR_INDEX_E_BG_GRAY6,
    Gray5 = BG_COLOR_INDEX_E_BG_GRAY5,
    Gray4 = BG_COLOR_INDEX_E_BG_GRAY4,
    Gray3 = BG_COLOR_INDEX_E_BG_GRAY3,
    Gray2 = BG_COLOR_INDEX_E_BG_GRAY2,
    Gray1 = BG_COLOR_INDEX_E_BG_GRAY1,
    Black = BG_COLOR_INDEX_E_BG_BLACK,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum WaveformMode {
    #[default]
    Auto = WFM_MODE_INDEX_E_WFM_AUTO,
    Dual = WFM_MODE_INDEX_E_WFM_DU,
    GC16 = WFM_MODE_INDEX_E_WFM_GC16,
    GC4 = WFM_MODE_INDEX_E_WFM_GC4,
    A2 = WFM_MODE_INDEX_E_WFM_A2,
    GL16 = WFM_MODE_INDEX_E_WFM_GL16,
    Reagl = WFM_MODE_INDEX_E_WFM_REAGL,
    Reagld = WFM_MODE_INDEX_E_WFM_REAGLD,
    GC16Fast = WFM_MODE_INDEX_E_WFM_GC16_FAST,
    GL16Fast = WFM_MODE_INDEX_E_WFM_GL16_FAST,
    DU4 = WFM_MODE_INDEX_E_WFM_DU4,
    GL4 = WFM_MODE_INDEX_E_WFM_GL4,
    GL16Inv = WFM_MODE_INDEX_E_WFM_GL16_INV,
    GCK16 = WFM_MODE_INDEX_E_WFM_GCK16,
    GLKW16 = WFM_MODE_INDEX_E_WFM_GLKW16,
    Init = WFM_MODE_INDEX_E_WFM_INIT,
    Unknown = WFM_MODE_INDEX_E_WFM_UNKNOWN,
    Init2 = WFM_MODE_INDEX_E_WFM_INIT2,
    A2In = WFM_MODE_INDEX_E_WFM_A2IN,
    A2Out = WFM_MODE_INDEX_E_WFM_A2OUT,
    GC16HQ = WFM_MODE_INDEX_E_WFM_GC16HQ,
    GS16 = WFM_MODE_INDEX_E_WFM_GS16,
    GU16 = WFM_MODE_INDEX_E_WFM_GU16,
    GLK16 = WFM_MODE_INDEX_E_WFM_GLK16,
    Clear = WFM_MODE_INDEX_E_WFM_CLEAR,
    GC4L = WFM_MODE_INDEX_E_WFM_GC4L,
    GCC16 = WFM_MODE_INDEX_E_WFM_GCC16,
    GC16Partial = WFM_MODE_INDEX_E_WFM_GC16_PARTIAL,
    GCK16Partial = WFM_MODE_INDEX_E_WFM_GCK16_PARTIAL,
    Dunm = WFM_MODE_INDEX_E_WFM_DUNM,
    P2Sw = WFM_MODE_INDEX_E_WFM_P2SW,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum HardwareDitherMode {
    #[default]
    Passthrough = HW_DITHER_INDEX_E_HWD_PASSTHROUGH,
    FloydSteinberg = HW_DITHER_INDEX_E_HWD_FLOYD_STEINBERG,
    Atkinson = HW_DITHER_INDEX_E_HWD_ATKINSON,
    Ordered = HW_DITHER_INDEX_E_HWD_ORDERED,
    QuantOnly = HW_DITHER_INDEX_E_HWD_QUANT_ONLY,
    Legacy = HW_DITHER_INDEX_E_HWD_LEGACY,
}
