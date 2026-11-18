// Copyright 2006 The Android Open Source Project
// Copyright 2012 Google Inc.
// Adapted color reduction and preblend arithmetic: see ../LICENSE.skia.
//! Foreground-dependent grayscale coverage for the shared approximate Mac profile.
//!
//! Existing outline widening approximates black text. The generated tables
//! invert that black sRGB preblend before applying the requested color's table.
//! This preserves black coverage; it does not emulate CoreText scan conversion.

include!("glyph_gamma_tables.rs");

/// Three-bit canonical channels, halved for non-LCD CoreText smoothing, then
/// canonicalized again before the grayscale luminance table is selected.
pub(crate) fn index(color: [f32; 4]) -> u8 {
    let canonical = |v: u32| {
        let index = v >> 5;
        (index << 5) | (index << 2) | (index >> 1)
    };
    let component = |v: f32| canonical(canonical((v.clamp(0.0, 1.0) * 255.0 + 0.5) as u32) / 2);
    let [r, g, b, _] = color;
    let luminance = (component(r) * 54 + component(g) * 183 + component(b) * 19) >> 8;
    (luminance >> 5) as u8
}
pub(crate) fn apply(alphas: &mut [u8], index: Option<u8>) {
    if let Some(index) = index.filter(|&index| index != 0) {
        let table = &TABLES[usize::from(index)];
        for alpha in alphas {
            *alpha = table[usize::from(*alpha)];
        }
    }
}
