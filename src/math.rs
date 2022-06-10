pub fn root_mean_square_distance(x: f32, s: &[f32]) -> f32 {
    f32::sqrt(s.iter().map(|y| (x - y) * (x - y)).sum::<f32>() / (s.len() as f32))
}

pub fn root_mean_square(s: &[f32]) -> f32 {
    // Don't need to worry about infinity because numbers will be small
    f32::sqrt(s.iter().map(|x| x * x).sum::<f32>() / (s.len() as f32))
}

pub fn max_minus_min(s: &[f32]) -> f32 {
    assert!(s.len() > 0);
    let mut max: f32 = f32::NEG_INFINITY;
    let mut min: f32 = f32::INFINITY;
    for x in s.iter() {
        max = max.max(*x);
        min = min.min(*x);
    }
    max - min
}
