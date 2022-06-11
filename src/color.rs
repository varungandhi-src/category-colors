use std::{cmp::Ordering, fmt::Display, str::FromStr};

use p::{convert::FromColorUnclamped, ColorDifference, Lch, RelativeContrast};
use palette as p;
use rand::Rng as RngTrait;

use crate::{
    convert::{array_to_triple, triple_to_array},
    cost::{ContrastNeed, ScaledCost},
    random::Rng,
};

pub type Color = p::rgb::Rgb<p::encoding::srgb::Srgb, f32>;
pub type LinearRgb = p::rgb::Rgb<p::encoding::Linear<p::encoding::srgb::Srgb>, f32>;

#[track_caller]
pub fn rgb(s: &'static str) -> Color {
    let c = p::rgb::Rgb::<p::encoding::srgb::Srgb, u8>::from_str(s).expect("invalid rgb color");
    Color::from_format(c)
}

// Checked that this is close to JS
pub fn distance(c1: Color, c2: Color) -> f32 {
    let c1 = Lch::from_color_unclamped(c1);
    let c2 = Lch::from_color_unclamped(c2);
    // Note: This color difference is different from the one used by chroma.js
    // This uses CIEDE2000 whereas chroma.js used the older CMC l:c (1984)
    c1.get_color_difference(&c2)
}

pub fn get_closest_color(c: Color, cs: &[Color]) -> Color {
    assert!(cs.len() > 0);
    let mut out = None;
    let mut closest = 1e10;
    for x in cs.iter() {
        let d = distance(c, *x);
        if d < closest {
            closest = d;
            out = Some(*x);
        }
    }
    out.unwrap()
}

pub fn pairwise_distances_2(bg_colors: &[Color], fg_colors: &[Color], out: &mut Vec<f32>) {
    out.clear();
    for bg_color in bg_colors {
        for fg_color in fg_colors {
            out.push(distance(*bg_color, *fg_color));
        }
    }
}

pub fn pairwise_distances(fg_colors: &[Color], out: &mut Vec<f32>) {
    out.clear();
    for i in 0..fg_colors.len() {
        for j in (i + 1)..fg_colors.len() {
            out.push(distance(fg_colors[i], fg_colors[j]));
        }
    }
}

