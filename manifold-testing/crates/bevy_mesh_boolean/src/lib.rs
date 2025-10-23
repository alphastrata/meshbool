use bevy::prelude::*;
use manifold_rs;
use std::panic;

/// Bundle containing all components needed for a boolean operation entity
#[derive(Bundle)]
pub struct BooleanEntityBundle {
    pub pbr: PbrBundle,
    pub visibility: Visibility,
}

impl BooleanEntityBundle {
    pub fn new(mesh: Handle<Mesh>, material: Handle<StandardMaterial>, transform: Transform) -> Self {
        Self {
            pbr: PbrBundle {
                mesh,
                material,
                transform,
                ..default()
            },
            visibility: Visibility::default(),
        }
    }
}

// The core plugin struct
pub struct MeshBooleanPlugin;

impl MeshBooleanPlugin {
    /// Spawns two entities with a boolean operation between them
    pub fn spawn_boolean_operation(
        commands: &mut Commands,
        _meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        primary_mesh: Handle<Mesh>,
        secondary_mesh: Handle<Mesh>,
        primary_transform: Transform,
        secondary_transform: Transform,
        result_material: Handle<StandardMaterial>,
    ) -> BooleanOperationBundle {
        // Spawn the primary mesh entity
        let primary_entity = commands
            .spawn(BooleanEntityBundle::new(
                primary_mesh.clone(),
                materials.add(Color::srgb(0.8, 0.7, 0.6)),
                primary_transform,
            ))
            .insert(PrimaryBooleanMesh {
                secondary_entity: Entity::PLACEHOLDER,
            })
            .id();

        // Spawn the secondary mesh entity
        let secondary_entity = commands
            .spawn(BooleanEntityBundle::new(
                secondary_mesh.clone(),
                materials.add(Color::srgb(0.6, 0.7, 0.8)),
                secondary_transform,
            ))
            .insert(SecondaryBooleanMesh {
                primary_entity,
            })
            .id();

        // Spawn the result entity (initially hidden)
        let result_entity = commands
            .spawn(BooleanEntityBundle::new(
                primary_mesh, // Placeholder, will be replaced
                result_material,
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        // Update the primary entity to reference the secondary
        commands.entity(primary_entity).insert(PrimaryBooleanMesh {
            secondary_entity,
        });

        // Insert the handles resource to track all entities
        commands.insert_resource(BooleanHandles {
            primary_entity,
            secondary_entity,
            result_entity,
        });

        BooleanOperationBundle {
            primary: primary_entity,
            secondary: secondary_entity,
            result: result_entity,
        }
    }
}

/// A bundle that represents a complete boolean operation setup
pub struct BooleanOperationBundle {
    pub primary: Entity,
    pub secondary: Entity,
    pub result: Entity,
}

impl Plugin for MeshBooleanPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BooleanOpState>()
            .add_systems(Update, apply_boolean_op);
    }
}

// Resource to control the boolean operation
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOpState {
    #[default]
    None,
    Intersect,
    Union,
    Subtract,
}

// Component to mark the primary entity in a boolean operation
#[derive(Component)]
pub struct PrimaryBooleanMesh {
    pub secondary_entity: Entity,
}

// Component to mark the secondary entity in a boolean operation
#[derive(Component)]
pub struct SecondaryBooleanMesh {
    pub primary_entity: Entity,
}

// Resource to hold entity handles for boolean operations
#[derive(Resource)]
pub struct BooleanHandles {
    pub primary_entity: Entity,
    pub secondary_entity: Entity,
    pub result_entity: Entity,
}

