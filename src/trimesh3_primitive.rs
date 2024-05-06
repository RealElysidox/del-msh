//! methods that generate meshes of primitive shapes (e.g., cylinder, torus)

use num_traits::AsPrimitive;

/// generate 3D mesh of open cylinder
/// * `radius` - radius
/// * 'length' - length
#[allow(clippy::identity_op)]
pub fn cylinder_open_end_yup<T>(
    ndiv_circumference: usize,
    ndiv_side: usize,
    radius: T,
    length: T,
    is_center: bool)
    -> (nalgebra::Matrix3xX<usize>, nalgebra::Matrix3xX<T>)
    where T: nalgebra::RealField + num_traits::FloatConst + Copy + 'static,
          usize: AsPrimitive<T>
{
    assert!(ndiv_circumference > 2);
    let num_vtx = ndiv_circumference * (ndiv_side + 1);
    let mut vtx2xyz = nalgebra::Matrix3xX::<T>::zeros(num_vtx);
    let two = T::one() + T::one();
    let half = T::one() / two;
    let pi: T = T::PI();
    let dr: T = two * pi / ndiv_circumference.as_();
    let y_min = if is_center {-length * half} else {T::zero()};
    for is in 0..ndiv_side + 1 {
        let y0: T = y_min + length * is.as_() / ndiv_side.as_();
        for ilo in 0..ndiv_circumference {
            let x0 = radius * (dr * ilo.as_()).cos();
            let z0 = radius * (dr * ilo.as_()).sin();
            let i_vtx = is * ndiv_circumference + ilo;
            vtx2xyz.column_mut(i_vtx).copy_from_slice(&[x0, y0, z0]);
        }
    }
    // ------------------------------------
    let num_tri = ndiv_side * ndiv_circumference * 2;
    let mut tri2vtx = nalgebra::Matrix3xX::<usize>::zeros(num_tri);
    for i_side in 0..ndiv_side {
        for i_edge in 0..ndiv_circumference {
            let i0_vtx = i_side * ndiv_circumference + i_edge;
            let i1_vtx = i_side * ndiv_circumference + (i_edge + 1) % ndiv_circumference;
            let i2_vtx = i0_vtx + ndiv_circumference;
            let i3_vtx = i1_vtx + ndiv_circumference;
            let i_quad = i_side * ndiv_circumference + i_edge;
            tri2vtx.column_mut(i_quad * 2 + 0).copy_from_slice(&[i0_vtx, i3_vtx, i1_vtx]);
            tri2vtx.column_mut(i_quad * 2 + 1).copy_from_slice(&[i0_vtx, i2_vtx, i3_vtx]);
        }
    }
    (tri2vtx, vtx2xyz)
}

#[test]
fn test_cylider_open_end_tri3() {
    crate::trimesh3_primitive::cylinder_open_end_yup::<f32>(
        32, 16, 1f32, 1f32, true);
    crate::trimesh3_primitive::cylinder_open_end_yup::<f64>(
        32, 16, 1f64, 1f64, true);
}

// -------------------------------------

pub fn cylinder_open_connecting_two_points<Real>(
    ndiv_circumference: usize,
    r: Real,
    p0: nalgebra::Vector3::<Real>,
    p1: nalgebra::Vector3::<Real>)
    -> (nalgebra::Matrix3xX<usize>, nalgebra::Matrix3xX<Real>)
    where Real: 'static + nalgebra::RealField + Copy + num_traits::FloatConst,
          usize: AsPrimitive<Real>,
            f64: AsPrimitive<Real>
{
    let len = (p1 - p0).norm();
    let (tri2vtx, vtx2xyz0)
        = cylinder_open_end_yup(ndiv_circumference, 1, r, len, false);
    let rot = del_geo::mat3::minimum_rotation_matrix(
        nalgebra::Vector3::<Real>::new(Real::zero(), Real::one(), Real::zero()),
        (p1-p0).normalize());
    let mut vtx2xyz1 = rot * vtx2xyz0;
    for mut xyz in vtx2xyz1.column_iter_mut() { xyz += p0; }
    (tri2vtx, vtx2xyz1)
}


