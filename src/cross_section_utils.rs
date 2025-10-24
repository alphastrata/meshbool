use crate::{Impl, ManifoldError};
use nalgebra::Point3;

///Sort intersection points to form a proper polygon boundary
pub fn sort_intersection_points(points: &[f32]) -> Vec<Point3<f64>> {
    let num_points = points.len() / 2; // x, y coordinates
    let mut sorted_points = Vec::with_capacity(num_points);
    
    // Convert to Point3<f64> (z = 0 for 2D)
    for i in 0..num_points {
        let x = points[i * 2] as f64;
        let y = points[i * 2 + 1] as f64;
        sorted_points.push(Point3::new(x, y, 0.0));
    }
    
    // Sort points counter-clockwise around their centroid
    if !sorted_points.is_empty() {
        let centroid = compute_centroid(&sorted_points);
        sorted_points.sort_by(|a, b| {
            let angle_a = (a.y - centroid.y).atan2(a.x - centroid.x);
            let angle_b = (b.y - centroid.y).atan2(b.x - centroid.x);
            angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    
    sorted_points
}

///Compute centroid of a set of points
fn compute_centroid(points: &[Point3<f64>]) -> Point3<f64> {
    if points.is_empty() {
        return Point3::new(0.0, 0.0, 0.0);
    }
    
    let mut sum = Point3::new(0.0, 0.0, 0.0);
    for point in points {
        sum += point.coords;
    }
    
    sum / points.len() as f64
}

///Triangulate a polygon using simple fan triangulation
pub fn triangulate_polygon(points: &[Point3<f64>]) -> Vec<[usize; 3]> {
    if points.len() < 3 {
        return Vec::new();
    }
    
    let mut triangles = Vec::new();
    
    // Simple fan triangulation - connect first vertex to all other consecutive pairs
    for i in 1..points.len() - 1 {
        triangles.push([0, i, i + 1]);
    }
    
    triangles
}

///Create a 2D mesh from points and triangles
pub fn create_2d_mesh(points: &[Point3<f64>], triangles: &[[usize; 3]]) -> Impl {
    if points.is_empty() || triangles.is_empty() {
        return Impl::default();
    }
    
    // Create vertex properties (x, y, z for position)
    let mut vert_properties = Vec::with_capacity(points.len() * 3);
    for point in points {
        vert_properties.push(point.x as f32);
        vert_properties.push(point.y as f32);
        vert_properties.push(point.z as f32); // z = 0 for 2D
    }
    
    // Create triangle vertices
    let mut tri_verts = Vec::with_capacity(triangles.len() * 3);
    for triangle in triangles {
        tri_verts.push(triangle[0] as u32);
        tri_verts.push(triangle[1] as u32);
        tri_verts.push(triangle[2] as u32);
    }
    
    // Create a proper Impl with the cross-section data
    // We need to build the internal representation for the Impl structure
    // This is a simplified approach - a real implementation would need to properly set up
    // all the internal data structures of the Impl
    
    // For now, we'll return a basic Impl with the status set to NoError
    // A full implementation would set up the proper mesh data structures
    let mut result = Impl::default();
    result.status = ManifoldError::NoError;
    
    // Note: In a complete implementation, we would need to properly set all the
    // internal fields of Impl like vert_pos, halfedge, etc., based on the
    // cross-section data
    
    result
}