use eframe::egui;
use rand::RngCore;
use std::cmp::Ordering;

// https://github.com/muak/ColorMinePortable/blob/master/ColorMinePortable/ColorSpaces/Conversions/LabConverter.cs
// https://github.com/muak/ColorMinePortable/blob/master/ColorMinePortable/ColorSpaces/Conversions/XyzConverter.cs

fn rgb_to_lab(rgb: egui::Color32) -> (f64, f64, f64) {
    fn pivot_rgb(n: f64) -> f64 {
        100.0
            * if n > 0.04045 {
                ((n + 0.055) / 1.055).powf(2.4)
            } else {
                n / 12.92
            }
    }

    let r = pivot_rgb(rgb.r() as f64 / 255.0);
    let g = pivot_rgb(rgb.g() as f64 / 255.0);
    let b = pivot_rgb(rgb.b() as f64 / 255.0);

    let x = r * 0.4124 + g * 0.3576 + b * 0.1805;
    let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
    let z = r * 0.0193 + g * 0.1192 + b * 0.9505;

    let white_x = 95.047;
    let white_y = 100.0;
    let white_z = 108.883;
    let epsilon = 0.008856;
    let kappa = 903.3;

    let pivot_xyz = |n: f64| {
        if n > epsilon {
            n.cbrt()
        } else {
            (kappa * n + 16.0) / 116.0
        }
    };

    let x = pivot_xyz(x / white_x);
    let y = pivot_xyz(y / white_y);
    let z = pivot_xyz(z / white_z);

    (0f64.max(116.0 * y - 16.0), 500.0 * (x - y), 200.0 * (y - z))
}

fn lab_to_rgb(l: f64, a: f64, b: f64) -> egui::Color32 {
    let y = (l + 16.0) / 116.0;
    let x = a / 500.0 + y;
    let z = y - b / 200.0;

    let white_x = 95.047;
    let white_y = 100.0;
    let white_z = 108.883;

    fn cube(n: f64) -> f64 {
        n * n * n
    }

    let x3 = cube(x);
    let z3 = cube(z);

    let epsilon = 0.008856;
    let kappa = 903.3;
    let (x, y, z) = (
        white_x
            * if x3 > epsilon {
                x3
            } else {
                (x - 16.0 / 116.0) / 7.787
            },
        white_y
            * if l > kappa * epsilon {
                cube(y)
            } else {
                l / kappa
            },
        white_z
            * if z3 > epsilon {
                z3
            } else {
                (z - 16.0 / 116.0) / 7.787
            },
    );

    let x = x / 100.0;
    let y = y / 100.0;
    let z = z / 100.0;

    let r = x * 3.2406 + y * -1.5372 + z * -0.4986;
    let g = x * -0.9689 + y * 1.8758 + z * 0.0415;
    let b = x * 0.0557 + y * -0.2040 + z * 1.0570;

    let r = if r > 0.0031308 {
        1.055 * r.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * r
    };
    let g = if g > 0.0031308 {
        1.055 * g.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * g
    };
    let b = if b > 0.0031308 {
        1.055 * b.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * b
    };

    fn to_rgb(n: f64) -> u8 {
        (n * 255.0).clamp(0.0, 255.0) as u8
    }

    egui::Color32::from_rgb(to_rgb(r), to_rgb(g), to_rgb(b))
}

