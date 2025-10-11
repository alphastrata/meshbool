#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cube, get_mesh_gl, translate};
    use manifold_rs::Manifold;
    use nalgebra::Vector3;
    
    #[test]
    fn test_cube_approx_eq() {
        let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
        let their_cube = Manifold::cube(2.0, 2.0, 2.0);
        
        let result = approx_eq!(&our_cube, &their_cube);
        assert!(result, "Basic cube mesh should be approximately equal");
        
        println!("Cube comparison:");
        let our_mesh_gl = get_mesh_gl(&our_cube, 0);
        let their_mesh_gl = their_cube.to_mesh();
        
        let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
        let their_num_verts = their_mesh_gl.vertices().len() / 3;
        let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
        let their_num_tris = their_mesh_gl.indices().len() / 3;
        
        println!("  Our cube: {} verts, {} tris", our_num_verts, our_num_tris);
        println!("  Their cube: {} verts, {} tris", their_num_verts, their_num_tris);
    }
    
    #[test]
    fn test_translated_cube_approx_eq() {
        let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
        let translated_our_cube = translate(&our_cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
        
        let their_cube = Manifold::cube(2.0, 2.0, 2.0);
        let translated_their_cube = their_cube.translate(1.0, 1.0, 1.0);
        
        let result = approx_eq!(&translated_our_cube, &translated_their_cube);
        assert!(result, "Translated cube meshes should be approximately equal");
        
        println!("Translation comparison successful");
    }
    
    #[test]
    fn test_boolean_union_approx_eq() {
        let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
        let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
        let our_union = &our_cube1 + &our_cube2;
        
        let their_cube1 = Manifold::cube(2.0, 2.0, 2.0);
        let their_cube2 = Manifold::cube(1.0, 1.0, 1.0);
        let their_union = their_cube1.union(&their_cube2);
        
        let result = approx_eq!(&our_union, &their_union);
        assert!(result, "Union of cubes should be approximately equal");
        
        println!("Union comparison:");
        println!("  Our union: {} tris", our_union.num_tri());
        let their_mesh = their_union.to_mesh();
        println!("  Their union: {} tris", their_mesh.indices().len() / 3);
    }
    
    #[test]
    fn test_boolean_intersection_approx_eq() {
        let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
        let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
        let our_intersection = &our_cube1 ^ &our_cube2;
        
        let their_cube1 = Manifold::cube(2.0, 2.0, 2.0);
        let their_cube2 = Manifold::cube(1.0, 1.0, 1.0);
        let their_intersection = their_cube1.intersection(&their_cube2);
        
        let result = approx_eq!(&our_intersection, &their_intersection);
        assert!(result, "Intersection of cubes should be approximately equal");
        
        println!("Intersection comparison:");
        println!("  Our intersection: {} tris", our_intersection.num_tri());
        let their_mesh = their_intersection.to_mesh();
        println!("  Their intersection: {} tris", their_mesh.indices().len() / 3);
    }
    
    #[test]
    fn test_boolean_difference_approx_eq() {
        let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
        let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
        let our_difference = &our_cube1 - &our_cube2;
        
        let their_cube1 = Manifold::cube(2.0, 2.0, 2.0);
        let their_cube2 = Manifold::cube(1.0, 1.0, 1.0);
        let their_difference = their_cube1.difference(&their_cube2);
        
        // Use a higher tolerance for difference operations which can vary more significantly
        let result = approx_eq!(&our_difference, &their_difference, 0.2);
        assert!(result, "Difference of cubes should be approximately equal");
        
        println!("Difference comparison:");
        println!("  Our difference: {} tris", our_difference.num_tri());
        let their_mesh = their_difference.to_mesh();
        println!("  Their difference: {} tris", their_mesh.indices().len() / 3);
    }
}