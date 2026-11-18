//! Snap layout metrics in logical pixels before device scaling.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Quantization {
    step: f32,
    enabled: bool,
}

impl Quantization {
    pub(crate) fn new(scale: f32, enabled: bool) -> Self {
        Self {
            step: if scale.is_finite() && scale > 0.0 {
                scale
            } else {
                1.0
            },
            enabled,
        }
    }

    pub(crate) fn css_pixel(self) -> f32 {
        self.step
    }

    pub(crate) fn enabled(self) -> bool {
        self.enabled
    }

    pub(crate) fn round(self, value: f32) -> f32 {
        (value / self.step).round() * self.step
    }

    pub(crate) fn floor(self, value: f32) -> f32 {
        (value / self.step).floor() * self.step
    }

    pub(crate) fn round_f64(self, value: f64) -> f32 {
        ((value / f64::from(self.step)).round() * f64::from(self.step)) as f32
    }
}