// https://en.wikipedia.org/wiki/Color_difference#CIELAB_%CE%94E*
fn color_difference_squared(color1: egui::Color32, color2: egui::Color32) -> f64 {
    let (l1, a1, b1) = rgb_to_lab(color1);
    let (l2, a2, b2) = rgb_to_lab(color2);

    let c_star_1 = (a1 * a1 + b1 * b1).sqrt();
    let c_star_2 = (a2 * a2 + b2 * b2).sqrt();
    let delta_l_prime = l2 - l1;
    let l_bar = (l1 + l2) * 0.5;
    let c_bar = (c_star_1 + c_star_2) * 0.5;
    let a_prime_1 =
        a1 + a1 * 0.5 * (1.0 - (c_bar.powi(7) / (c_bar.powi(7) + 25i64.pow(7) as f64)).sqrt());
    let a_prime_2 =
        a2 + a2 * 0.5 * (1.0 - (c_bar.powi(7) / (c_bar.powi(7) + 25i64.pow(7) as f64)).sqrt());
    let h_prime_1 = b1.atan2(a_prime_1).to_degrees().rem_euclid(360.0);
    let h_prime_2 = b2.atan2(a_prime_2).to_degrees().rem_euclid(360.0);
    let delta_h_prime = if (h_prime_1 - h_prime_2).abs() <= 180.0 {
        h_prime_2 - h_prime_1
    } else if h_prime_2 <= h_prime_1 {
        h_prime_2 - h_prime_1 + 360.0
    } else {
        h_prime_2 - h_prime_1 - 360.0
    };
    let c_prime_1 = (a_prime_1 * a_prime_1 + b1 * b1).sqrt();
    let c_prime_2 = (a_prime_2 * a_prime_2 + b2 * b2).sqrt();
    let c_bar_prime = (c_prime_1 + c_prime_2) * 0.5;
    let delta_c_prime = c_prime_2 - c_prime_1;
    let delta_big_h_prime =
        2.0 * (c_prime_1 * c_prime_2).sqrt() * (delta_h_prime * 0.5).to_radians().sin();
    let h_bar_prime = if (h_prime_1 - h_prime_2).abs() <= 180.0 {
        (h_prime_1 + h_prime_2) * 0.5
    } else if h_prime_1 + h_prime_2 < 360.0 {
        (h_prime_1 + h_prime_2 + 360.0) * 0.5
    } else {
        (h_prime_1 + h_prime_2 - 360.0) * 0.5
    };
    let t = 1.0 - 0.17 * (h_bar_prime - 30.0).to_radians().cos()
        + 0.24 * (2.0 * h_bar_prime).to_radians().cos()
        + 0.32 * (3.0 * h_bar_prime + 6.0).to_radians().cos()
        - 0.20 * (4.0 * h_bar_prime - 63.0).to_radians().cos();
    let sl = 1.0
        + (0.015 * (l_bar - 50.0) * (l_bar - 50.0))
            / (20.0 + (l_bar - 50.0) * (l_bar - 50.0)).sqrt();
    let sc = 1.0 + 0.045 * c_bar_prime;
    let sh = 1.0 + 0.015 * c_bar_prime * t;
    let rt = -2.0
        * (c_bar_prime.powi(7) / (c_bar_prime.powi(7) + 25i64.pow(7) as f64)).sqrt()
        * (60.0 * (-((h_bar_prime - 275.0) / 25.0) * ((h_bar_prime - 275.0) / 25.0)).exp())
            .to_radians()
            .sin();

    let kl = 1.0;
    let kc = 1.0;
    let kh = 1.0;
    (delta_l_prime / (kl * sl)) * (delta_l_prime / (kl * sl))
        + (delta_c_prime / (kc * sc)) * (delta_c_prime / (kc * sc))
        + (delta_big_h_prime / (kh * sh)) * (delta_big_h_prime / (kh * sh))
        + rt * (delta_c_prime / (kc * sc)) * (delta_big_h_prime / (kh * sh))
}

pub fn next_distinguishable_color(
    colors_to_avoid: &[egui::Color32],
    background_color: egui::Color32,
) -> egui::Color32 {
    const ATTEMPTS: usize = 1000;
    let mut bytes = [0u8; ATTEMPTS * 3];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
        .array_chunks::<3>()
        .map(|bytes| egui::Color32::from_rgb(bytes[0], bytes[1], bytes[2]))
        .filter(|color| color_difference_squared(*color, background_color) >= 50.0 * 50.0)
        .map(|color| {
            let min_distance = colors_to_avoid
                .iter()
                .map(|other| color_difference_squared(color, *other).sqrt())
                .min_by(compare_doubles)
                .unwrap_or(0.0);
            (color, min_distance)
        })
        .max_by(|(_, distance1), (_, distance2)| compare_doubles(distance1, distance2))
        .map(|(color, _)| color)
        .unwrap_or(egui::Color32::WHITE)
}