// The system that applies the boolean operation
fn apply_boolean_op(
    mut commands: Commands,
    boolean_handles: Option<Res<BooleanHandles>>,
    pbr_query: Query<(&Handle<Mesh>, &Transform)>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut visibility_query: Query<&mut Visibility>,
    op_state: Res<BooleanOpState>,
) {
    let operation_start_time = std::time::Instant::now();
    
    if !op_state.is_changed() {
        return;
    }

    eprintln!("[TIMING {}] Boolean operation triggered. State: {:?}", 
             operation_start_time.elapsed().as_micros(), *op_state);
    
    let handles = match boolean_handles {
        Some(h) => h,
        None => {
            eprintln!("[TIMING {}] No BooleanHandles resource found, skipping operation", 
                     operation_start_time.elapsed().as_micros());
            // Just return if no handles exist yet - this can happen during startup
            // before the demo is fully set up
            return;
        },
    };

    let primary_entity = handles.primary_entity;
    let secondary_entity = handles.secondary_entity;
    let result_entity = handles.result_entity;

    eprintln!("[TIMING {}] Processing boolean operation on entities - Primary: {:?}, Secondary: {:?}, Result: {:?}", 
             operation_start_time.elapsed().as_micros(), primary_entity, secondary_entity, result_entity);

    // Get primary mesh data
    let (primary_mesh_handle, primary_transform) = {
        if let Ok((mesh_handle, transform)) = pbr_query.get(primary_entity) {
            eprintln!("[TIMING {}] Found primary mesh handle and transform", 
                     operation_start_time.elapsed().as_micros());
            (mesh_handle.clone(), *transform)
        } else {
            eprintln!("[TIMING {}] [ERROR] Could not get primary mesh data for entity {:?}", 
                     operation_start_time.elapsed().as_micros(), primary_entity);
            return;
        }
    };

    // Get secondary mesh data
    let (secondary_mesh_handle, secondary_transform) = {
        if let Ok((mesh_handle, transform)) = pbr_query.get(secondary_entity) {
            eprintln!("[TIMING {}] Found secondary mesh handle and transform", 
                     operation_start_time.elapsed().as_micros());
            (mesh_handle.clone(), *transform)
        } else {
            eprintln!("[TIMING {}] [ERROR] Could not get secondary mesh data for entity {:?}", 
                     operation_start_time.elapsed().as_micros(), secondary_entity);
            return;
        }
    };

    // Get actual mesh assets
    let primary_mesh = match mesh_assets.get(&primary_mesh_handle) {
        Some(mesh) => {
            eprintln!("[TIMING {}] Retrieved primary mesh asset", 
                     operation_start_time.elapsed().as_micros());
            mesh
        },
        None => {
            eprintln!("[TIMING {}] [ERROR] Could not retrieve primary mesh asset", 
                     operation_start_time.elapsed().as_micros());
            return;
        }
    };

    let secondary_mesh = match mesh_assets.get(&secondary_mesh_handle) {
        Some(mesh) => {
            eprintln!("[TIMING {}] Retrieved secondary mesh asset", 
                     operation_start_time.elapsed().as_micros());
            mesh
        },
        None => {
            eprintln!("[TIMING {}] [ERROR] Could not retrieve secondary mesh asset", 
                     operation_start_time.elapsed().as_micros());
            return;
        }
    };

    // Log mesh statistics
    let primary_vertex_count = if let Some(positions) = primary_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
            _ => 0,
        }
    } else {
        0
    };
    
    let secondary_vertex_count = if let Some(positions) = secondary_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
            _ => 0,
        }
    } else {
        0
    };
    
    eprintln!("[TIMING {}] Mesh statistics - Primary: {} vertices, Secondary: {} vertices", 
             operation_start_time.elapsed().as_micros(), primary_vertex_count, secondary_vertex_count);

    // If no operation, show original shapes
    if *op_state == BooleanOpState::None {
        eprintln!("[TIMING {}] No operation selected, showing original shapes", 
                 operation_start_time.elapsed().as_micros());
        if let Ok(mut primary_vis) = visibility_query.get_mut(primary_entity) {
            *primary_vis = Visibility::Visible;
        }
        if let Ok(mut secondary_vis) = visibility_query.get_mut(secondary_entity) {
            *secondary_vis = Visibility::Visible;
        }
        if let Ok(mut result_vis) = visibility_query.get_mut(result_entity) {
            *result_vis = Visibility::Hidden;
        }
        return;
    }

    // Hide original shapes and show result
    eprintln!("[TIMING {}] Hiding original shapes and preparing for boolean operation", 
             operation_start_time.elapsed().as_micros());
    if let Ok(mut primary_vis) = visibility_query.get_mut(primary_entity) {
        *primary_vis = Visibility::Hidden;
    }
    if let Ok(mut secondary_vis) = visibility_query.get_mut(secondary_entity) {
        *secondary_vis = Visibility::Hidden;
    }

    // Try to convert Bevy meshes to manifold-rs manifolds
    eprintln!("[TIMING {}] Attempting to convert Bevy meshes to manifold-rs manifolds...", 
             operation_start_time.elapsed().as_micros());
    
    let start_time = std::time::Instant::now();
    
    // Try actual conversion first
    let primary_manifold_opt = bevy_mesh_to_manifold(primary_mesh);
    let secondary_manifold_opt = bevy_mesh_to_manifold(secondary_mesh);
    
    let conversion_time = start_time.elapsed();
    eprintln!("[TIMING {}] [CONVERSION] Mesh conversion took: {:?}", 
             operation_start_time.elapsed().as_micros(), conversion_time);
    
    match (primary_manifold_opt, secondary_manifold_opt) {
        (Some(mut primary_manifold), Some(mut secondary_manifold)) => {
            eprintln!("[TIMING {}] [SUCCESS] Successfully converted both meshes to manifolds", 
                     operation_start_time.elapsed().as_micros());
            
            // Apply transformations to position the shapes for intersection
            let primary_pos = primary_transform.translation;
            let secondary_pos = secondary_transform.translation;

            eprintln!("[TIMING {}] Applying transformations - Primary: {:?}, Secondary: {:?}", 
                     operation_start_time.elapsed().as_micros(), primary_pos, secondary_pos);
            
            primary_manifold = primary_manifold.translate(
                primary_pos.x as f64,
                primary_pos.y as f64,
                primary_pos.z as f64,
            );
            secondary_manifold = secondary_manifold.translate(
                secondary_pos.x as f64,
                secondary_pos.y as f64,
                secondary_pos.z as f64,
            );

            // Log initial mesh info
            let prim_mesh_info = primary_manifold.to_mesh();
            let sec_mesh_info = secondary_manifold.to_mesh();
            let prim_vertices_before = prim_mesh_info.vertices().len();
            let sec_vertices_before = sec_mesh_info.vertices().len();
            eprintln!("[TIMING {}] Boolean operation: Primary vertices: {}, Secondary vertices: {}", 
                     operation_start_time.elapsed().as_micros(), prim_vertices_before, sec_vertices_before);

            // Perform boolean operation
            let operation_start = std::time::Instant::now();
            let result_manifold = match *op_state {
                BooleanOpState::Intersect => {
                    eprintln!("[TIMING {}] Performing intersection operation...", 
                             operation_start_time.elapsed().as_micros());
                    // Wrap in catch_unwind to prevent crashes
                    let result = std::panic::catch_unwind(|| {
                        primary_manifold.boolean_op(&secondary_manifold, manifold_rs::BooleanOp::Intersection)
                    });
                    
                    match result {
                        Ok(manifold) => {
                            eprintln!("[TIMING {}] [SUCCESS] Intersection operation completed successfully", 
                                     operation_start_time.elapsed().as_micros());
                            manifold
                        },
                        Err(_) => {
                            eprintln!("[TIMING {}] [ERROR] Intersection operation panicked - likely due to mesh complexity, returning empty manifold", 
                                     operation_start_time.elapsed().as_micros());
                            manifold_rs::Manifold::empty()
                        }
                    }
                },
                BooleanOpState::Union => {
                    eprintln!("[TIMING {}] Performing union operation...", 
                             operation_start_time.elapsed().as_micros());
                    // Wrap in catch_unwind to prevent crashes
                    let result = std::panic::catch_unwind(|| {
                        primary_manifold.boolean_op(&secondary_manifold, manifold_rs::BooleanOp::Union)
                    });
                    
                    match result {
                        Ok(manifold) => {
                            eprintln!("[TIMING {}] [SUCCESS] Union operation completed successfully", 
                                     operation_start_time.elapsed().as_micros());
                            manifold
                        },
                        Err(_) => {
                            eprintln!("[TIMING {}] [ERROR] Union operation panicked - likely due to mesh complexity, returning empty manifold", 
                                     operation_start_time.elapsed().as_micros());
                            manifold_rs::Manifold::empty()
                        }
                    }
                },
                BooleanOpState::Subtract => {
                    eprintln!("[TIMING {}] Performing subtraction operation...", 
                             operation_start_time.elapsed().as_micros());
                    // Wrap in catch_unwind to prevent crashes
                    let result = std::panic::catch_unwind(|| {
                        primary_manifold.boolean_op(&secondary_manifold, manifold_rs::BooleanOp::Difference)
                    });
                    
                    match result {
                        Ok(manifold) => {
                            eprintln!("[TIMING {}] [SUCCESS] Subtraction operation completed successfully", 
                                     operation_start_time.elapsed().as_micros());
                            manifold
                        },
                        Err(_) => {
                            eprintln!("[TIMING {}] [ERROR] Subtraction operation panicked - likely due to mesh complexity, returning empty manifold", 
                                     operation_start_time.elapsed().as_micros());
                            manifold_rs::Manifold::empty()
                        }
                    }
                },
                BooleanOpState::None => return, // Already handled above
            };
            let operation_time = operation_start.elapsed();
            eprintln!("[TIMING {}] [BOOLEAN OP] Boolean operation took: {:?}", 
                     operation_start_time.elapsed().as_micros(), operation_time);

            // Log the result info
            let result_mesh_info = result_manifold.to_mesh();
            let result_vertices = result_mesh_info.vertices().len();
            let result_triangles = result_mesh_info.indices().len() / 3;
            eprintln!("[TIMING {}] [RESULT] Result after operation - Vertices: {}, Triangles: {}", 
                     operation_start_time.elapsed().as_micros(), result_vertices, result_triangles);

            // Convert back to Bevy mesh with detailed tracing
            eprintln!("[TRACE] Converting result manifold to Bevy mesh...");
            let conversion_back_start = std::time::Instant::now();
            let result_bevy_mesh = manifold_to_bevy_mesh(result_manifold);
            let conversion_back_time = conversion_back_start.elapsed();
            eprintln!("[TIMING {}] [CONVERSION BACK] Mesh conversion back to Bevy took: {:?}", 
                     operation_start_time.elapsed().as_micros(), conversion_back_time);
            
            // Log mesh stats before adding to assets
            let pre_add_vertex_count = if let Some(positions) = result_bevy_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                match positions {
                    bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                    _ => 0,
                }
            } else {
                0
            };
            
            let pre_add_index_count = if let Some(indices) = result_bevy_mesh.indices() {
                match indices {
                    bevy::render::mesh::Indices::U32(indices_vec) => indices_vec.len(),
                    bevy::render::mesh::Indices::U16(indices_vec) => indices_vec.len(),
                }
            } else {
                0
            };
            
            eprintln!("[TRACE] Pre-add mesh stats - Vertices: {}, Indices: {}", pre_add_vertex_count, pre_add_index_count);
            
            let result_mesh_handle = mesh_assets.add(result_bevy_mesh);
            eprintln!("[TRACE] Added result mesh to assets with handle");

            // Log detailed mesh information before updating entity
            eprintln!("[DEBUG] Preparing to update result entity with mesh data...");
            if let Some(result_mesh_asset) = mesh_assets.get(&result_mesh_handle) {
                let vertex_count = if let Some(positions) = result_mesh_asset.attribute(Mesh::ATTRIBUTE_POSITION) {
                    match positions {
                        bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                        _ => 0,
                    }
                } else {
                    0
                };
                
                let index_count = if let Some(indices) = result_mesh_asset.indices() {
                    match indices {
                        bevy::render::mesh::Indices::U32(indices_vec) => indices_vec.len(),
                        bevy::render::mesh::Indices::U16(indices_vec) => indices_vec.len(),
                    }
                } else {
                    0
                };
                
                eprintln!("[DEBUG] Result mesh asset stats - Vertices: {}, Indices: {}", vertex_count, index_count);
                
                // Check first few vertex positions to verify mesh data
                if let Some(positions) = result_mesh_asset.attribute(Mesh::ATTRIBUTE_POSITION) {
                    match positions {
                        bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => {
                            if !pos.is_empty() {
                                eprintln!("[DEBUG] First 3 result vertices: {:?}", &pos[..pos.len().min(3)]);
                            }
                        },
                        _ => eprintln!("[DEBUG] Result mesh position attribute is not Float32x3"),
                    }
                }
            } else {
                eprintln!("[ERROR] Could not retrieve result mesh asset!");
            }
            
            // Update the result entity
            let update_start = std::time::Instant::now();
            commands.entity(result_entity).insert(result_mesh_handle);
            let update_time = update_start.elapsed();
            eprintln!("[TIMING {}] [ENTITY UPDATE] Entity update took: {:?}", 
                     operation_start_time.elapsed().as_micros(), update_time);
            
            if let Ok(mut result_vis) = visibility_query.get_mut(result_entity) {
                *result_vis = Visibility::Visible;
                eprintln!("[DEBUG] Set result entity visibility to Visible");
            } else {
                eprintln!("[ERROR] Could not get visibility for result entity!");
            }
            
            // Check if the result is empty
            if result_vertices == 0 {
                eprintln!("[TIMING {}] [PANIC] Result mesh has 0 vertices - boolean operation failed", 
                         operation_start_time.elapsed().as_micros());
                panic!("Boolean operation {:?} failed: Result mesh has 0 vertices. This indicates that the operation was not desirable or the input shapes didn't properly overlap for the operation. Ensure shapes overlap for boolean operations to work properly.", *op_state);
            } else {
                eprintln!("[TIMING {}] [SUCCESS] Boolean operation completed with {} vertices in result", 
                         operation_start_time.elapsed().as_micros(), result_vertices);
            }
        },
        _ => {
            eprintln!("[TIMING {}] [FALLBACK] Failed to convert one or both meshes to manifolds, falling back to primitive shapes", 
                     operation_start_time.elapsed().as_micros());
            eprintln!("[TIMING {}] [FALLBACK] This typically happens when STEP meshes are not watertight solids", 
                     operation_start_time.elapsed().as_micros());
            eprintln!("[TIMING {}] [FALLBACK] Consider using mesh repair tools or ensuring STEP file exports watertight solids", 
                     operation_start_time.elapsed().as_micros());
            
            // Fall back to primitive manifolds directly
            let primitive1 = manifold_rs::Manifold::cube(1.0, 1.0, 1.0);  // Cube
            let primitive2 = manifold_rs::Manifold::sphere(0.8, 64);      // Sphere

            // Apply transformations to position the shapes for intersection
            let primary_pos = primary_transform.translation;
            let secondary_pos = secondary_transform.translation;

            let primary_manifold = primitive1.translate(
                primary_pos.x as f64,
                primary_pos.y as f64,
                primary_pos.z as f64,
            );
            let secondary_manifold = primitive2.translate(
                secondary_pos.x as f64,
                secondary_pos.y as f64,
                secondary_pos.z as f64,
            );

            // Log initial mesh info
            let prim_mesh_info = primary_manifold.to_mesh();
            let sec_mesh_info = secondary_manifold.to_mesh();
            let prim_vertices_before = prim_mesh_info.vertices().len();
            let sec_vertices_before = sec_mesh_info.vertices().len();
            eprintln!("[TIMING {}] [FALLBACK] Boolean operation: Primary vertices: {}, Secondary vertices: {}", 
                     operation_start_time.elapsed().as_micros(), prim_vertices_before, sec_vertices_before);

            // Perform boolean operation
            let operation_start = std::time::Instant::now();
            let result_manifold = match *op_state {
                BooleanOpState::Intersect => {
                    eprintln!("[TIMING {}] [FALLBACK] Performing intersection operation...", 
                             operation_start_time.elapsed().as_micros());
                    // Wrap in catch_unwind to prevent crashes
                    let result = std::panic::catch_unwind(|| {
                        primary_manifold.boolean_op(&secondary_manifold, manifold_rs::BooleanOp::Intersection)
                    });
                    
                    match result {
                        Ok(manifold) => {
                            eprintln!("[TIMING {}] [FALLBACK SUCCESS] Intersection operation completed successfully", 
                                     operation_start_time.elapsed().as_micros());
                            manifold
                        },
                        Err(_) => {
                            eprintln!("[TIMING {}] [FALLBACK ERROR] Intersection operation panicked - returning empty manifold", 
                                     operation_start_time.elapsed().as_micros());
                            manifold_rs::Manifold::empty()
                        }
                    }
                },
                BooleanOpState::Union => {
                    eprintln!("[TIMING {}] [FALLBACK] Performing union operation...", 
                             operation_start_time.elapsed().as_micros());
                    // Wrap in catch_unwind to prevent crashes
                    let result = std::panic::catch_unwind(|| {
                        primary_manifold.boolean_op(&secondary_manifold, manifold_rs::BooleanOp::Union)
                    });
                    
                    match result {
                        Ok(manifold) => {
                            eprintln!("[TIMING {}] [FALLBACK SUCCESS] Union operation completed successfully", 
                                     operation_start_time.elapsed().as_micros());
                            manifold
                        },
                        Err(_) => {
                            eprintln!("[TIMING {}] [FALLBACK ERROR] Union operation panicked - returning empty manifold", 
                                     operation_start_time.elapsed().as_micros());
                            manifold_rs::Manifold::empty()
                        }
                    }
                },
                BooleanOpState::Subtract => {
                    eprintln!("[TIMING {}] [FALLBACK] Performing subtraction operation...", 
                             operation_start_time.elapsed().as_micros());
                    // Wrap in catch_unwind to prevent crashes
                    let result = std::panic::catch_unwind(|| {
                        primary_manifold.boolean_op(&secondary_manifold, manifold_rs::BooleanOp::Difference)
                    });
                    
                    match result {
                        Ok(manifold) => {
                            eprintln!("[TIMING {}] [FALLBACK SUCCESS] Subtraction operation completed successfully", 
                                     operation_start_time.elapsed().as_micros());
                            manifold
                        },
                        Err(_) => {
                            eprintln!("[TIMING {}] [FALLBACK ERROR] Subtraction operation panicked - returning empty manifold", 
                                     operation_start_time.elapsed().as_micros());
                            manifold_rs::Manifold::empty()
                        }
                    }
                },
                BooleanOpState::None => return, // Already handled above
            };
            let operation_time = operation_start.elapsed();
            eprintln!("[TIMING {}] [FALLBACK BOOLEAN] Boolean operation took: {:?}", 
                     operation_start_time.elapsed().as_micros(), operation_time);

            // Log the result info
            let result_mesh_info = result_manifold.to_mesh();
            let result_vertices = result_mesh_info.vertices().len();
            let result_triangles = result_mesh_info.indices().len() / 3;
            eprintln!("[TIMING {}] [FALLBACK RESULT] Result after operation - Vertices: {}, Triangles: {}", 
                     operation_start_time.elapsed().as_micros(), result_vertices, result_triangles);

            // Convert back to Bevy mesh
            let conversion_back_start = std::time::Instant::now();
            let result_bevy_mesh = manifold_to_bevy_mesh(result_manifold);
            let conversion_back_time = conversion_back_start.elapsed();
            eprintln!("[TIMING {}] [FALLBACK CONVERSION] Mesh conversion back to Bevy took: {:?}", 
                     operation_start_time.elapsed().as_micros(), conversion_back_time);

            let result_mesh_handle = mesh_assets.add(result_bevy_mesh);

            // Update the result entity
            let update_start = std::time::Instant::now();
            commands.entity(result_entity).insert(result_mesh_handle);
            let update_time = update_start.elapsed();
            eprintln!("[TIMING {}] [FALLBACK UPDATE] Entity update took: {:?}", 
                     operation_start_time.elapsed().as_micros(), update_time);
            
            if let Ok(mut result_vis) = visibility_query.get_mut(result_entity) {
                *result_vis = Visibility::Visible;
            }
            
            // Check if the result is empty
            if result_vertices == 0 {
                eprintln!("[TIMING {}] [PANIC FALLBACK] Result mesh has 0 vertices - boolean operation failed", 
                         operation_start_time.elapsed().as_micros());
                panic!("Boolean operation {:?} failed: Result mesh has 0 vertices (fallback path). This indicates that the operation was not desirable or the input shapes didn't properly overlap for the operation. Ensure shapes overlap for boolean operations to work properly.", *op_state);
            } else {
                eprintln!("[TIMING {}] [SUCCESS FALLBACK] Boolean operation completed with {} vertices in result", 
                         operation_start_time.elapsed().as_micros(), result_vertices);
            }
        }
    }
    
    let total_time = operation_start_time.elapsed();
    eprintln!("[TIMING {}] [TOTAL] Boolean operation sequence completed in {:?}", 
             total_time.as_micros(), total_time);
}