// -------------------------------------

#[allow(clippy::identity_op)]
fn cylinder_like_topology(
    ndiv_side: usize,
    ndiv_circumference: usize) -> Vec<usize>
{
    let ndiv_longtitude = ndiv_side + 2;
    let num_tri = ndiv_circumference * (ndiv_longtitude - 1) * 2 + ndiv_circumference * 2;
    let mut tri2vtx = Vec::<usize>::with_capacity(num_tri * 3);
    for ic in 0..ndiv_circumference {
        tri2vtx.push(0);
        tri2vtx.push((ic + 0) % ndiv_circumference + 1);
        tri2vtx.push((ic + 1) % ndiv_circumference + 1);
    }
    for ih in 0..ndiv_longtitude - 2 {
        for ic in 0..ndiv_circumference {
            let i1 = (ih + 0) * ndiv_circumference + 1 + (ic + 0) % ndiv_circumference;
            let i2 = (ih + 0) * ndiv_circumference + 1 + (ic + 1) % ndiv_circumference;
            let i3 = (ih + 1) * ndiv_circumference + 1 + (ic + 1) % ndiv_circumference;
            let i4 = (ih + 1) * ndiv_circumference + 1 + (ic + 0) % ndiv_circumference;
            tri2vtx.push(i3);
            tri2vtx.push(i2);
            tri2vtx.push(i1);
            tri2vtx.push(i4);
            tri2vtx.push(i3);
            tri2vtx.push(i1);
        }
    }
    for ic in 0..ndiv_circumference {
        tri2vtx.push(ndiv_circumference * (ndiv_longtitude - 1) + 1);
        tri2vtx.push((ndiv_longtitude - 2) * ndiv_circumference + 1 + (ic + 1) % ndiv_circumference);
        tri2vtx.push((ndiv_longtitude - 2) * ndiv_circumference + 1 + (ic + 0) % ndiv_circumference);
    }
    tri2vtx
}

/// generate 3D mesh of closed cylinder
/// * `radius` - radius
/// * 'length' - length
#[allow(clippy::identity_op)]
pub fn cylinder_closed_end_yup<T>(
    radius: T,
    length: T,
    ndiv_circumference: usize,
    ndiv_length: usize,
    is_center: bool)
    -> (nalgebra::Matrix3xX<usize>, nalgebra::Matrix3xX<T>)
    where T: num_traits::FloatConst + 'static + Copy + nalgebra::RealField,
          usize: AsPrimitive<T>
{
    let num_vtx = ndiv_circumference * (ndiv_length + 1) + 2;
    let mut vtx2xyz = nalgebra::Matrix3xX::<T>::zeros(num_vtx);
    assert!( ndiv_length >= 1);
    assert!( ndiv_circumference > 2 );
    let zero = T::zero();
    let two = T::one() + T::one();
    let half = T::one() / two;
    let pi: T = T::PI();
    let dl: T = length / ndiv_length.as_();
    let dr: T = two * pi / ndiv_circumference.as_();
    let y_min = if is_center {-length * half} else {zero};
    // bottom
    vtx2xyz.column_mut(0).copy_from_slice( &[zero, y_min, zero]);
    for il in 0..ndiv_length + 1 {
        let y0 = y_min + dl * il.as_();
        for ilo in 0..ndiv_circumference {
            let x0 = radius * (dr * ilo.as_()).cos();
            let z0 = radius * (dr * ilo.as_()).sin();
            let i_vtx = il * ndiv_circumference + ilo + 1;
            vtx2xyz.column_mut(i_vtx).copy_from_slice(&[x0, y0, z0]);
        }
    }
    // top
    vtx2xyz.column_mut(num_vtx-1).copy_from_slice(&[zero, y_min + length, zero]);
    // ------------------------------------
    let tri2vtx = cylinder_like_topology(ndiv_length, ndiv_circumference);
    let tri2vtx = nalgebra::Matrix3xX::<usize>::from_column_slice(&tri2vtx);
    (tri2vtx, vtx2xyz)
}

