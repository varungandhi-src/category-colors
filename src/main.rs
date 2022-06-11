use std::{env::args, fmt::Display};

use palette::RelativeContrast;
use rand::{Rng as RandRng, SeedableRng};

mod brettel;
mod color;
mod convert;
mod cost;
mod math;
mod random;
mod sg;

use crate::brettel::*;
use crate::color::*;
use crate::cost::*;
use crate::math::*;
use crate::random::*;
use crate::sg::*;

#[derive(Clone)]
struct State {
    bg_colors: BackgroundColors,
    // This is kept redundant, the bg_colors are synced later.
    bg_color_array: Vec<Color>,
    fg_colors: Vec<Color>,
    target_bg_colors: Vec<Color>,
    target_fg_colors: Vec<Color>,
    weights: Weights,
}

#[derive(Default)]
struct ScratchBuffers {
    // For color transformation (before distance computation)
    bg_colors: Vec<Color>,
    fg_colors: Vec<Color>,

    // Intermediate distances/contrast/target deltas.
    bg_to_bg: Vec<f32>,
    bg_to_fg: Vec<f32>,
    fg_to_fg: Vec<f32>,
}

struct Report {
    start_cost: TotalCost,
    final_cost: TotalCost,
    start_state: State,
    final_state: State,
    duration: std::time::Duration,
    n_iterations: u64,
    weights: Weights,
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cost: {} (start) → {} (final)\n",
            self.start_cost.total(&self.weights),
            self.final_cost.total(&self.weights)
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

impl State {
    const INITIAL_TEMPERATURE: f32 = 1000.;
    const COOLING_RATE: f32 = 0.99;
    const CUTOFF: f32 = 0.0001;

    fn distance_cost(&self, bufs: &mut ScratchBuffers, v: Vision) -> ScaledCost {
        // Map to bretter-function transformed colors first.
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

        // Compute distances and scores if needed.
        let mut bg_bg_score: f32 = 0.;
        if self.weights.distance_bg_bg_weight != 0. {
            pairwise_distances(&bufs.bg_colors, &mut bufs.bg_to_bg);
            bg_bg_score = root_mean_square_distance(100., &bufs.bg_to_bg);
        }

        let mut bg_fg_score: f32 = 0.;
        if self.weights.distance_bg_fg_weight != 0. {
            pairwise_distances_2(&bufs.bg_colors, &bufs.fg_colors, &mut bufs.bg_to_fg);
            bg_fg_score = root_mean_square_distance(100., &bufs.bg_to_fg);
        }

        let mut fg_fg_score: f32 = 0.;
        if self.weights.distance_fg_fg_weight != 0. {
            pairwise_distances(&bufs.fg_colors, &mut bufs.fg_to_fg);
            fg_fg_score = root_mean_square_distance(100., &bufs.fg_to_fg);
        }

        ScaledCost::new(
            bg_bg_score * self.weights.distance_bg_bg_weight
                + bg_fg_score * self.weights.distance_bg_fg_weight
                + fg_fg_score * self.weights.distance_fg_fg_weight,
        )
    }

    fn target_cost(&self, bufs: &mut ScratchBuffers) -> ScaledCost {
        let mut target_bg_score: f32 = 0.;
        if self.weights.target_bg_weight != 0. {
            bufs.bg_to_bg.clear();
            for current in self.bg_color_array.iter() {
                let closest = get_closest_color(*current, &self.target_bg_colors);
                bufs.bg_to_bg.push(distance(*current, closest));
            }
            target_bg_score = root_mean_square(&bufs.bg_to_bg);
        }

        let mut target_fg_score: f32 = 0.;
        if self.weights.target_fg_weight != 0. {
            bufs.fg_to_fg.clear();
            for current in self.fg_colors.iter() {
                let closest = get_closest_color(*current, &self.target_fg_colors);
                bufs.fg_to_fg.push(distance(*current, closest));
            }
            target_fg_score = root_mean_square(&bufs.fg_to_fg);
        }

        ScaledCost::new(
            target_bg_score * self.weights.target_bg_weight
                + target_fg_score * self.weights.target_fg_weight,
        )
    }