// Converts a Bevy mesh to a manifold-rs Manifold
pub fn bevy_mesh_to_manifold(mesh: &Mesh) -> Option<manifold_rs::Manifold> {
    eprintln!("[DEBUG] Converting Bevy mesh to manifold-rs Manifold");
    
    // Get positions
    let positions = if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => {
                eprintln!("[DEBUG] Found {} position vertices", pos.len());
                pos
            },
            _ => {
                eprintln!("[DEBUG] Position attribute is not Float32x3");
                return None;
            }
        }
    } else {
        eprintln!("[DEBUG] No position attribute found");
        return None;
    };

    // Get indices
    let indices = if let Some(indices) = mesh.indices() {
        match indices {
            bevy::render::mesh::Indices::U32(indices_vec) => {
                eprintln!("[DEBUG] Found {} U32 indices", indices_vec.len());
                indices_vec.clone()
            },
            bevy::render::mesh::Indices::U16(indices_vec) => {
                eprintln!("[DEBUG] Found {} U16 indices, converting to U32", indices_vec.len());
                // Convert u16 to u32 indices
                indices_vec.iter().map(|&i| i as u32).collect()
            }
        }
    } else {
        eprintln!("[DEBUG] No indices found, creating indices for {} positions", positions.len());
        // If no indices, create indices for all vertices
        (0..positions.len() as u32).collect()
    };

    // Convert vertices to the format expected by manifold-rs
    let vertices_f32: Vec<f32> = positions.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
    eprintln!("[DEBUG] Flattened to {} float values", vertices_f32.len());

    let conversion_start = std::time::Instant::now();
    let mesh = manifold_rs::Mesh::new(&vertices_f32, &indices);
    let manifold = manifold_rs::Manifold::from_mesh(mesh);
    let conversion_time = conversion_start.elapsed();
    
    // Check if the resulting manifold is valid (has vertices)
    let mesh_info = manifold.to_mesh();
    let result_vertices = mesh_info.vertices().len();
    let result_indices = mesh_info.indices().len();
    
    eprintln!("[DEBUG] Manifold conversion completed in {:?} - Result: {} vertices, {} indices", 
             conversion_time, result_vertices, result_indices);
    
    if result_vertices > 0 {
        eprintln!("[DEBUG] Successfully converted to manifold with {} vertices", result_vertices);
        Some(manifold)
    } else {
        eprintln!("[DEBUG] Conversion resulted in 0 vertices - mesh may not be a valid solid");
        eprintln!("[DEBUG] This typically happens when:");
        eprintln!("[DEBUG]   - Mesh is not watertight (has holes)");
        eprintln!("[DEBUG]   - Triangle winding order is inconsistent"); 
        eprintln!("[DEBUG]   - Mesh contains degenerate/self-intersecting geometry");
        eprintln!("[DEBUG]   - Mesh normals are inconsistent");
        eprintln!("[DEBUG] Attempting to make mesh watertight...");
        
        // Try to make the mesh watertight
        if let Some(watertight_manifold) = make_mesh_watertight(positions, &indices) {
            eprintln!("[DEBUG] Successfully made mesh watertight");
            Some(watertight_manifold)
        } else {
            eprintln!("[DEBUG] Failed to make mesh watertight");
            eprintln!("[DEBUG] Consider using mesh repair tools or ensuring STEP file exports watertight solids");
            // If conversion resulted in no vertices, this might indicate the mesh isn't a valid solid
            // In that case, we return None to indicate failure
            None
        }
    }
}

