use meshbool::MeshBool;
use nalgebra::{Vector3, Point3};

#[test]
fn test_meshbool_basic_functionality() {
    // Test basic cube creation
    let cube1 = MeshBool::cube(Vector3::new(1.0, 1.0, 1.0), true);
    println!("Cube1: {} triangles, {} vertices", cube1.num_tri(), cube1.num_vert());
    assert!(cube1.num_tri() > 0);
    assert!(cube1.num_vert() > 0);
    
    // Test translation
    let translated = cube1.translate(Point3::new(2.0, 0.0, 0.0));
    println!("Translated: {} triangles, {} vertices", translated.num_tri(), translated.num_vert());
    assert!(translated.num_tri() > 0);
    assert!(translated.num_vert() > 0);
    
    // Test boolean operations
    let cube2 = MeshBool::cube(Vector3::new(1.0, 1.0, 1.0), true);
    let union = &cube1 + &cube2;
    println!("Union: {} triangles, {} vertices", union.num_tri(), union.num_vert());
    assert!(union.num_tri() > 0);
    assert!(union.num_vert() > 0);
    
    // Test that we can get mesh data
    let mesh_gl = meshbool::get_mesh_gl(&cube1, 0);
    println!("MeshGL: {} vertices, {} triangles", mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize, mesh_gl.tri_verts.len() / 3);
    assert!(!mesh_gl.vert_properties.is_empty());
    assert!(!mesh_gl.tri_verts.is_empty());
    
    println!("✅ All basic functionality tests passed!");
}