    fn contrast_cost(&self, bufs: &mut ScratchBuffers) -> ScaledCost {
        let mut contrast_bg_bg_score: f32 = 0.;
        if self.weights.contrast_bg_bg_weight != 0. {
            contrast_bg_bg_score = self.bg_colors.contrast_cost().value();
        }

        let mut contrast_bg_fg_score: f32 = 0.;
        if self.weights.contrast_bg_fg_weight != 0. {
            bufs.bg_to_fg.clear();
            for bg in self.bg_color_array.iter() {
                for fg in self.fg_colors.iter() {
                    bufs.bg_to_fg.push(
                        ContrastRatio::for_pair(*bg, *fg, ContrastNeed::Text)
                            .cost()
                            .value(),
                    );
                }
            }
            contrast_bg_fg_score = root_mean_square(&bufs.bg_to_fg);
        }

        ScaledCost::new(
            contrast_bg_bg_score * self.weights.contrast_bg_bg_weight
                + contrast_bg_fg_score * self.weights.contrast_bg_fg_weight,
        )
    }

    fn total_cost(&self, bufs: &mut ScratchBuffers) -> TotalCost {
        use Vision::*;

        return TotalCost {
            contrast_cost: self.contrast_cost(bufs).value(),
            distance_cost: self.distance_cost(bufs, Default).value(),
            // Range calculation has to happen after the above, so distance values are filled.
            range_cost: max_minus_min(&bufs.fg_to_fg),
            target_cost: self.target_cost(bufs).value(),
            protanopia_cost: self.distance_cost(bufs, Protanopia).value(),
            deuteranopia_cost: self.distance_cost(bufs, Deuteranopia).value(),
            tritanopia_cost: self.distance_cost(bufs, Tritanopia).value(),
        };
    }

    fn new(bg_colors: BackgroundColors, target_fg_colors: Vec<Color>, weights: Weights) -> Self {
        State {
            bg_colors,
            bg_color_array: bg_colors.updateable_array().to_vec(),
            fg_colors: target_fg_colors.clone(),
            target_bg_colors: bg_colors.updateable_array().to_vec(),
            target_fg_colors,
            weights,
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
        let mut old_cost = start_cost.clone();

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
                let delta = new_cost.total(&self.weights) - old_cost.total(&self.weights);
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
            weights: self.weights.clone(),
        }
    }
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

fn print_contrast_table(rows: Vec<Color>, cols: Vec<Color>, need: ContrastNeed) {
    println!("");
    let mut t = contrast_table(rows, cols, need);
    t.sort_rows(&|cr1, cr2| {
        let v1: Vec<_> = cr1.iter().map(|cr| cr.value()).collect();
        let v2: Vec<_> = cr2.iter().map(|cr| cr.value()).collect();
        root_mean_square(&v1)
            .partial_cmp(&root_mean_square(&v2))
            .expect("Failed float comparison!")
    });
    t.table().printstd();
    println!("");
}

fn main() {
    mode_main(Mode::Dark);
    mode_main(Mode::Light);
}

fn default_weights() -> Weights {
    Weights {
        contrast_weight: 2.,
        distance_weight: 0.75,
        range_weight: 0.25,
        target_weight: 0.50,
        protanopia_weight: 0.33,
        deuteranopia_weight: 0.33,
        tritanopia_weight: 0.33,
        distance_bg_bg_weight: 0.1,
        distance_bg_fg_weight: 0.2,
        distance_fg_fg_weight: 0.7,
        target_bg_weight: 0.1,
        target_fg_weight: 0.9,
        contrast_bg_bg_weight: 0.2,
        contrast_bg_fg_weight: 0.8,
    }
    .initialize()
}

fn mode_main(mode: Mode) {
    let bgs = mode.bg_colors().into_array().to_vec();
    println!("{} mode background contrast", mode.text());
    print_contrast_table(bgs.clone(), bgs.clone(), ContrastNeed::Background);

    let fgs = mode.brand_colors();
    println!("{} mode background ↔ foreground contrast", mode.text());
    print_contrast_table(fgs.clone(), bgs.clone(), ContrastNeed::Text);

    let mut rng = setup();

    let mut state = State::new(mode.bg_colors(), mode.brand_colors(), default_weights());
    let report = state.optimize(&mut rng);

    let new_bg_colors = report.final_state.bg_colors.into_array().to_vec();
    println!("Updated {} mode background contrast", mode.text());
    print_contrast_table(
        new_bg_colors.clone(),
        new_bg_colors.clone(),
        ContrastNeed::Background,
    );

    let new_fg_colors = report.final_state.fg_colors.clone();
    print!("Updated {} mode bg ↔ fg contrast", mode.text());
    print_contrast_table(
        new_fg_colors.clone(),
        new_bg_colors.clone(),
        ContrastNeed::Text,
    );

    println!("{report}");
}
