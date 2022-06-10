use std::fmt::Display;

use crate::color::ContrastRatio;

#[derive(Copy, Clone)]
pub enum ContrastNeed {
    Background,
    Text,
}

impl ContrastNeed {
    pub fn minimum_ratio(self) -> f32 {
        match self {
            ContrastNeed::Background => 3.,
            ContrastNeed::Text => 4.5,
        }
    }
}

#[derive(Copy, Clone)]
// Utility struct for inserting assertions about cost values.
//
// This applies to intermediate cost compuations but not the total cost.
pub struct ScaledCost {
    value: f32,
}

impl ScaledCost {
    pub fn new(value: f32) -> ScaledCost {
        assert!(value >= 0.0);
        assert!(value <= 100.0);
        ScaledCost { value }
    }
    pub fn value(&self) -> f32 {
        self.value
    }
}


#[allow(dead_code)]
// Returned cost is between 0 and 100.
pub fn contrast_cost(contrast: ContrastRatio) -> ScaledCost {
    let ratio = contrast.value();
    assert!(1. <= ratio && ratio <= 21.);
    let min_ratio = contrast.need().minimum_ratio();
    if ratio < min_ratio {
        return ScaledCost::new(100.)
    }
    // Sigmoid pushing towards high contrast
    ScaledCost::new(100. / (1. + (4. * (contrast.value() - ratio)).exp()))
}

#[derive(Copy, Clone)]
pub struct TotalCost {
    pub distance_cost: f32,
    pub range_cost: f32,
    pub target_cost: f32,
    pub protanopia_cost: f32,
    pub deuteranopia_cost: f32,
    pub tritanopia_cost: f32,
}

impl Display for TotalCost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "distance={:.2}  target={:.2}  range={:.2}  a11y={:.2},{:.2},{:.2}",
            self.distance_cost,
            self.target_cost,
            self.range_cost,
            self.protanopia_cost,
            self.deuteranopia_cost,
            self.tritanopia_cost
        )
    }
}

impl TotalCost {
    const DISTANCE_WEIGHT: f32 = 1.;
    const RANGE_WEIGHT: f32 = 0.5;
    const TARGET_WEIGHT: f32 = 1.;
    const PROTANOPIA_WEIGHT: f32 = 0.33;
    const DEUTERANOPIA_WEIGHT: f32 = 0.33;
    const TRITANOPIA_WEIGHT: f32 = 0.33;

    pub fn total(&self) -> f32 {
        Self::DISTANCE_WEIGHT * self.distance_cost
            + Self::RANGE_WEIGHT * self.range_cost
            + Self::TARGET_WEIGHT * self.target_cost
            + Self::PROTANOPIA_WEIGHT * self.protanopia_cost
            + Self::DEUTERANOPIA_WEIGHT * self.deuteranopia_cost
            + Self::TRITANOPIA_WEIGHT * self.tritanopia_cost
    }
}