
use std::{env::args, fmt::Display, str::FromStr};

use p::{convert::FromColorUnclamped, ColorDifference, Lch, RelativeContrast};
use palette as p;
use rand::{Rng as RandRng, SeedableRng};

type Color = p::rgb::Rgb<p::encoding::srgb::Srgb, f32>;
type LinearRgb = p::rgb::Rgb<p::encoding::Linear<p::encoding::srgb::Srgb>, f32>;

#[derive(Copy, Clone)]
enum Mode {
    Light,
    #[allow(dead_code)]
    Dark,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
struct BackgroundColors {
    main: Color,
    range_selection: Color,
    // Default selection using line number gutter in blob view
    line_selection: Color,
    // Selection with mouse in blob view text
    git_added: Color,
    git_line_selection: Color,
    git_deleted: Color,
}

impl BackgroundColors {
    const COUNT: usize = 2;
    fn into_array(&self) -> [Color; Self::COUNT] {
        [
            self.main,
            // self.range_selection,
            self.line_selection,
            // self.git_added,
            // self.git_line_selection,
            // self.git_deleted,
        ]
    }

    const MODIFIABLE_COUNT: usize = 1;

    fn updateable_array(&self) -> [Color; Self::MODIFIABLE_COUNT] {
        [
            self.line_selection,
            // self.git_added,
            // self.git_line_selection,
            // self.git_deleted,
        ]
    }

