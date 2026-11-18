//! Convert CSS blending modes to the shared CPU/WASM painting representation.
use peniko::{BlendMode, Compose, Mix};
use style::values::specified::effects::Blend;

pub(crate) fn blend_mode(value: Blend) -> BlendMode {
    let mix = match value {
        Blend::Normal => Mix::Normal,
        Blend::Multiply => Mix::Multiply,
        Blend::Screen => Mix::Screen,
        Blend::Overlay => Mix::Overlay,
        Blend::Darken => Mix::Darken,
        Blend::Lighten => Mix::Lighten,
        Blend::ColorDodge => Mix::ColorDodge,
        Blend::ColorBurn => Mix::ColorBurn,
        Blend::HardLight => Mix::HardLight,
        Blend::SoftLight => Mix::SoftLight,
        Blend::Difference => Mix::Difference,
        Blend::Exclusion => Mix::Exclusion,
        Blend::Hue => Mix::Hue,
        Blend::Saturation => Mix::Saturation,
        Blend::Color => Mix::Color,
        Blend::Luminosity => Mix::Luminosity,
        Blend::PlusLighter => return BlendMode::new(Mix::Normal, Compose::PlusLighter),
    };
    mix.into()
}