#[test]
fn test_cylider_closed_end_tri3() {
    cylinder_closed_end_yup::<f32>(1., 1., 32, 32, true);
    cylinder_closed_end_yup::<f64>(1., 1., 32, 32, true);
}

// ------------------------

#[allow(clippy::identity_op)]
pub fn capsule_yup<T>(
    r: T,
    l: T,
    ndiv_circum: usize,
    ndiv_longtitude: usize,
    ndiv_length: usize) -> (nalgebra::Matrix3xX<usize>, nalgebra::Matrix3xX<T>)
    where T: num_traits::FloatConst + 'static + Copy + nalgebra::RealField,
          f64: AsPrimitive<T>,
          usize: AsPrimitive<T>
{
    let (tri2vtx, mut vtx2xyz) = cylinder_closed_end_yup::<T>(
        T::one(), T::one(),
        ndiv_circum, 2 * ndiv_longtitude + ndiv_length - 2, true);
    assert_eq!(vtx2xyz.len() / 3, (2 * ndiv_longtitude + ndiv_length - 1) * ndiv_circum + 2);
    let pi: T = T::PI();
    let half: T = 0.5.as_();
    {   // South Pole
        vtx2xyz[0] = T::zero();
        vtx2xyz[1] = -l * half - r;
        vtx2xyz[2] = T::zero();
    }
    for ir in 0..ndiv_longtitude {
        let t0 = pi * half * (ndiv_longtitude - 1 - ir).as_() / ndiv_longtitude.as_();
        let y0 = -l * half - r * t0.sin();
        let c0 = r * t0.cos();
        for ic in 0..ndiv_circum {
            let theta = 2.as_() * pi * ic.as_() / ndiv_circum.as_();
            vtx2xyz[(1 + ir * ndiv_circum + ic) * 3 + 0] = c0 * theta.cos();
            vtx2xyz[(1 + ir * ndiv_circum + ic) * 3 + 1] = y0;
            vtx2xyz[(1 + ir * ndiv_circum + ic) * 3 + 2] = c0 * theta.sin();
        }
    }
    for il in 0..ndiv_length - 1 {
        let y0 = -l * half + (il + 1).as_() * l / ndiv_length.as_();
        for ic in 0..ndiv_circum {
            let theta = 2.as_() * pi * ic.as_() / ndiv_circum.as_();
            vtx2xyz[(1 + (il + ndiv_longtitude) * ndiv_circum + ic) * 3 + 0] = r * theta.cos();
            vtx2xyz[(1 + (il + ndiv_longtitude) * ndiv_circum + ic) * 3 + 1] = y0;
            vtx2xyz[(1 + (il + ndiv_longtitude) * ndiv_circum + ic) * 3 + 2] = r * theta.sin();
        }
    }
    for ir in 0..ndiv_longtitude {
        let t0 = pi * half * ir.as_() / ndiv_longtitude.as_();
        let y0 = l * half + r * t0.sin();
        let c0 = r * t0.cos();
        for ic in 0..ndiv_circum {
            let theta = 2.as_() * pi * ic.as_() / ndiv_circum.as_();
            vtx2xyz[(1 + (ir + ndiv_length + ndiv_longtitude - 1) * ndiv_circum + ic) * 3 + 0] = c0 * theta.cos();
            vtx2xyz[(1 + (ir + ndiv_length + ndiv_longtitude - 1) * ndiv_circum + ic) * 3 + 1] = y0;
            vtx2xyz[(1 + (ir + ndiv_length + ndiv_longtitude - 1) * ndiv_circum + ic) * 3 + 2] = c0 * theta.sin();
        }
    }
    {  // North Pole
        let np = vtx2xyz.len() / 3;
        vtx2xyz[(np - 1) * 3 + 0] = T::zero();
        vtx2xyz[(np - 1) * 3 + 1] = l * half + r;
        vtx2xyz[(np - 1) * 3 + 2] = T::zero();
    }
    (tri2vtx, vtx2xyz)
}

