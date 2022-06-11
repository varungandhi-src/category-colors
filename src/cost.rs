use std::fmt::Display;

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

#[allow(dead_code)]
pub struct CriterionCost {
    bg_bg: ScaledCost,
    bg_fg: ScaledCost,
    fg_fg: ScaledCost,
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

#[derive(Clone)]
pub struct TotalCost {
    pub contrast_cost: f32,
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
            "contrast={:.2}  distance={:.2}  target={:.2}  range={:.2}  a11y={:.2},{:.2},{:.2}",
            self.contrast_cost,
            self.distance_cost,
            self.target_cost,
            self.range_cost,
            self.protanopia_cost,
            self.deuteranopia_cost,
            self.tritanopia_cost
        )
    }
}

#[derive(Clone)]
pub struct Weights {
    pub contrast_weight: f32,
    pub distance_weight: f32,
    pub range_weight: f32,
    pub target_weight: f32,
    pub protanopia_weight: f32,
    pub deuteranopia_weight: f32,
    pub tritanopia_weight: f32,

    pub distance_bg_bg_weight: f32,
    pub distance_bg_fg_weight: f32,
    pub distance_fg_fg_weight: f32,

    pub target_bg_weight: f32,
    pub target_fg_weight: f32,

    pub contrast_bg_bg_weight: f32,
    pub contrast_bg_fg_weight: f32,
}

impl Weights {
    pub fn initialize(mut self) -> Self {
        assert!((0.99..=1.01).contains(
            &(self.distance_bg_bg_weight + self.distance_bg_fg_weight + self.distance_fg_fg_weight)
        ));
        self.distance_fg_fg_weight = 1. - (self.distance_bg_bg_weight + self.distance_bg_fg_weight);
        assert!((0.99..=1.01).contains(&(self.target_bg_weight + self.target_fg_weight)));
        self.target_fg_weight = 1. - self.target_bg_weight;
        assert!((0.99..=1.01).contains(&(self.contrast_bg_bg_weight + self.contrast_bg_fg_weight)));
        self.contrast_bg_fg_weight = 1. - self.contrast_bg_bg_weight;

        self
    }
}

impl TotalCost {
    pub fn total(&self, w: &Weights) -> f32 {
        w.contrast_weight * self.contrast_cost
            + w.distance_weight * self.distance_cost
            + w.range_weight * self.range_cost
            + w.target_weight * self.target_cost
            + w.protanopia_weight * self.protanopia_cost
            + w.deuteranopia_weight * self.deuteranopia_cost
            + w.tritanopia_weight * self.tritanopia_cost
    }
}
