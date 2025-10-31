/// Convert a MeshBoolImpl to a Bevy Mesh
#[cfg(feature = "bevy")]
pub fn meshbool_to_bevy_mesh(meshbool: &crate::MeshBoolImpl) -> bevy_mesh::Mesh {
    // Get the mesh in the format suitable for graphics libraries
    let mesh_gl = crate::get_mesh_gl(meshbool, 0);
    meshgl_to_bevy_mesh(&mesh_gl)
}

/// Convert a MeshGL to a Bevy Mesh
#[cfg(feature = "bevy")]
pub fn meshgl_to_bevy_mesh(mesh_gl: &crate::MeshGL) -> bevy_mesh::Mesh {
    use bevy_mesh::{Mesh, PrimitiveTopology, VertexAttributeValues};
    use bevy_asset::RenderAssetUsages;
    
    let mut bevy_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default()
    );
    
    // Extract vertex data
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let mut positions = Vec::with_capacity(num_verts);
    let mut normals = Vec::with_capacity(num_verts);
    let mut uvs = Vec::with_capacity(num_verts);
    
    for i in 0..num_verts {
        let offset = i * mesh_gl.num_prop as usize;
        positions.push([
            mesh_gl.vert_properties[offset],
            mesh_gl.vert_properties[offset + 1], 
            mesh_gl.vert_properties[offset + 2]
        ]);
        
        // Extract normals if available (typically after position data)
        if mesh_gl.num_prop >= 6 {
            normals.push([
                mesh_gl.vert_properties[offset + 3],
                mesh_gl.vert_properties[offset + 4], 
                mesh_gl.vert_properties[offset + 5]
            ]);
        } else {
            // Default normal if not provided
            normals.push([0.0, 1.0, 0.0]);
        }
        
        // Extract UVs if available (typically after normals)
        if mesh_gl.num_prop >= 8 {
            uvs.push([
                mesh_gl.vert_properties[offset + 6],
                mesh_gl.vert_properties[offset + 7]
            ]);
        }
    }
    
    // Extract indices
    let indices: Vec<u32> = mesh_gl.tri_verts.clone();
    
    // Insert data into Bevy mesh
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(positions));
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(normals));
    
    if !uvs.is_empty() {
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    }
    
    bevy_mesh.insert_indices(bevy_mesh::Indices::U32(indices));
    
    bevy_mesh
}

/// Convert a Bevy Mesh to a MeshGL
#[cfg(feature = "bevy")]
pub fn bevy_mesh_to_meshgl(bevy_mesh: &bevy_mesh::Mesh) -> Option<crate::MeshGL> {
    // Extract data from Bevy mesh
    let positions = bevy_mesh.attribute(bevy_mesh::Mesh::ATTRIBUTE_POSITION)?;
    let normals = bevy_mesh.attribute(bevy_mesh::Mesh::ATTRIBUTE_NORMAL);
    let uvs = bevy_mesh.attribute(bevy_mesh::Mesh::ATTRIBUTE_UV_0);
    
    let indices = bevy_mesh.indices();
    
    let mut vert_properties = Vec::new();
    let mut tri_verts = Vec::new();
    
    // Determine number of properties per vertex
    let num_prop = {
        let base = 3; // position
        let has_normals = normals.is_some();
        let has_uvs = uvs.is_some();
        base + if has_normals { 3 } else { 0 } + if has_uvs { 2 } else { 0 }
    };
    
    // Process positions
    match positions {
        bevy_mesh::VertexAttributeValues::Float32x3(positions) => {
            for (i, pos) in positions.iter().enumerate() {
                vert_properties.push(pos[0]);
                vert_properties.push(pos[1]);
                vert_properties.push(pos[2]);
                
                // Add normals if available
                if let Some(bevy_mesh::VertexAttributeValues::Float32x3(norms)) = normals {
                    if i < norms.len() {
                        vert_properties.push(norms[i][0]);
                        vert_properties.push(norms[i][1]);
                        vert_properties.push(norms[i][2]);
                    } else {
                        vert_properties.push(0.0);
                        vert_properties.push(1.0);
                        vert_properties.push(0.0);
                    }
                } else if num_prop >= 6 {
                    // Default normal
                    vert_properties.push(0.0);
                    vert_properties.push(1.0);
                    vert_properties.push(0.0);
                }
                
                // Add UVs if available
                if let Some(bevy_mesh::VertexAttributeValues::Float32x2(uv_vals)) = uvs {
                    if i < uv_vals.len() {
                        vert_properties.push(uv_vals[i][0]);
                        vert_properties.push(uv_vals[i][1]);
                    } else {
                        vert_properties.push(0.0);
                        vert_properties.push(0.0);
                    }
                } else if num_prop >= 8 {
                    // Default UV
                    vert_properties.push(0.0);
                    vert_properties.push(0.0);
                }
            }
        },
        _ => return None,
    };
    
    // Process indices
    match indices {
        Some(bevy_mesh::Indices::U32(indices)) => {
            tri_verts.extend_from_slice(indices);
        },
        Some(bevy_mesh::Indices::U16(indices)) => {
            tri_verts.extend(indices.iter().map(|&i| i as u32));
        },
        None => {
            // If no indices, we need to create them based on triangle count
            // For Bevy, vertices are usually already in triangle order
            return None; // We can't create indices without them being provided
        }
    };
    
    Some(crate::MeshGL {
        num_prop,
        vert_properties,
        tri_verts,
        merge_from_vert: Vec::default(),
        merge_to_vert: Vec::default(),
        run_index: Vec::default(),
        run_original_id: Vec::default(),
        run_transform: Vec::default(),
        face_id: Vec::default(),
        tolerance: 1e-6, // Default tolerance
    })
}