//! Glyph coverage of the embedded Spleen fonts: everything the chat is
//! expected to render must have a glyph; emoji must map to None (the
//! renderer skips them) rather than panicking or falling back to '?'.

use nr_ui::Fonts;

#[test]
fn fonts_cover_expected_ranges() {
    let fonts = Fonts::load();
    for (name, font) in [
        ("small", &fonts.small),
        ("body", &fonts.body),
        ("head", &fonts.head),
        ("big", &fonts.big),
    ] {
        // ASCII.
        for ch in ' '..='~' {
            assert!(font.glyph(ch).is_some(), "{name}: missing ASCII {ch:?}");
        }
        // Accented Latin + Polish (Latin-1 / Latin Extended-A).
        for ch in "éàüößñçżółćęśźńĄŻ".chars() {
            assert!(font.glyph(ch).is_some(), "{name}: missing {ch:?}");
        }
        // Typographic punctuation the models like to emit ('—' has no
        // glyph but the renderer substitutes '-'; see nr_gfx::draw).
        for ch in "–“”‘’…«»".chars() {
            assert!(font.glyph(ch).is_some(), "{name}: missing {ch:?}");
        }
        assert_eq!(nr_gfx::draw::substitute('—'), Some('-'));
        // Emoji are not covered: must be None (renderer skips them).
        for ch in "😊🌆🦙👍🚀".chars() {
            assert!(
                font.glyph(ch).is_none(),
                "{name}: unexpected glyph for {ch:?}"
            );
        }
    }
}

#[test]
fn question_mark_is_not_a_fallback() {
    // A '?' in output must mean the model emitted '?', never a render
    // substitution.
    let fonts = Fonts::load();
    let q = fonts.body.glyph('?').unwrap();
    let emoji = fonts.body.glyph('😊');
    assert!(emoji.is_none());
    assert_ne!(Some(q), emoji);
}

#[test]
fn japanese_psf_fonts_cover_common_jis_characters() {
    let fonts = Fonts::load();
    let (Some(small), Some(body), Some(head)) = (
        fonts.japanese_small(),
        fonts.japanese_body(),
        fonts.japanese_head(),
    ) else {
        return;
    };

    for (name, font, size) in [("small", small, 16), ("body", body, 24), ("head", head, 32)] {
        assert_eq!((font.width, font.height), (size, size));
        for ch in "日本語かなカナ漢字、。ー".chars() {
            assert!(font.glyph(ch).is_some(), "{name}: missing {ch:?}");
        }
    }

    assert_eq!(
        nr_gfx::draw::text_width_with_fallback(&fonts.body, Some(body), "A日A", 1, 0),
        48
    );
}
