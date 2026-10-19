//! Embedded Spleen PSF2 fonts (BSD licensed, see assets/fonts/SPLEEN-LICENSE).

use nr_gfx::PsfFont;

static SMALL: &[u8] = include_bytes!("../../../assets/fonts/spleen-8x16.psfu");
static BODY: &[u8] = include_bytes!("../../../assets/fonts/spleen-12x24.psfu");
static HEAD: &[u8] = include_bytes!("../../../assets/fonts/spleen-16x32.psfu");
static BIG: &[u8] = include_bytes!("../../../assets/fonts/spleen-32x64.psfu");

#[cfg(nightrun_japanese_font)]
static JAPANESE_16: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/noto-jp-16.psf"));
#[cfg(nightrun_japanese_font)]
static JAPANESE_24: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/noto-jp-24.psf"));
#[cfg(nightrun_japanese_font)]
static JAPANESE_32: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/noto-jp-32.psf"));

#[cfg(nightrun_japanese_font)]
pub struct JapaneseFonts {
    pub small: PsfFont<'static>,
    pub body: PsfFont<'static>,
    pub head: PsfFont<'static>,
}

pub struct Fonts {
    pub small: PsfFont<'static>,
    pub body: PsfFont<'static>,
    pub head: PsfFont<'static>,
    pub big: PsfFont<'static>,
    #[cfg(nightrun_japanese_font)]
    pub japanese: JapaneseFonts,
}

impl Fonts {
    pub fn load() -> Fonts {
        Fonts {
            small: PsfFont::parse(SMALL).expect("spleen 8x16"),
            body: PsfFont::parse(BODY).expect("spleen 12x24"),
            head: PsfFont::parse(HEAD).expect("spleen 16x32"),
            big: PsfFont::parse(BIG).expect("spleen 32x64"),
            #[cfg(nightrun_japanese_font)]
            japanese: JapaneseFonts {
                small: PsfFont::parse(JAPANESE_16).expect("Noto Sans JP 16px"),
                body: PsfFont::parse(JAPANESE_24).expect("Noto Sans JP 24px"),
                head: PsfFont::parse(JAPANESE_32).expect("Noto Sans JP 32px"),
            },
        }
    }

    pub fn japanese_small(&self) -> Option<&PsfFont<'static>> {
        #[cfg(nightrun_japanese_font)]
        {
            return Some(&self.japanese.small);
        }
        #[cfg(not(nightrun_japanese_font))]
        {
            None
        }
    }

    pub fn japanese_body(&self) -> Option<&PsfFont<'static>> {
        #[cfg(nightrun_japanese_font)]
        {
            return Some(&self.japanese.body);
        }
        #[cfg(not(nightrun_japanese_font))]
        {
            None
        }
    }

    pub fn japanese_head(&self) -> Option<&PsfFont<'static>> {
        #[cfg(nightrun_japanese_font)]
        {
            return Some(&self.japanese.head);
        }
        #[cfg(not(nightrun_japanese_font))]
        {
            None
        }
    }
}