// Attempts to make a mesh watertight by creating a convex hull or other repair techniques
pub fn make_mesh_watertight(
    positions: &[[f32; 3]], 
    indices: &[u32]
) -> Option<manifold_rs::Manifold> {
    eprintln!("[MAKE_WATERTIGHT] Attempting to make mesh watertight...");
    eprintln!("[MAKE_WATERTIGHT] Input: {} positions, {} indices", positions.len(), indices.len());
    
    // For very large meshes, skip convex hull to prevent stack overflow
    if positions.len() > 10000 {
        eprintln!("[MAKE_WATERTIGHT] Large mesh detected ({} vertices), skipping convex hull computation", positions.len());
        eprintln!("[MAKE_WATERTIGHT] Falling back to bounding box for watertight approximation");
        
        // Calculate bounding box
        let mut min_bound = Vec3::new(positions[0][0], positions[0][1], positions[0][2]);
        let mut max_bound = min_bound;
        
        for vertex in positions.iter() {
            let v = Vec3::new(vertex[0], vertex[1], vertex[2]);
            min_bound = min_bound.min(v);
            max_bound = max_bound.max(v);
        }
        
        let size = max_bound - min_bound;
        let center = (min_bound + max_bound) * 0.5;
        
        eprintln!("[MAKE_WATERTIGHT] Bounding box: min={:?}, max={:?}, size={:?}, center={:?}", 
                 min_bound, max_bound, size, center);
        
        // Create a cube that encompasses the mesh
        let manifold = manifold_rs::Manifold::cube(
            size.x as f64, 
            size.y as f64, 
            size.z as f64
        ).translate(
            center.x as f64,
            center.y as f64,
            center.z as f64,
        );
        
        // Check if the resulting manifold is valid
        let mesh_info = manifold.to_mesh();
        let result_vertices = mesh_info.vertices().len();
        let result_indices = mesh_info.indices().len();
        eprintln!("[MAKE_WATERTIGHT] Bounding box result: {} vertices, {} indices", result_vertices, result_indices);
        
        if result_vertices > 0 {
            eprintln!("[MAKE_WATERTIGHT] Successfully created watertight manifold with bounding box");
            return Some(manifold);
        } else {
            eprintln!("[MAKE_WATERTIGHT] Bounding box failed to create valid manifold");
        }
    } else if positions.len() >= 4 {
        eprintln!("[MAKE_WATERTIGHT] Creating convex hull from {} vertices", positions.len());
        
        // Convert to manifold-rs format
        let vertices_f32: Vec<f32> = positions.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
        eprintln!("[MAKE_WATERTIGHT] Flattened to {} float values", vertices_f32.len());
        
        // Try to create a convex hull - wrap in catch_unwind to prevent crashes
        let result = std::panic::catch_unwind(|| {
            // Create a point cloud and compute hull
            let mut point_cloud = Vec::new();
            for vertex in positions.iter().take(1000) { // Limit to first 1000 points to prevent overflow
                point_cloud.push(manifold_rs::Manifold::cube(0.01, 0.01, 0.01).translate(
                    vertex[0] as f64,
                    vertex[1] as f64,
                    vertex[2] as f64,
                ));
            }
            
            if !point_cloud.is_empty() {
                let combined_points = point_cloud.into_iter().reduce(|acc, m| acc.boolean_op(&m, manifold_rs::BooleanOp::Union)).unwrap_or_else(|| manifold_rs::Manifold::empty());
                let manifold = combined_points.hull(); // Compute convex hull
                
                // Check if the resulting manifold is valid
                let mesh_info = manifold.to_mesh();
                let result_vertices = mesh_info.vertices().len();
                let result_indices = mesh_info.indices().len();
                eprintln!("[MAKE_WATERTIGHT] Convex hull result: {} vertices, {} indices", result_vertices, result_indices);
                
                if result_vertices > 0 {
                    Some(manifold)
                } else {
                    None
                }
            } else {
                None
            }
        });
        
        match result {
            Ok(Some(manifold)) => {
                eprintln!("[MAKE_WATERTIGHT] Successfully created watertight manifold with convex hull");
                return Some(manifold);
            },
            Ok(None) => {
                eprintln!("[MAKE_WATERTIGHT] Convex hull failed to create valid manifold");
            },
            Err(_) => {
                eprintln!("[MAKE_WATERTIGHT] Convex hull computation panicked - likely stack overflow, falling back to bounding box");
            }
        }
    }
    
    // If convex hull fails or we have too few vertices, try to create a bounding box
    if !positions.is_empty() {
        eprintln!("[MAKE_WATERTIGHT] Creating bounding box from {} vertices", positions.len());
        
        // Calculate bounding box
        let mut min_bound = Vec3::new(positions[0][0], positions[0][1], positions[0][2]);
        let mut max_bound = min_bound;
        
        for vertex in positions.iter() {
            let v = Vec3::new(vertex[0], vertex[1], vertex[2]);
            min_bound = min_bound.min(v);
            max_bound = max_bound.max(v);
        }
        
        let size = max_bound - min_bound;
        let center = (min_bound + max_bound) * 0.5;
        
        eprintln!("[MAKE_WATERTIGHT] Bounding box: min={:?}, max={:?}, size={:?}, center={:?}", 
                 min_bound, max_bound, size, center);
        
        // Create a cube that encompasses the mesh
        let manifold = manifold_rs::Manifold::cube(
            size.x as f64, 
            size.y as f64, 
            size.z as f64
        ).translate(
            center.x as f64,
            center.y as f64,
            center.z as f64,
        );
        
        // Check if the resulting manifold is valid
        let mesh_info = manifold.to_mesh();
        let result_vertices = mesh_info.vertices().len();
        let result_indices = mesh_info.indices().len();
        eprintln!("[MAKE_WATERTIGHT] Bounding box result: {} vertices, {} indices", result_vertices, result_indices);
        
        if result_vertices > 0 {
            eprintln!("[MAKE_WATERTIGHT] Successfully created watertight manifold with bounding box");
            return Some(manifold);
        } else {
            eprintln!("[MAKE_WATERTIGHT] Bounding box failed to create valid manifold");
        }
    }
    
    // If all else fails, create a small cube at the origin
    eprintln!("[MAKE_WATERTIGHT] Creating fallback cube");
    let manifold = manifold_rs::Manifold::cube(1.0, 1.0, 1.0);
    
    // Check if the resulting manifold is valid
    let mesh_info = manifold.to_mesh();
    let result_vertices = mesh_info.vertices().len();
    let result_indices = mesh_info.indices().len();
    eprintln!("[MAKE_WATERTIGHT] Fallback cube result: {} vertices, {} indices", result_vertices, result_indices);
    
    if result_vertices > 0 {
        eprintln!("[MAKE_WATERTIGHT] Successfully created watertight manifold with fallback cube");
        Some(manifold)
    } else {
        eprintln!("[MAKE_WATERTIGHT] Even fallback cube failed");
        None
    }
}