#[test]
fn test_capsule_tri3() {
    capsule_yup::<f32>(1., 1., 32, 12, 5);
    capsule_yup::<f64>(1., 1., 32, 12, 5);
}


pub fn capsule_connecting_two_point<T>(
    p0: &[T; 3],
    p1: &[T; 3],
    rad: T,
    ndiv_circum: usize,
    ndiv_longtitude: usize,
    ndiv_length: usize)
    -> (nalgebra::Matrix3xX::<usize>, nalgebra::Matrix3xX::<T>)
    where T: nalgebra::RealField + Copy + num_traits::Float + num_traits::FloatConst,
          f64: AsPrimitive<T>,
          usize: AsPrimitive<T>
{
    let p0 = nalgebra::Vector3::<T>::from_column_slice(p0);
    let p1 = nalgebra::Vector3::<T>::from_column_slice(p1);
    let len = (p1 - p0).norm();
    let (tri2vtx, mut vtx2xyz) = capsule_yup(
        rad, len,
        ndiv_circum, ndiv_longtitude, ndiv_length);
    let q2 = nalgebra::Vector3::<T>::new(T::zero(), len * 0.5_f64.as_(), T::zero());
    let mat = del_geo::mat3::minimum_rotation_matrix(
        nalgebra::Vector3::<T>::new(T::zero(), T::one(), T::zero()),
        (p1 - p0).normalize());
    for mut p in vtx2xyz.column_iter_mut(){
        let q: nalgebra::Vector3::<T> = mat*(q2+p.xyz()) + p0;
        p.copy_from(&q);
    }
    (tri2vtx, vtx2xyz)
}


// --------------------------------------------------------

#[allow(clippy::identity_op)]
pub fn torus_yup<Index, Float>(
    major_radius: Float,
    minor_radius: Float,
    ndiv_major: usize, // latitude
    ndiv_minor: usize) // longtitude
    -> (Vec<Index>, Vec<Float>)
    where Float: num_traits::Float + Default + 'static,
          Index: Default + 'static + Copy,
          f32: AsPrimitive<Float>,
          usize: AsPrimitive<Float> + AsPrimitive<Index>
{
    let rlg: Float = (std::f32::consts::PI * 2_f32).as_() / ndiv_major.as_();  // latitude
    let rlt: Float = (std::f32::consts::PI * 2_f32).as_() / ndiv_minor.as_();
    let mut vtx2xyz: Vec<Float> = vec![Default::default(); ndiv_major * ndiv_minor * 3];
    for ilg in 0..ndiv_major {
        for ilt in 0..ndiv_minor {
            let lt: Float = <usize as AsPrimitive<Float>>::as_(ilt) * rlt;
            let lg: Float = <usize as AsPrimitive<Float>>::as_(ilg) * rlg;
            let r0: Float = major_radius + minor_radius * lt.cos();
            vtx2xyz[(ilg * ndiv_minor + ilt) * 3 + 0] = r0 * lg.sin();
            vtx2xyz[(ilg * ndiv_minor + ilt) * 3 + 1] = r0 * lg.cos();
            vtx2xyz[(ilg * ndiv_minor + ilt) * 3 + 2] = minor_radius * lt.sin();
        }
    }
    let mut tri2vtx: Vec<Index> = vec![Default::default(); ndiv_major * ndiv_minor * 6];
    for ilg in 0..ndiv_major {
        for ilt in 0..ndiv_minor {
            let iug = if ilg == ndiv_major - 1 { 0 } else { ilg + 1 };
            let iut = if ilt == ndiv_minor - 1 { 0 } else { ilt + 1 };
            tri2vtx[(ilg * ndiv_minor + ilt) * 6 + 0] = (ilg * ndiv_minor + ilt).as_();
            tri2vtx[(ilg * ndiv_minor + ilt) * 6 + 2] = (iug * ndiv_minor + ilt).as_();
            tri2vtx[(ilg * ndiv_minor + ilt) * 6 + 1] = (iug * ndiv_minor + iut).as_();
            tri2vtx[(ilg * ndiv_minor + ilt) * 6 + 3] = (ilg * ndiv_minor + ilt).as_();
            tri2vtx[(ilg * ndiv_minor + ilt) * 6 + 5] = (iug * ndiv_minor + iut).as_();
            tri2vtx[(ilg * ndiv_minor + ilt) * 6 + 4] = (ilg * ndiv_minor + iut).as_();
        }
    }
    (tri2vtx, vtx2xyz)
}

