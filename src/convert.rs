
pub fn triple_to_array(t: (f32, f32, f32)) -> [f32; 3] {
    [t.0, t.1, t.2]
}

pub fn array_to_triple(a: [f32; 3]) -> (f32, f32, f32) {
    (a[0], a[1], a[2])
}