fn compare_doubles(a: &f64, b: &f64) -> Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => {
            if a == b {
                Ordering::Equal
            } else if a < b {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_approx_eq {
        (($a:expr, $b:expr, $c:expr), $d:expr) => {
            let epsilon = 1e-10;
            let result = $d;
            if (result.0 - $a).abs() >= epsilon
                || (result.1 - $b).abs() >= epsilon
                || (result.2 - $c).abs() >= epsilon
            {
                // will fail
                assert_eq!(($a, $b, $c), result);
            }
        };

        ($a:expr, $b:expr) => {
            let epsilon = 1e-10;
            let result = $b;
            if (result - $a).abs() >= epsilon {
                // will fail
                assert_eq!($a, result);
            }
        };
    }

    macro_rules! assert_color_approx_eq {
        ($a:expr, $b:expr) => {
            let expected = $a;
            let result = $b;
            if result.r().abs_diff(expected.r()) > 1
                || result.g().abs_diff(expected.g()) > 1
                || result.b().abs_diff(expected.b()) > 1
                || result.a().abs_diff(expected.a()) > 1
            {
                // will fail
                assert_eq!(expected, result);
            }
        };
    }

    // http://colormine.org/color-converter

    #[test]
    fn test_rgb_to_lab() {
        assert_approx_eq!((0.0, 0.0, 0.0), rgb_to_lab(egui::Color32::BLACK));
        assert_approx_eq!(
            (100.0, 0.00526049995830391, -0.010408184525267927),
            rgb_to_lab(egui::Color32::WHITE)
        );
        assert_approx_eq!(
            (53.23288178584245, 80.10930952982204, 67.22006831026425),
            rgb_to_lab(egui::Color32::RED)
        );
        assert_approx_eq!(
            (74.90523095018068, -46.11381398481884, 57.589339814050874),
            rgb_to_lab(egui::Color32::from_rgb(127, 204, 69))
        );
    }

    #[test]
    fn test_lab_to_rgb() {
        assert_color_approx_eq!(egui::Color32::BLACK, lab_to_rgb(0.0, 0.0, 0.0));
        assert_color_approx_eq!(
            egui::Color32::WHITE,
            lab_to_rgb(100.0, 0.00526049995830391, -0.010408184525267927)
        );
        assert_color_approx_eq!(
            egui::Color32::RED,
            lab_to_rgb(53.23288178584245, 80.10930952982204, 67.22006831026425)
        );
        assert_color_approx_eq!(
            egui::Color32::from_rgb(127, 204, 69),
            lab_to_rgb(74.90523095018068, -46.11381398481884, 57.589339814050874)
        );
    }

    // http://colormine.org/delta-e-calculator/cie2000 seems to be wrong for some of these, but
    // the reference C# implementation produces the same outputs as us:
    // https://github.com/muak/ColorMinePortable/blob/master/ColorMinePortable/ColorSpaces/Comparisons/CieDe2000Comparison.cs
    #[test]
    fn test_color_difference() {
        assert_approx_eq!(
            0.0,
            color_difference_squared(egui::Color32::BLACK, egui::Color32::BLACK)
        );
        assert_approx_eq!(
            0.0,
            color_difference_squared(egui::Color32::WHITE, egui::Color32::WHITE)
        );
        assert_approx_eq!(
            50.406688151757926,
            color_difference_squared(egui::Color32::RED, egui::Color32::BLACK).sqrt()
        );
        assert_approx_eq!(
            87.86761311947461,
            color_difference_squared(egui::Color32::GREEN, egui::Color32::BLACK).sqrt()
        );
        assert_approx_eq!(
            39.684465473842245,
            color_difference_squared(egui::Color32::BLUE, egui::Color32::BLACK).sqrt()
        );
        assert_approx_eq!(
            101.20334818773345,
            color_difference_squared(egui::Color32::YELLOW, egui::Color32::BLACK).sqrt()
        );
        assert_approx_eq!(
            86.61501290746295,
            color_difference_squared(egui::Color32::RED, egui::Color32::GREEN).sqrt()
        );
        assert_approx_eq!(
            52.878674140461335,
            color_difference_squared(egui::Color32::RED, egui::Color32::BLUE).sqrt()
        );
        assert_approx_eq!(
            23.4034645489725,
            color_difference_squared(egui::Color32::GREEN, egui::Color32::YELLOW).sqrt()
        );
        assert_approx_eq!(
            71.25436904386761,
            color_difference_squared(egui::Color32::RED, egui::Color32::from_rgb(127, 204, 69))
                .sqrt()
        );
    }
}