#[test]
fn test_torus_tri3() {
    torus_yup::<usize, f32>(1., 1., 32, 32);
    torus_yup::<usize, f64>(1., 1., 32, 32);
}

// --------------

/// the spherical coordinate around y-axis
#[allow(clippy::identity_op)]
pub fn sphere_yup<T>(
    radius: T,
    n_longitude: usize,
    n_latitude: usize) -> (Vec<usize>, Vec<T>)
    where T: num_traits::Float + 'static,
          f32: AsPrimitive<T>,
          usize: AsPrimitive<T>
{
    let mut vtx2xyz = Vec::<T>::new();
    let mut tri2vtx = Vec::<usize>::new();
    vtx2xyz.clear();
    if n_longitude <= 1 || n_latitude <= 2 {
        return (tri2vtx, vtx2xyz);
    }
    let pi: T = std::f32::consts::PI.as_();
    let dl: T = pi / n_longitude.as_();
    let dr: T = 2.as_() * pi / n_latitude.as_();
    vtx2xyz.reserve((n_latitude * (n_longitude - 1) + 2) * 3);
    for ila in 0..n_longitude + 1 {
        let y0 = (dl * ila.as_()).cos();
        let r0 = (dl * ila.as_()).sin();
        for ilo in 0..n_latitude {
            let x0 = r0 * (dr * ilo.as_()).sin();
            let z0 = r0 * (dr * ilo.as_()).cos();
            vtx2xyz.push(radius * x0);
            vtx2xyz.push(radius * y0);
            vtx2xyz.push(radius * z0);
            if ila == 0 || ila == n_longitude { break; }
        }
    }
//
    let ntri = n_latitude * (n_longitude - 1) * 2 + n_latitude * 2;
    tri2vtx.reserve(ntri * 3);

    let tri2vtx = cylinder_like_topology(n_longitude - 2, n_latitude);
    (tri2vtx, vtx2xyz)
}

#[test]
fn test_sphere_yup() {
    let (tri2vtx, vtx2xyz) = sphere_yup::<f64>(1.0, 16, 8);
    crate::io_obj::save_tri2vtx_vtx2xyz(
        "target/sphere_yup.obj",
        &tri2vtx, &vtx2xyz, 3).unwrap();
}

// ----------------------------------------

