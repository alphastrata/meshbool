use crate::MeshGL;

/// Helper function to compute cross-section of a mesh at given Z height
/// Returns vertices of intersection points and indices to form polygons
pub fn compute_cross_section(mesh_gl: &MeshGL, height: f64) -> (Vec<f32>, Vec<u32>) {
    let mut intersection_points = Vec::new();
    let mut polygon_indices = Vec::new();
    
    // Map to track intersection points to avoid duplicates
    let mut point_map = std::collections::HashMap::new();
    
    // Iterate through all triangles
    for i in (0..mesh_gl.tri_verts.len()).step_by(3) {
        let v0_idx = mesh_gl.tri_verts[i] as usize;
        let v1_idx = mesh_gl.tri_verts[i + 1] as usize;
        let v2_idx = mesh_gl.tri_verts[i + 2] as usize;
        
        // Get vertex positions (assuming first 3 properties are x, y, z)
        let v0_x = mesh_gl.vert_properties[v0_idx * mesh_gl.num_prop as usize] as f64;
        let v0_y = mesh_gl.vert_properties[v0_idx * mesh_gl.num_prop as usize + 1] as f64;
        let v0_z = mesh_gl.vert_properties[v0_idx * mesh_gl.num_prop as usize + 2] as f64;
        
        let v1_x = mesh_gl.vert_properties[v1_idx * mesh_gl.num_prop as usize] as f64;
        let v1_y = mesh_gl.vert_properties[v1_idx * mesh_gl.num_prop as usize + 1] as f64;
        let v1_z = mesh_gl.vert_properties[v1_idx * mesh_gl.num_prop as usize + 2] as f64;
        
        let v2_x = mesh_gl.vert_properties[v2_idx * mesh_gl.num_prop as usize] as f64;
        let v2_y = mesh_gl.vert_properties[v2_idx * mesh_gl.num_prop as usize + 1] as f64;
        let v2_z = mesh_gl.vert_properties[v2_idx * mesh_gl.num_prop as usize + 2] as f64;
        
        // Check for triangle-Z plane intersection
        let mut intersections = Vec::new();
        
        // Check edge v0-v1
        if let Some(intersection) = intersect_edge_with_plane(v0_x, v0_y, v0_z, v1_x, v1_y, v1_z, height) {
            intersections.push(intersection);
        }
        
        // Check edge v1-v2  
        if let Some(intersection) = intersect_edge_with_plane(v1_x, v1_y, v1_z, v2_x, v2_y, v2_z, height) {
            intersections.push(intersection);
        }
        
        // Check edge v2-v0
        if let Some(intersection) = intersect_edge_with_plane(v2_x, v2_y, v2_z, v0_x, v0_y, v0_z, height) {
            intersections.push(intersection);
        }
        
        // If we have 2 intersection points, add them to our polygon
        if intersections.len() == 2 {
            // Add first point
            let p0_key = format!("{:.6}_{:.6}", intersections[0].0, intersections[0].1);
            let p0_idx = if let Some(&idx) = point_map.get(&p0_key) {
                idx
            } else {
                let idx = intersection_points.len() / 2;
                intersection_points.push(intersections[0].0 as f32);
                intersection_points.push(intersections[0].1 as f32);
                point_map.insert(p0_key, idx);
                idx
            };
            
            // Add second point
            let p1_key = format!("{:.6}_{:.6}", intersections[1].0, intersections[1].1);
            let p1_idx = if let Some(&idx) = point_map.get(&p1_key) {
                idx
            } else {
                let idx = intersection_points.len() / 2;
                intersection_points.push(intersections[1].0 as f32);
                intersection_points.push(intersections[1].1 as f32);
                point_map.insert(p1_key, idx);
                idx
            };
            
            polygon_indices.push(p0_idx as u32);
            polygon_indices.push(p1_idx as u32);
        }
    }
    
    (intersection_points, polygon_indices)
}

/// Helper function to compute intersection of edge with Z-plane
/// Returns None if no intersection or if intersection is at endpoint
fn intersect_edge_with_plane(x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64, height: f64) -> Option<(f64, f64)> {
    // Check if edge crosses the plane
    let z_diff = z1 - z0;
    
    // Avoid division by zero
    if z_diff.abs() < 1e-10 {
        return None;
    }
    
    let t = (height - z0) / z_diff;
    
    // Check if intersection is within edge bounds (excluding endpoints to avoid duplicates)
    if t <= 0.0 || t >= 1.0 {
        return None;
    }
    
    // Compute intersection point
    let x = x0 + t * (x1 - x0);
    let y = y0 + t * (y1 - y0);
    
    Some((x, y))
}