    fn update(&mut self, new: [Color; Self::MODIFIABLE_COUNT]) {
        self.line_selection = new[0];
        // self.git_added = new[1];
        // self.git_line_selection = new[2];
        // self.git_deleted = new[3];
    }
}

#[track_caller]
fn rgb(s: &'static str) -> Color {
    let c = p::rgb::Rgb::<p::encoding::srgb::Srgb, u8>::from_str(s).expect("invalid rgb color");
    Color::from_format(c)
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

impl Mode {
    fn bg_colors(self) -> BackgroundColors {
        match self {
            Mode::Dark => dark_mode_bg_colors(),
            Mode::Light => light_mode_bg_colors(),
        }
    }

    fn brand_color_keys(self) -> Vec<&'static str> {
        match self {
            Mode::Dark => vec!["light", "medium"],
            Mode::Light => vec!["medium", "dark"],
        }
    }

    fn brand_colors(self) -> Vec<Color> {
        let cols = brand_colors();
        let mut out = vec![];
        for key in self.brand_color_keys().into_iter() {
            out.extend(cols[key].iter());
        }
        return out;
    }
}

use std::collections::HashMap;

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

type Rng = rand_chacha::ChaCha8Rng;

// Checked that this is close to JS
fn distance(c1: Color, c2: Color) -> f32 {
    let c1 = Lch::from_color_unclamped(c1);
    let c2 = Lch::from_color_unclamped(c2);
    // Note: This color difference is different from the one used by chroma.js
    // This uses CIEDE2000 whereas chroma.js used the older CMC l:c (1984)
    c1.get_color_difference(&c2)
}

fn get_closest_color(c: Color, cs: &[Color]) -> Color {
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

fn root_mean_square_distance(x: f32, s: &[f32]) -> f32 {
    f32::sqrt(s.iter().map(|y| (x - y) * (x - y)).sum::<f32>() / (s.len() as f32))
}

fn root_mean_square(s: &[f32]) -> f32 {
    // Don't need to worry about infinity because numbers will be small
    f32::sqrt(s.iter().map(|x| x * x).sum::<f32>() / (s.len() as f32))
}

fn bg_to_fg_distances(bg_colors: &[Color], fg_colors: &[Color], out: &mut Vec<f32>) {
    out.clear();
    for bg_color in bg_colors {
        for fg_color in fg_colors {
            out.push(distance(*bg_color, *fg_color));
        }
    }
}

fn fg_mutual_distances(fg_colors: &[Color], out: &mut Vec<f32>) {
    out.clear();
    for i in 0..fg_colors.len() {
        for j in (i + 1)..fg_colors.len() {
            out.push(distance(fg_colors[i], fg_colors[j]));
        }
    }
}

fn max_minus_min(s: &[f32]) -> f32 {
    s.iter()
        .max_by(|a, b| a.partial_cmp(b).expect("Finite floats"))
        .expect("Expected non-empty slice")
        - s.iter()
            .min_by(|a, b| a.partial_cmp(b).expect("Finite floats"))
            .expect("Expected non-empty slice")
}

fn triple_to_array(t: (f32, f32, f32)) -> [f32; 3] {
    [t.0, t.1, t.2]
}

fn array_to_triple(a: [f32; 3]) -> (f32, f32, f32) {
    (a[0], a[1], a[2])
}

fn random_nearby_color(c: Color, rng: &mut Rng) -> Color {
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
enum Vision {
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

struct BrettelParams {
    rgb_cvd_from_rgb_1: [f32; 9],
    rgb_cvd_from_rgb_2: [f32; 9],
    separation_plane_normal: [f32; 3],
}

fn brettel_params(v: Vision) -> Option<BrettelParams> {
    use Vision::*;
    match v {
        Default | Achromatomaly | Achromatopsia => None,
        Protanopia | Protonomaly => Some(BrettelParams {
            rgb_cvd_from_rgb_1: [
                0.1451, 1.20165, -0.34675, 0.10447, 0.85316, 0.04237, 0.00429, -0.00603, 1.00174,
            ],
            rgb_cvd_from_rgb_2: [
                0.14115, 1.16782, -0.30897, 0.10495, 0.8573, 0.03776, 0.00431, -0.00586, 1.00155,
            ],
            separation_plane_normal: [0.00048, 0.00416, -0.00464],
        }),
        Deuteranomaly | Deuteranopia => Some(BrettelParams {
            rgb_cvd_from_rgb_1: [
                0.36198, 0.86755, -0.22953, 0.26099, 0.64512, 0.09389, -0.01975, 0.02686, 0.99289,
            ],
            rgb_cvd_from_rgb_2: [
                0.37009, 0.8854, -0.25549, 0.25767, 0.63782, 0.10451, -0.0195, 0.02741, 0.99209,
            ],
            separation_plane_normal: [-0.00293, -0.00645, 0.00938],
        }),
        Tritanomaly | Tritanopia => Some(BrettelParams {
            rgb_cvd_from_rgb_1: [
                1.01354, 0.14268, -0.15622, -0.01181, 0.87561, 0.13619, 0.07707, 0.81208, 0.11085,
            ],
            rgb_cvd_from_rgb_2: [
                0.93337, 0.19999, -0.13336, 0.05809, 0.82565, 0.11626, -0.37923, 1.13825, 0.24098,
            ],
            separation_plane_normal: [0.0396, -0.02831, -0.01129],
        }),
    }
}

fn brettel(c_srgb: Color, v: Vision, severity: f32) -> Color {
    let c_lrgb = LinearRgb::from_encoding(c_srgb);
    let params = brettel_params(v).expect(&format!("Unexpected vision {:?}", v));

    let separation_plane_normal = params.separation_plane_normal;
    let rgb_cvd_from_rgb_1 = params.rgb_cvd_from_rgb_1;
    let rgb_cvd_from_rgb_2 = params.rgb_cvd_from_rgb_2;

    let rgb = c_lrgb.into_components();

    // Check on which plane we should project by comparing wih the separation plane normal.
    let dot_with_sep_plane = rgb.0 * separation_plane_normal[0]
        + rgb.1 * separation_plane_normal[1]
        + rgb.2 * separation_plane_normal[2];
    let rgb_cvd_from_rgb = if dot_with_sep_plane >= 0. {
        rgb_cvd_from_rgb_1
    } else {
        rgb_cvd_from_rgb_2
    };

    // Transform to the full dichromat projection plane.
    let mut rgb_cvd = (0., 0., 0.);
    rgb_cvd.0 =
        rgb_cvd_from_rgb[0] * rgb.0 + rgb_cvd_from_rgb[1] * rgb.1 + rgb_cvd_from_rgb[2] * rgb.2;
    rgb_cvd.1 =
        rgb_cvd_from_rgb[3] * rgb.0 + rgb_cvd_from_rgb[4] * rgb.1 + rgb_cvd_from_rgb[5] * rgb.2;
    rgb_cvd.2 =
        rgb_cvd_from_rgb[6] * rgb.0 + rgb_cvd_from_rgb[7] * rgb.1 + rgb_cvd_from_rgb[8] * rgb.2;

    // Apply the severity factor as a linear interpolation.
    // It's the same to do it in the RGB space or in the LMS
    // space since it's a linear transform.
    rgb_cvd.0 = rgb_cvd.0 * severity + rgb.0 * (1.0 - severity);
    rgb_cvd.1 = rgb_cvd.1 * severity + rgb.1 * (1.0 - severity);
    rgb_cvd.2 = rgb_cvd.2 * severity + rgb.2 * (1.0 - severity);

    // Go back to sRGB
    Color::from_encoding(LinearRgb::from_components(rgb_cvd))
}

fn monochrome_with_severity(c: Color, severity: f32) -> Color {
    let srgb = c.into_components();
    let z = (srgb.0 * 0.299 + srgb.1 * 0.587 + srgb.2 * 0.114).round();
    let r = z * severity + (1.0 - severity) * srgb.0;
    let g = z * severity + (1.0 - severity) * srgb.1;
    let b = z * severity + (1.0 - severity) * srgb.2;
    return Color::from_components((r, g, b));
}

fn brettel_function(c: Color, v: Vision) -> Color {
    use Vision::*;
    match v {
        Default => c,
        Achromatomaly => monochrome_with_severity(c, 0.6),
        Achromatopsia => monochrome_with_severity(c, 1.0),
        Protanopia | Deuteranopia | Tritanopia => brettel(c, v, 1.0),
        Protonomaly | Deuteranomaly | Tritanomaly => brettel(c, v, 0.6),
    }
}

#[allow(dead_code)]
enum ContrastNeed {
    Background,
    Text,
}

#[allow(dead_code)]
// Returned cost is between 0 and 100.
fn contrast_cost(contrast: f32, need: ContrastNeed) -> f32 {
    assert!(1. <= contrast && contrast <= 8.);
    let cost = |min_ratio: f32| -> f32 {
        if contrast < min_ratio {
            return 100.;
        }
        // Sigmoid pushing towards high contrast
        return 100. / (1. + (4. * (contrast - min_ratio)).exp());
    };
    match need {
        ContrastNeed::Background => cost(3.),
        ContrastNeed::Text => cost(4.5),
    }
}

#[derive(Clone)]
struct State {
    bg_colors: BackgroundColors,
    // This is kept redundant, the bg_colors are synced later.
    bg_color_array: Vec<Color>,
    fg_colors: Vec<Color>,
    target_bg_colors: Vec<Color>,
    target_fg_colors: Vec<Color>,
}

#[derive(Default)]
struct ScratchBuffers {
    bg_to_fg: Vec<f32>,
    fg_to_fg: Vec<f32>,
    bg_colors: Vec<Color>,
    fg_colors: Vec<Color>,
    target_fg_deltas: Vec<f32>,
    target_bg_deltas: Vec<f32>,
}

struct Report {
    start_cost: TotalCost,
    final_cost: TotalCost,
    start_state: State,
    final_state: State,
    duration: std::time::Duration,
    n_iterations: u64,
}

fn hex_colors(cs: &[Color]) -> Vec<String> {
    cs.iter()
        .map(|c| format!("#{:x}", c.into_format::<u8>()))
        .collect()
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cost: {} (start) → {} (final)\n",
            self.start_cost.total(),
            self.final_cost.total()
        )?;
        write!(f, "Cost breakdown:\n")?;
        write!(f, "{}\n", self.start_cost)?;
        write!(f, "        ↓\n")?;
        write!(f, "{}\n", self.final_cost)?;
        let secs = self.duration.as_secs_f32();
        write!(
            f,
            "Time: {:.2}s for {} iterations ({} iters/sec)\n",
            secs,
            self.n_iterations,
            (self.n_iterations as f32) / secs
        )?;
        write!(
            f,
            "Background colors:\n  {:?}\n",
            hex_colors(&self.start_state.bg_colors.into_array())
        )?;
        write!(
            f,
            "        ↓\n  {:?}\n\n",
            hex_colors(&self.final_state.bg_colors.into_array())
        )?;
        write!(
            f,
            "Foreground colors:\n  {:?}\n",
            hex_colors(&self.start_state.fg_colors)
        )?;
        write!(
            f,
            "        ↓\n  {:?}\n",
            hex_colors(&self.final_state.fg_colors)
        )
    }
}

struct Cost {
    value: f32,
}

impl Cost {
    fn new(value: f32) -> Cost {
        assert!(value >= 0.0);
        assert!(value <= 100.0);
        Cost { value }
    }
}

#[derive(Copy, Clone)]
struct TotalCost {
    distance_cost: f32,
    range_cost: f32,
    target_cost: f32,
    protanopia_cost: f32,
    deuteranopia_cost: f32,
    tritanopia_cost: f32,
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

    fn total(&self) -> f32 {
        Self::DISTANCE_WEIGHT * self.distance_cost
            + Self::RANGE_WEIGHT * self.range_cost
            + Self::TARGET_WEIGHT * self.target_cost
            + Self::PROTANOPIA_WEIGHT * self.protanopia_cost
            + Self::DEUTERANOPIA_WEIGHT * self.deuteranopia_cost
            + Self::TRITANOPIA_WEIGHT * self.tritanopia_cost
    }
}

impl State {
    const INITIAL_TEMPERATURE: f32 = 1000.;
    const COOLING_RATE: f32 = 0.99;
    const CUTOFF: f32 = 0.0001;

    const DISTANCE_BG_WEIGHT: f32 = 0.2;
    const DISTANCE_FG_WEIGHT: f32 = 1. - Self::DISTANCE_BG_WEIGHT;

    const TARGET_BG_WEIGHT: f32 = 0.1;
    const TARGET_FG_WEIGHT: f32 = 1. - Self::TARGET_BG_WEIGHT;

    fn distance_cost(&self, bufs: &mut ScratchBuffers, v: Vision) -> Cost {
        bufs.bg_colors.clear();
        bufs.fg_colors.clear();

        bufs.bg_colors.extend(
            self.bg_colors
                .into_array()
                .into_iter()
                .map(|c| brettel_function(c, v)),
        );

        bufs.fg_colors
            .extend(self.fg_colors.iter().map(|c| brettel_function(*c, v)));

        fg_mutual_distances(&bufs.fg_colors, &mut bufs.fg_to_fg);
        let fg_score = root_mean_square_distance(100., &bufs.fg_to_fg);

        bg_to_fg_distances(&bufs.bg_colors, &bufs.fg_colors, &mut bufs.bg_to_fg);
        let bg_score = root_mean_square_distance(100., &bufs.bg_to_fg);

        Cost::new(bg_score * Self::DISTANCE_BG_WEIGHT + fg_score * Self::DISTANCE_FG_WEIGHT)
    }

    fn target_cost(&self, bufs: &mut ScratchBuffers) -> Cost {
        bufs.target_fg_deltas.clear();
        for current in self.fg_colors.iter() {
            let closest = get_closest_color(*current, &self.target_fg_colors);
            bufs.target_fg_deltas.push(distance(*current, closest));
        }
        let target_fg_score = root_mean_square(&bufs.target_fg_deltas);

        bufs.target_bg_deltas.clear();
        for current in self.bg_color_array.iter() {
            let closest = get_closest_color(*current, &self.target_bg_colors);
            bufs.target_bg_deltas.push(distance(*current, closest));
        }
        let target_bg_score = root_mean_square(&bufs.target_bg_deltas);

        Cost::new(
            target_bg_score * Self::TARGET_BG_WEIGHT + target_fg_score * Self::TARGET_FG_WEIGHT,
        )
    }

    fn total_cost(&self, bufs: &mut ScratchBuffers) -> TotalCost {
        use Vision::*;

        return TotalCost {
            distance_cost: self.distance_cost(bufs, Default).value,
            // Range calculation has to happen after the above, so distance values are filled.
            range_cost: max_minus_min(&bufs.fg_to_fg),
            target_cost: self.target_cost(bufs).value,
            protanopia_cost: self.distance_cost(bufs, Protanopia).value,
            deuteranopia_cost: self.distance_cost(bufs, Deuteranopia).value,
            tritanopia_cost: self.distance_cost(bufs, Tritanopia).value,
        };
    }

    fn new(bg_colors: BackgroundColors, target_fg_colors: Vec<Color>) -> Self {
        State {
            bg_colors,
            bg_color_array: bg_colors.updateable_array().to_vec(),
            fg_colors: target_fg_colors.clone(),
            target_bg_colors: bg_colors.updateable_array().to_vec(),
            target_fg_colors,
        }
    }

    fn sync_bg_slot(&mut self, mut i: usize) {
        if i < self.fg_colors.len() {
            return;
        }
        i = i - self.fg_colors.len();
        let mut a = self.bg_colors.updateable_array();
        a[i] = self.bg_color_array[i];
        self.bg_colors.update(a);
    }

    fn color_slot(&mut self, i: usize) -> &mut Color {
        if i < self.fg_colors.len() {
            &mut self.fg_colors[i]
        } else {
            &mut self.bg_color_array[i - self.fg_colors.len()]
        }
    }

    fn optimize(&mut self, rng: &mut Rng) -> Report {
        let mut bufs = ScratchBuffers::default();
        let start_cost = self.total_cost(&mut bufs);
        let start_state = self.clone();
        let mut old_cost = start_cost;

        let mut temperature = Self::INITIAL_TEMPERATURE;

        let start_time = std::time::Instant::now();
        let mut n_iterations = 0;

        while temperature > Self::CUTOFF {
            for i in 0..self.fg_colors.len() + BackgroundColors::MODIFIABLE_COUNT {
                let old_color;
                {
                    let slot = self.color_slot(i);
                    old_color = *slot;
                    *slot = random_nearby_color(old_color, rng);
                    self.sync_bg_slot(i);
                }
                // FIXME: Make this incremental for better performance!
                let new_cost = self.total_cost(&mut bufs);
                let delta = new_cost.total() - old_cost.total();
                let acceptance_probability = (-delta / temperature).exp();
                let accept = rng.gen_range(0. ..=1.) < acceptance_probability;
                if accept {
                    old_cost = new_cost;
                } else {
                    // Reset!
                    *self.color_slot(i) = old_color;
                    self.sync_bg_slot(i);
                }
            }
            n_iterations += 1;
            // Cooling
            temperature *= Self::COOLING_RATE;
        }

        let duration = std::time::Instant::now() - start_time;

        Report {
            start_cost,
            final_cost: self.total_cost(&mut bufs),
            start_state,
            final_state: self.clone(),
            n_iterations,
            duration,
        }
    }
}

struct ColorDataTable<X> {
    cols: Vec<Color>,
    rows: Vec<Color>,
    data: Vec<Vec<X>>,
    info: &'static str,
}

impl<X> ColorDataTable<X> {
    fn new(
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

use prettytable::format::Alignment;
use prettytable::{Cell, Row, Table};

impl<X: Display> Display for ColorDataTable<X> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut t = Table::new();
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
                row.add_cell(Cell::new(&format!("{j}")))
            }
            t.add_row(row);
        }
        t.fmt(f)
    }
}