pub fn random_nearby_color(c: Color, rng: &mut Rng) -> Color {
    let channel = rng.gen_range(0..3);
    // NOTE: The original code in category-colors uses chroma.js's
    // chroma.Color's .gl() method which is documented to return CMYK.
    // The perturbation is seemingly done in CMYK space and then converted
    // back. However, if you look at the output of that function, it returns
    // RGBA values. 💩
    let mut rgb = triple_to_array(c.into_components());
    let old_val = rgb[channel];

    const WIGGLE: f32 = 0.05;
    let new_val = f32::clamp(old_val + rng.gen_range(-WIGGLE..=WIGGLE), 0., 1.);

    rgb[channel] = new_val;
    Color::from_components(array_to_triple(rgb))
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum Vision {
    Default,
    Protanopia,
    Protonomaly,
    Deuteranopia,
    Deuteranomaly,
    Tritanopia,
    Tritanomaly,
    Achromatopsia,
    Achromatomaly,
}

pub fn hex_colors(cs: &[Color]) -> Vec<String> {
    cs.iter()
        .map(|c| format!("#{:x}", c.into_format::<u8>()))
        .collect()
}

use prettytable::{format::Alignment, Attr};
use prettytable::{Cell, Row, Table};

pub struct ColorDataTable<X> {
    cols: Vec<Color>,
    rows: Vec<Color>,
    data: Vec<Vec<X>>,
    info: &'static str,
}

enum Attention {
    Good,
    Normal,
    Bad,
}

pub trait DrawAttention {
    fn attention(&self) -> Attention;
}

impl<X> ColorDataTable<X> {
    pub fn new(
        rows: Vec<Color>,
        cols: Vec<Color>,
        info: &'static str,
        build: &dyn Fn(Color, Color) -> X,
    ) -> ColorDataTable<X> {
        let mut data = vec![];
        for row_color in rows.iter() {
            let mut data_row = vec![];
            for col_color in cols.iter() {
                data_row.push(build(*row_color, *col_color));
            }
            data.push(data_row);
        }
        ColorDataTable {
            cols,
            rows,
            info,
            data,
        }
    }
}

impl<X: Clone> ColorDataTable<X> {
    pub fn sort_rows(&mut self, compare: &dyn Fn(&[X], &[X]) -> Ordering) {
        let mut glued: Vec<_> = self
            .rows
            .clone()
            .into_iter()
            .zip(self.data.clone().into_iter())
            .collect();
        glued.sort_by(|(_, v1), (_, v2)| compare(v1, v2));
        for (i, (r, d)) in glued.into_iter().enumerate() {
            self.rows[i] = r;
            self.data[i] = d;
        }
    }
}

impl<X: Display + DrawAttention> ColorDataTable<X> {
    pub fn table(&self) -> prettytable::Table {
        let mut t = Table::new();
        t.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        let mut headings = vec![self.info.to_string()];
        headings.extend(hex_colors(&self.cols));
        t.add_row(Row::new(
            headings
                .into_iter()
                .map(|s| {
                    let mut c = Cell::new(&s);
                    c.align(Alignment::CENTER);
                    return c;
                })
                .collect(),
        ));
        for (i, row_color) in hex_colors(&self.rows).into_iter().enumerate() {
            let mut row = Row::new(vec![Cell::new(&row_color)]);
            for j in self.data[i].iter() {
                let mut c = Cell::new(&format!("{j}"));
                match j.attention() {
                    Attention::Normal => {}
                    Attention::Bad => {
                        c = c.with_style(Attr::Standout(true));
                    }
                    Attention::Good => {
                        c = c.with_style(Attr::Underline(true));
                    }
                }
                row.add_cell(c);
            }
            t.add_row(row);
        }
        return t;
    }
}

impl<X: Display + DrawAttention> Display for ColorDataTable<X> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.table().fmt(f)
    }
}

#[derive(Copy, Clone)]
pub struct ContrastRatio {
    value: f32,
    need: ContrastNeed,
}

impl ContrastRatio {
    pub fn new(value: f32, need: ContrastNeed) -> ContrastRatio {
        if value < 1.0 {
            return ContrastRatio {
                value: 1. / value,
                need,
            };
        }
        ContrastRatio { value, need }
    }
    pub fn for_pair(c1: Color, c2: Color, need: ContrastNeed) -> ContrastRatio {
        Self::new(c1.get_contrast_ratio(&c2), need)
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn need(&self) -> ContrastNeed {
        self.need
    }
    pub fn cost(&self) -> ScaledCost {
        let ratio = self.value();
        assert!(1. <= ratio && ratio <= 21.);
        let min_ratio = self.need().minimum_ratio();
        if ratio < min_ratio {
            return ScaledCost::new(100.);
        }
        // Sigmoid pushing towards high contrast
        ScaledCost::new(100. / (1. + (4. * (self.value() - ratio)).exp()))
    }
}

impl DrawAttention for ContrastRatio {
    fn attention(&self) -> Attention {
        if self.value() < self.need().minimum_ratio() {
            return Attention::Bad;
        }
        return Attention::Normal;
    }
}

impl Display for ContrastRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}:1", self.value)
    }
}

pub fn contrast_table(
    rows: Vec<Color>,
    cols: Vec<Color>,
    need: ContrastNeed,
) -> ColorDataTable<ContrastRatio> {
    ColorDataTable::new(rows, cols, "contrast", &|c1, c2| {
        ContrastRatio::new(c1.get_contrast_ratio(&c2), need)
    })
}