#[allow(clippy::identity_op)]
pub fn hemisphere_zup<T>(
    radius: T,
    n_longitude: usize,
    n_latitude: usize) -> (Vec<usize>, Vec<T>)
    where T: num_traits::Float + 'static,
          f32: AsPrimitive<T>,
          usize: AsPrimitive<T>
{
    if n_longitude == 0 || n_latitude <= 2 {
        return (vec!(), vec!());
    }
    let pi: T = std::f32::consts::PI.as_();
    let dl: T = 0.5.as_() * pi / n_longitude.as_();
    let dr: T = 2.as_() * pi / n_latitude.as_();
    let nvtx = n_latitude * n_longitude + 1;
    let mut vtx2xyz = Vec::<T>::with_capacity(nvtx * 3);
    for ila in 0..n_longitude + 1 {
        let z0 = (dl * ila.as_()).cos();
        let r0 = (dl * ila.as_()).sin();
        for ilo in 0..n_latitude {
            let x0 = r0 * (dr * ilo.as_()).sin();
            let y0 = r0 * (dr * ilo.as_()).cos();
            vtx2xyz.push(radius * x0);
            vtx2xyz.push(radius * y0);
            vtx2xyz.push(radius * z0);
            if ila == 0 { break; }
        }
    }
    assert_eq!(nvtx * 3, vtx2xyz.len());
    //
    let ntri = n_latitude * (n_longitude - 1) * 2 + n_latitude;
    let mut tri2vtx = Vec::<usize>::with_capacity(ntri * 3);
    for ilo in 0..n_latitude {
        tri2vtx.push(0);
        tri2vtx.push((ilo + 0) % n_latitude + 1);
        tri2vtx.push((ilo + 1) % n_latitude + 1);
    }
    for ilong in 0..n_longitude - 1 {
        for ilat in 0..n_latitude {
            let i1 = (ilong + 0) * n_latitude + 1 + (ilat + 0) % n_latitude;
            let i2 = (ilong + 0) * n_latitude + 1 + (ilat + 1) % n_latitude;
            let i3 = (ilong + 1) * n_latitude + 1 + (ilat + 1) % n_latitude;
            let i4 = (ilong + 1) * n_latitude + 1 + (ilat + 0) % n_latitude;
            tri2vtx.push(i3);
            tri2vtx.push(i2);
            tri2vtx.push(i1);
            tri2vtx.push(i4);
            tri2vtx.push(i3);
            tri2vtx.push(i1);
        }
    }
    assert_eq!(ntri * 3, tri2vtx.len());
    (tri2vtx, vtx2xyz)
}

// ---------------------------

#[allow(clippy::identity_op)]
pub fn bypyramid_zup<Real>(
    length: Real,
    rad_ratio: Real,
    node_ratio: Real) -> (Vec<usize>, Vec<Real>)
    where Real: num_traits::FloatConst + num_traits::Float + 'static,
          usize: AsPrimitive<Real>
{
    let zero = Real::zero();
    let mut vtx2xyz: Vec<Real> = vec!(
        zero, zero, zero,
        zero, zero, length);
    {
        let dt = Real::PI() / (Real::one() + Real::one());
        let r0 = length * rad_ratio;
        for idiv in 0..4 {
            let s0 = r0 * (idiv.as_() * dt).sin();
            let c0 = r0 * (idiv.as_() * dt).cos();
            vtx2xyz.push(s0);
            vtx2xyz.push(c0);
            vtx2xyz.push(length * node_ratio);
        }
    }
    //
    let mut tri2vtx: Vec<usize> = vec!();
    for idiv in 0..4 {
        tri2vtx.push(0);
        tri2vtx.push(2 + (0 + idiv) % 4);
        tri2vtx.push(2 + (1 + idiv) % 4);
        //
        tri2vtx.push(1);
        tri2vtx.push(2 + (1 + idiv) % 4);
        tri2vtx.push(2 + (0 + idiv) % 4);
    }
    (tri2vtx, vtx2xyz)
}

#[test]
fn test_biypyramid_zup() {
    let (tri2vtx, vtx2xyz)
        = bypyramid_zup::<f64>(2.0, 0.2, 0.3);
    crate::io_obj::save_tri2vtx_vtx2xyz(
        "target/bipyramid_zup.obj",
        &tri2vtx, &vtx2xyz, 3).unwrap();
}

// ------------------

