use std::collections::HashMap;

use palette::RelativeContrast;

use crate::{color::*, cost::{contrast_cost, ContrastNeed, ScaledCost}, math::root_mean_square};

#[derive(Copy, Clone)]
pub enum Mode {
    Light,
    #[allow(dead_code)]
    Dark,
}

impl Mode {
    pub fn bg_colors(self) -> BackgroundColors {
        match self {
            Mode::Dark => dark_mode_bg_colors(),
            Mode::Light => light_mode_bg_colors(),
        }
    }

    pub fn brand_color_keys(self) -> Vec<&'static str> {
        match self {
            Mode::Dark => vec!["light", "medium"],
            Mode::Light => vec!["medium", "dark"],
        }
    }

    pub fn brand_colors(self) -> Vec<Color> {
        let cols = brand_colors();
        let mut out = vec![];
        for key in self.brand_color_keys().into_iter() {
            out.extend(cols[key].iter());
        }
        return out;
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct BackgroundColors {
    main: Color,
    /// Selection with mouse in blob view text
    range_selection: Color,
    /// Default selection using line number gutter in blob view
    line_selection: Color,
    git_added: Color,
    git_line_selection: Color,
    git_deleted: Color,
}

impl BackgroundColors {
    pub const COUNT: usize = 2;
    pub fn into_array(&self) -> [Color; Self::COUNT] {
        [
            self.main,
            // self.range_selection,
            self.line_selection,
            // self.git_added,
            // self.git_line_selection,
            // self.git_deleted,
        ]
    }

    pub const MODIFIABLE_COUNT: usize = 1;

    pub fn updateable_array(&self) -> [Color; Self::MODIFIABLE_COUNT] {
        [
            self.line_selection,
            // self.git_added,
            // self.git_line_selection,
            // self.git_deleted,
        ]
    }

    pub fn update(&mut self, new: [Color; Self::MODIFIABLE_COUNT]) {
        self.line_selection = new[0];
        // self.git_added = new[1];
        // self.git_line_selection = new[2];
        // self.git_deleted = new[3];
    }

    pub fn contrast_cost(&self) -> ScaledCost {
        let pairs = vec![
            (self.main, self.range_selection),
            (self.main, self.line_selection),
            (self.main, self.git_added),
            (self.main, self.git_line_selection),
            (self.main, self.git_deleted),

            (self.range_selection, self.line_selection),
            (self.range_selection, self.git_added),
            (self.range_selection, self.git_line_selection),
            (self.range_selection, self.git_deleted),

            (self.git_added, self.git_line_selection),
            (self.git_added, self.git_deleted),
            (self.git_line_selection, self.git_deleted),
        ];
        let mut contrast_values = Vec::with_capacity(pairs.len());
        for (c1, c2) in pairs.into_iter() {
            let need = ContrastNeed::Background;
            contrast_values.push(contrast_cost(
                ContrastRatio::new(c1.get_contrast_ratio(&c2), need),
            ).value());
        }
        ScaledCost::new(root_mean_square(&contrast_values))
    }

}

fn dark_mode_bg_colors() -> BackgroundColors {
    BackgroundColors {
        main: rgb("#1d212f"),
        line_selection: rgb("#343a4d"),
        range_selection: rgb("#886547"),
        git_deleted: rgb("#3e1d1d"),
        git_line_selection: rgb("#14171f"),
        git_added: rgb("#224035"),
    }
}

fn light_mode_bg_colors() -> BackgroundColors {
    BackgroundColors {
        main: rgb("#ffffff"),
        line_selection: rgb("#e6ebf2"),
        range_selection: rgb("#fedabd"),
        git_deleted: rgb("#ffecec"),
        git_line_selection: rgb("#e6ebf2"),
        git_added: rgb("#eeffec"),
    }
}

// From
// https://handbook.sourcegraph.com/departments/engineering/product/design/brand_guidelines/color/#secondary-colors
//
// These also include the primary colors.
fn brand_colors() -> HashMap<&'static str, Vec<Color>> {
    let mut h = HashMap::new();
    h.insert(
        "mist",
        [
            "#fff2cf", // yellow mist
            "#ffc9c9", // orange mist
            "#ffd1f2", // pink mist
            "#e8d1ff", // violet mist
            "#bfbfff", // plum mist
            "#c7ffff", // blue mist
            "#c4ffe8", // green mist
        ],
    );
    h.insert(
        "light",
        [
            "#ffdb45", // lemon
            "#ff5543", // vermillion
            "#d62687", // cerise
            "#a112ff", // vivid violet
            "#6b59ed", // plum
            "#00cbec", // sky blue
            "#8fedcf", // mint
        ],
    );
    h.insert(
        "medium",
        [
            "#ffc247", // orange
            "#ed2e20", // pomegranate
            "#c4147d", // red violet
            "#820dde", // electric violet
            "#5033E1", // blurple
            "#00a1c7", // pacific blue
            "#17ab52", // mountain meadow
        ],
    );
    h.insert(
        "dark",
        [
            "#ff9933", // carrot
            "#c22626", // poppy
            "#9e1769", // disco
            "#6112a3", // seance
            "#3826cc", // persian blue
            "#005482", // orient
            "#1f7d45", // eucalyptus
        ],
    );
    h.into_iter()
        .map(|(k, v)| (k, v.map(rgb).into_iter().collect()))
        .collect()
}

// fn alert_colors() -> Vec<Color> {
//     ["#82a460", "#c3c865", "#bb3926"]
//         .map(rgb)
//         .into_iter()
//         .collect()
// }
