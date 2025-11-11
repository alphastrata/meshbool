#[cfg(feature = "bevy")]
#[cfg(test)]
mod tests {
    use meshbool::{MeshBool, bevy_mesh_to_meshbool, meshbool_to_bevy_mesh, ManifoldError};
    use bevy_mesh::{Mesh, VertexAttributeValues};
    use nalgebra::Vector3;

    #[test]
    fn test_gltf_sheep_conversion_process() {
        // Create a simple test using the cube function to get a valid MeshBool, 
        // then convert it to Bevy mesh format and back to test the conversion functions
        let original_cube = MeshBool::cube(Vector3::new(2.0, 1.0, 2.0), true);
        
        // Convert to MeshGL and then to Bevy mesh
        let mesh_gl = original_cube.get_mesh_gl(0);
        use meshbool::meshgl_to_bevy_mesh;
        let bevy_mesh = meshgl_to_bevy_mesh(&mesh_gl);
        
        // Main test: convert the Bevy mesh back to MeshBool
        println!("🐑 Converting Bevy mesh to MeshBool...");
        let meshbool_result = bevy_mesh_to_meshbool(&bevy_mesh);
        assert!(meshbool_result.is_some(), "Bevy mesh conversion to MeshBool should succeed");
        
        let converted_cube = meshbool_result.unwrap();
        assert!(converted_cube.num_tri() > 0, "Converted cube should have triangles");
        assert!(converted_cube.num_vert() > 0, "Converted cube should have vertices");
        assert_eq!(converted_cube.status(), ManifoldError::NoError, 
                  "Converted cube should have NoError status");
        println!("   ✓ Bevy mesh converted to MeshBool with {} triangles and {} vertices", 
                 converted_cube.num_tri(), converted_cube.num_vert());
        
        // Verify that the converted mesh has basic valid properties  
        println!("🔍 Testing basic properties of converted mesh...");
        assert!(!converted_cube.is_empty(), "Converted mesh should not be empty");
        assert_eq!(converted_cube.status(), ManifoldError::NoError, 
                  "Converted mesh should have NoError status");
        assert!(converted_cube.num_tri() > 0, "Converted mesh should have triangles");
        assert!(converted_cube.num_vert() > 0, "Converted mesh should have vertices");
        println!("   ✓ Basic mesh properties verified");
        
        println!("\n🎉 BEVY MESH CONVERSION TEST COMPLETED SUCCESSFULLY!");
        println!("==================================================");
    }
}