// Converts a manifold-rs Manifold to a Bevy mesh
pub fn manifold_to_bevy_mesh(manifold: manifold_rs::Manifold) -> Mesh {
    eprintln!("[DEBUG] Converting manifold-rs Manifold to Bevy mesh");
    let conversion_start = std::time::Instant::now();
    
    let mesh = manifold.to_mesh();
    let conversion_time = conversion_start.elapsed();

    let vertices = mesh.vertices();
    let indices = mesh.indices();
    
    eprintln!("[DEBUG] Manifold->Bevy conversion took {:?} - Vertices: {}, Indices: {}, Properties: {}", 
             conversion_time, vertices.len(), indices.len(), mesh.num_props());

    match mesh.num_props() {
        3 => {
            eprintln!("[DEBUG] Processing vertex data without normals");
            // Vertex without normals - vertices is a flat Vec<f32> where every 3 values are x,y,z
            let vertex_positions: Vec<[f32; 3]> = vertices.chunks(3).map(|chunk| [chunk[0], chunk[1], chunk[2]]).collect();
            eprintln!("[DEBUG] Created {} vertex positions from {} float values", vertex_positions.len(), vertices.len());

            let mesh_build_start = std::time::Instant::now();
            let mut result = Mesh::new(
                bevy::render::mesh::PrimitiveTopology::TriangleList,
                bevy::render::render_asset::RenderAssetUsages::all(),
            );
            
            eprintln!("[DEBUG] Inserting {} vertex positions", vertex_positions.len());
            result.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions);
            
            eprintln!("[DEBUG] Inserting {} indices", indices.len());
            result.insert_indices(bevy::render::mesh::Indices::U32(indices.clone()));
            
            // DO NOT call duplicate_vertices() or compute_flat_normals() as they can corrupt the mesh indices
            // Instead, let the renderer handle normals if needed
            
            let mesh_build_time = mesh_build_start.elapsed();
            
            // Verify the mesh has indices after construction
            if let Some(mesh_indices) = result.indices() {
                match mesh_indices {
                    bevy::render::mesh::Indices::U32(indices_vec) => {
                        eprintln!("[DEBUG] Mesh construction verified - {} U32 indices", indices_vec.len());
                    },
                    bevy::render::mesh::Indices::U16(indices_vec) => {
                        eprintln!("[DEBUG] Mesh construction verified - {} U16 indices", indices_vec.len());
                    }
                }
            } else {
                eprintln!("[DEBUG] Mesh construction completed but has no indices!");
            }
            
            eprintln!("[DEBUG] Bevy mesh construction took {:?}", mesh_build_time);
            result
        }
        6 => {
            eprintln!("[DEBUG] Processing vertex data with normals");
            // Vertex with normals - vertices is a flat Vec<f32> where every 6 values are x,y,z,nx,ny,nz
            let normals: Vec<[f32; 3]> = vertices.chunks(6).map(|chunk| [chunk[3], chunk[4], chunk[5]]).collect();
            let vertex_positions: Vec<[f32; 3]> = vertices.chunks(6).map(|chunk| [chunk[0], chunk[1], chunk[2]]).collect();
            eprintln!("[DEBUG] Created {} vertex positions and {} normals from {} float values", 
                     vertex_positions.len(), normals.len(), vertices.len());

            let mesh_build_start = std::time::Instant::now();
            let mut result = Mesh::new(
                bevy::render::mesh::PrimitiveTopology::TriangleList,
                bevy::render::render_asset::RenderAssetUsages::all(),
            );
            
            eprintln!("[DEBUG] Inserting {} vertex positions with normals", vertex_positions.len());
            result.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions);
            
            eprintln!("[DEBUG] Inserting {} normals", normals.len());
            result.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            
            eprintln!("[DEBUG] Inserting {} indices", indices.len());
            result.insert_indices(bevy::render::mesh::Indices::U32(indices.clone()));
            
            // DO NOT call duplicate_vertices() or compute_flat_normals() as they can corrupt the mesh indices
            // The normals are already provided, so we don't need to compute them
            
            let mesh_build_time = mesh_build_start.elapsed();
            
            // Verify the mesh has indices after construction
            if let Some(mesh_indices) = result.indices() {
                match mesh_indices {
                    bevy::render::mesh::Indices::U32(indices_vec) => {
                        eprintln!("[DEBUG] Mesh construction with normals verified - {} U32 indices", indices_vec.len());
                    },
                    bevy::render::mesh::Indices::U16(indices_vec) => {
                        eprintln!("[DEBUG] Mesh construction with normals verified - {} U16 indices", indices_vec.len());
                    }
                }
            } else {
                eprintln!("[DEBUG] Mesh construction with normals completed but has no indices!");
            }
            
            eprintln!("[DEBUG] Bevy mesh construction with normals took {:?}", mesh_build_time);
            result
        }
        num_props => {
            eprintln!("[ERROR] Invalid property count {num_props}");
            panic!("Invalid property count {num_props}")
        },
    }
}