#[allow(clippy::identity_op)]
fn arrow_yup<Real>(
    num_division_circumference: usize)
    -> (nalgebra::Matrix3xX<usize>, nalgebra::Matrix3xX<Real>)
    where Real: nalgebra::RealField + num_traits::FloatConst + 'static + Copy,
          f64: AsPrimitive<Real>,
          usize: AsPrimitive<Real>
{
    let dr: Real = Real::PI() * 2f64.as_() / num_division_circumference.as_();
    let stem_height: Real = 0.65f64.as_();
    let radius_small: Real = 0.1.as_();
    let radius_large: Real = 0.2.as_();
    let (tri2vtx, mut vtx2xyz)
        = cylinder_closed_end_yup(Real::one(), Real::one(),
                                  num_division_circumference, 2, true);
    assert_eq!(vtx2xyz.len(), (2 + 3 * num_division_circumference) * 3);
    vtx2xyz[0] = Real::zero();
    vtx2xyz[1] = Real::zero();
    vtx2xyz[2] = Real::zero();
    let height_rad = [
        (Real::zero(), radius_small),
        (stem_height, radius_small),
        (stem_height, radius_large)];
    for (il, (height, rad)) in height_rad.iter().enumerate() {
        for ilo in 0..num_division_circumference {
            let theta: Real = dr * ilo.as_();
            let x0 = theta.cos() * *rad;
            let z0 = theta.sin() * *rad;
            vtx2xyz[(1 + il * num_division_circumference + ilo) * 3 + 0] = x0;
            vtx2xyz[(1 + il * num_division_circumference + ilo) * 3 + 1] = *height;
            vtx2xyz[(1 + il * num_division_circumference + ilo) * 3 + 2] = z0;
        }
    }
    let n = 1 + 3 * num_division_circumference;
    vtx2xyz[n * 3 + 0] = Real::zero();
    vtx2xyz[n * 3 + 1] = Real::one();
    vtx2xyz[n * 3 + 2] = Real::zero();
    (tri2vtx, vtx2xyz)
}


#[test]
fn test_arrow_zup() {
    let (tri2vtx, vtx2xyz) = arrow_yup::<f64>(16);
    crate::io_obj::save_tri2vtx_vtx2xyz(
        "target/arrow_zup.obj",
        tri2vtx.as_slice(), vtx2xyz.as_slice(), 3).unwrap();
}

// --------------------------------

pub fn arrow_connecting_two_points<T>(
    p0: &[T; 3],
    p1: &[T; 3],
    num_division_circumference: usize)
    -> (nalgebra::Matrix3xX<usize>, nalgebra::Matrix3xX<T>)
    where T: nalgebra::RealField + Copy + num_traits::Float + num_traits::FloatConst,
          f64: AsPrimitive<T>,
          usize: AsPrimitive<T>
{
    let p0 = nalgebra::Vector3::<T>::from_column_slice(p0);
    let p1 = nalgebra::Vector3::<T>::from_column_slice(p1);
    let len = (p1 - p0).norm();
    let (tri2vtx, mut vtx2xyz) = arrow_yup(num_division_circumference);
    let mat = del_geo::mat3::minimum_rotation_matrix(
        nalgebra::Vector3::<T>::new(T::zero(), T::one(), T::zero()),
        (p1 - p0).normalize()).scale(len);
    for mut v in vtx2xyz.column_iter_mut() {
        let q1: nalgebra::Vector3::<T> = mat * v.xyz() + p0;
        v.copy_from(&q1);
    }
    (tri2vtx, vtx2xyz)
}

#[test]
fn test_arrow_connecting_two_points() {
    let (tri2vtx, vtx2xyz) = arrow_connecting_two_points::<f64>(
        &[1.0, 1.0, 1.0],
        &[1.0, 1.0, 2.0], 16);
    crate::io_obj::save_tri2vtx_vtx2xyz(
        "target/arrow_connecting_two_points.obj",
        tri2vtx.as_slice(), vtx2xyz.as_slice(), 3).unwrap();
}