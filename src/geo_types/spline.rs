use geo::Coord;

use crate::errors::SplineCreationError;

struct Hobby {}

fn coord_length(c: Coord) -> f64 {
    return (c.x * c.x + c.y * c.y).sqrt();
}

fn normalize_coord(c: Coord) -> Coord {
    let l = coord_length(c);
    Coord {
        x: c.x / l,
        y: c.y / l,
    }
}

// fn scale_coord(c: Coord, s: f64) -> Coord {
//     Coord {
//         x: c.x * s,
//         y: c.y * s,
//     }
// }

fn coord_rot(c: Coord, angle: f64) -> Coord {
    let ca = angle.cos();
    let sa = angle.sin();
    Coord {
        x: c.x * ca - c.y * sa,
        y: c.x * sa + c.y * ca,
    }
}

fn angle_between_vecs(v: Coord, w: Coord) -> f64 {
    (w.y * v.x - w.x * v.y).atan2(v.x * w.x + v.y * w.y)
    // return atan2(w[1] * v[0] - w[0] * v[1], v[0] * w[0] + v[1] * w[1]);
}
/// Takes a set of control points, and an omega value, and returns
/// a Vec of coords which can be used to construct a Bézier spline.
/// Returns a SplineCreationError if you bunged up the input data.
pub fn hobby_points(coords: &Vec<Coord>, omega: f64) -> Result<Vec<Coord>, SplineCreationError> {
    if coords.len() < 2 {
        return Err(SplineCreationError::InvalidInputGeometry);
    };

    let n = coords.len() - 1;

    let mut chords = vec![Coord { x: 0., y: 0. }; n]; //Vec:: with_capacity(n);
    let mut d: Vec<f64> = vec![0.; n];
    for i in 0..n {
        chords[i] = coords[i + 1] - coords[i];
        println!(
            "Cn+1: {:?}, Cn: {:?} => Chordn:{:?}",
            coords[i + 1],
            coords[i],
            chords[i]
        );
        d[i] = coord_length(chords[i]); //.magnitude();
        assert!(d[i] > 0.);
    }

    let mut gamma = vec![0.; n + 1];
    for i in 1..n {
        gamma[i] = angle_between_vecs(chords[i - 1], chords[i]);
    }
    gamma[n] = 0.;

    let (mut a_vec, mut b_vec, mut c_vec, mut d_vec) = (
        vec![0.0_f64; n + 1],
        vec![0.0_f64; n + 1],
        vec![0.0_f64; n + 1],
        vec![0.0_f64; n + 1],
    );

    b_vec[0] = 2. + omega;
    c_vec[0] = 2. + omega + 1.;
    d_vec[0] = -1. * c_vec[0] * gamma[1];

    for i in 1..n {
        //(let i = 1; i < n; i++) {
        a_vec[i] = 1. / d[i - 1];
        b_vec[i] = (2. * d[i - 1] + 2. * d[i]) / (d[i - 1] * d[i]);
        c_vec[i] = 1. / d[i];
        d_vec[i] = (-1. * (2. * gamma[i] * d[i] + gamma[i + 1] * d[i - 1])) / (d[i - 1] * d[i]);
    }

    a_vec[n] = 2. * omega + 1.;
    b_vec[n] = 2. + omega;
    d_vec[n] = 0.;

    println!("ABCD: {:?}, {:?}, {:?}, {:?}", a_vec, b_vec, c_vec, d_vec);

    let alpha = thomas(a_vec, b_vec, c_vec, d_vec);

    let mut beta: Vec<f64> = vec![0.; n];

    for i in 0..(n - 1) {
        beta[i] = -1. * gamma[i + 1] - alpha[i + 1]
    }

    beta[n - 1] = -1. * alpha[n];

    let mut c0: Vec<Coord> = vec![Coord { x: 0., y: 0. }; n]; //Vec::with_capacity(n);
    let mut c1: Vec<Coord> = vec![Coord { x: 0., y: 0. }; n]; //Vec::with_capacity(n);

    for i in 0..n {
        let a = (rho(alpha[i], beta[i]) * d[i]) / 3.;
        let b = (rho(beta[i], alpha[i]) * d[i]) / 3.;

        c0[i] = coords[i] + normalize_coord(coord_rot(chords[i], alpha[i])) * a;
        c1[i] = coords[i + 1] - normalize_coord(coord_rot(chords[i], -1. * beta[i])) * b;
    }

    let mut res: Vec<Coord> = Vec::with_capacity(n + 1); //vec![Coord { x: 0., y: 0. }; n + 1];

    for i in 0..n {
        // res.append(vec![points[i], c0[i], c1[i]]);
        res.append(&mut vec![coords[i], c0[i], c1[i]]);
    }

    res.push(coords[n]);
    Ok(res)
}