struct ContrastRatio {
    value: f32,
}

impl ContrastRatio {
    fn new(value: f32) -> ContrastRatio {
        if value < 1.0 {
            return ContrastRatio { value: 1. / value };
        }
        ContrastRatio { value }
    }
}

impl Display for ContrastRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = if self.value < 3.0 { "! " } else { "" };
        write!(f, "{}{:.2}:1", prefix, self.value)
    }
}

fn contrast_table(rows: Vec<Color>, cols: Vec<Color>) -> ColorDataTable<ContrastRatio> {
    ColorDataTable::new(rows, cols, "contrast", &|c1, c2| {
        ContrastRatio::new(c1.get_contrast_ratio(&c2))
    })
}

fn setup() -> Rng {
    let args = args();
    let rng;
    std::env::set_var("RUST_BACKTRACE", "1");
    if args.len() > 1 {
        let arg_vec: Vec<_> = args.collect();
        let seed_string = arg_vec[1].clone();
        let mut buf = [0u8; 32];
        let copy_len = 32.min(seed_string.len());
        for i in 0..copy_len {
            buf[i] = seed_string.as_bytes()[i];
        }
        rng = Rng::from_seed(buf);
    } else {
        rng = Rng::from_entropy();
    }
    return rng;
}

fn main() {
    let light_bgs = light_mode_bg_colors().into_array().to_vec();
    println!("Light mode background contrast");
    println!("{}", contrast_table(light_bgs.clone(), light_bgs.clone()));
    let light_fgs = Mode::Light.brand_colors();
    println!("Light mode background <-> foreground contrast");
    println!("{}", contrast_table(light_fgs.clone(), light_bgs.clone()));

    let mut rng = setup();

    let mut state = State::new(Mode::Light.bg_colors(), Mode::Light.brand_colors());
    let report = state.optimize(&mut rng);

    let new_bg_colors = report.final_state.bg_colors.into_array().to_vec();
    println!("Updated Light mode background contrast");
    println!(
        "{}",
        contrast_table(new_bg_colors.clone(), new_bg_colors.clone())
    );

    let new_fg_colors = report.final_state.fg_colors.clone();
    println!("Updated Light mode bg <-> fg contrast");
    println!(
        "{}",
        contrast_table(new_fg_colors.clone(), new_bg_colors.clone())
    );

    println!("{report}");
}