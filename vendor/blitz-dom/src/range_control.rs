//! Numeric state shared by progress/meter DOM properties and painting.
use crate::{ElementData, LocalName, local_name};

#[derive(Clone, Copy, Debug)]
pub struct RangeControl {
    pub progress: bool,
    pub determinate: bool,
    pub min: f64,
    pub max: f64,
    pub value: f64,
    pub low: f64,
    pub high: f64,
    pub optimum: f64,
}
impl RangeControl {
    pub fn fraction(self) -> f64 {
        if self.max > self.min {
            (self.value - self.min) / (self.max - self.min)
        } else {
            0.0
        }
    }
    /// 0 is optimal, 1 is suboptimal, 2 is the least desirable range.
    pub fn quality(self) -> u8 {
        if self.optimum < self.low {
            if self.value <= self.low {
                0
            } else if self.value <= self.high {
                1
            } else {
                2
            }
        } else if self.optimum > self.high {
            if self.value >= self.high {
                0
            } else if self.value >= self.low {
                1
            } else {
                2
            }
        } else if self.value >= self.low && self.value <= self.high {
            0
        } else {
            1
        }
    }
}
impl ElementData {
    pub fn range_control(&self) -> Option<RangeControl> {
        let progress = self.name.local == local_name!("progress");
        if !progress && self.name.local != local_name!("meter") {
            return None;
        }
        let attr = |name: &str| {
            self.attr(LocalName::from(name))
                .and_then(|s| s.trim().parse::<f64>().ok())
                .filter(|v| v.is_finite())
        };
        let min = if progress {
            0.0
        } else {
            attr("min").unwrap_or(0.0)
        };
        let max = if progress {
            attr("max").filter(|&v| v > 0.).unwrap_or(1.)
        } else {
            attr("max").unwrap_or(1.).max(min)
        };
        let value = attr("value").unwrap_or(0.).clamp(min, max);
        let low = attr("low").unwrap_or(min).clamp(min, max);
        let high = attr("high").unwrap_or(max).clamp(low, max);
        let optimum = attr("optimum")
            .unwrap_or(min / 2. + max / 2.)
            .clamp(min, max);
        Some(RangeControl {
            progress,
            determinate: self.has_attr(local_name!("value")),
            min,
            max,
            value,
            low,
            high,
            optimum,
        })
    }
}