fn rho(alpha: f64, beta: f64) -> f64 {
    let c = 2. / 3.;
    2. / (1. + c * beta.cos() + (1. - c) * alpha.cos())
}

fn thomas(avec: Vec<f64>, bvec: Vec<f64>, cvec: Vec<f64>, dvec: Vec<f64>) -> Vec<f64> {
    let n = bvec.len() - 1;
    let mut c_p: Vec<f64> = vec![0.; n + 1]; //Vec::with_capacity(n + 1);
    let mut d_p: Vec<f64> = vec![0.; n + 1]; //Vec::with_capacity(n + 1);

    c_p[0] = cvec[0] / bvec[0];
    d_p[0] = dvec[0] / bvec[0];

    for i in 1..=n {
        //(let i = 1; i <= n; i++) {
        let denom = bvec[i] - c_p[i - 1] * avec[i];
        c_p[i] = cvec[i] / denom;
        d_p[i] = (dvec[i] - d_p[i - 1] * avec[i]) / denom;
    }

    let mut x_vec = vec![0.; n + 1];

    x_vec[n] = d_p[n];
    for i in (0..=(n - 1)).rev() {
        x_vec[i] = d_p[i] - c_p[i] * x_vec[i + 1];
    }
    return x_vec;
}

#[cfg(test)]
pub mod test {
    use geo::{Coord, Vector2DOps};

    use crate::geo_types::spline::hobby_points;

    use super::angle_between_vecs;

    #[test]
    fn test_norm() {
        println!(
            "Norm 5,0: {:?}",
            super::normalize_coord(Coord { x: 5., y: 0. })
        );
        println!(
            "Norm 3,4: {:?}",
            super::normalize_coord(Coord { x: 3., y: 4. })
        );
    }

    #[test]
    fn test_vadd() {
        println!("CLen: {:?}", Coord { x: 2., y: 3. });
        let c = Coord { x: 5., y: 5. };
        println!("C+C: {:?}", c + c);
        println!("VMUL: {:?} ", c * 5.);
        println!("VSUB should be 3,4: {:?}", c - Coord { x: 2., y: 1. });
    }

    #[test]
    fn test_vscale() {
        let c = Coord { x: 5., y: 5. };
        println!("C*3.: {:?}", c * 3.);
    }

    #[test]
    fn test_angle_between() {
        println!(
            "should be 45: {}",
            angle_between_vecs(Coord { x: 5., y: 0. }, Coord { x: 5., y: 5. })
        );
        println!(
            "should be 90: {}",
            angle_between_vecs(Coord { x: 5., y: 0. }, Coord { x: 0., y: 5. })
        );
    }

    #[test]
    fn test_simple_spline() {
        let coords = vec![
            Coord { x: 1., y: 0. },
            Coord { x: 10., y: 0. },
            Coord { x: 10., y: 10. },
            Coord { x: 20., y: 10. },
            Coord { x: 20., y: 0. },
        ];
        println!("Spline out: {:?}", hobby_points(&coords, 0.5).unwrap());
    }
}
