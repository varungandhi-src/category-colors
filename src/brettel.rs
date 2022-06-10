use crate::color::*;

pub struct BrettelParams {
    rgb_cvd_from_rgb_1: [f32; 9],
    rgb_cvd_from_rgb_2: [f32; 9],
    separation_plane_normal: [f32; 3],
}

pub fn brettel_function(c: Color, v: Vision) -> Color {
    // TODO: How to describe what this does?
    use Vision::*;
    match v {
        Default => c,
        Achromatomaly => monochrome_with_severity(c, 0.6),
        Achromatopsia => monochrome_with_severity(c, 1.0),
        Protanopia | Deuteranopia | Tritanopia => brettel(c, v, 1.0),
        Protonomaly | Deuteranomaly | Tritanomaly => brettel(c, v, 0.6),
    }
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
