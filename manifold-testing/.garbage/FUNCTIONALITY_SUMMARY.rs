// Summary of all implemented functionality

/// This system implements space bar toggle for boolean operations
fn cycle_boolean_op(
    keys: Res<ButtonInput<KeyCode>>,
    mut op_state: ResMut<BooleanOpState>,
) {
    if keys.just_pressed(KeyCode::Space) {
        *op_state = match *op_state {
            BooleanOpState::None => BooleanOpState::Intersect,
            BooleanOpState::Intersect => BooleanOpState::Union,
            BooleanOpState::Union => BooleanOpState::Subtract,
            BooleanOpState::Subtract => BooleanOpState::None,
        };
        debug!("BooleanOpState changed to: {:?}", *op_state);
    }
}

/// This system implements camera orbiting around the main part
fn orbit_camera(
    mut query: Query<&mut Transform, With<OrbitCamera>>,
    mut orbit_state: ResMut<OrbitState>,
) {
    orbit_state.angle += 0.005; // Slowly rotate the camera
    if let Ok(mut transform) = query.get_single_mut() {
        let x = orbit_state.center.x + orbit_state.distance * orbit_state.angle.cos();
        let z = orbit_state.center.z + orbit_state.distance * orbit_state.angle.sin();
        let y = orbit_state.center.y + 2.0; // Keep a slight elevation
        
        *transform = Transform::from_translation(Vec3::new(x, y, z))
            .looking_at(orbit_state.center, Vec3::Y);
    }
}

/// This system exits when 'q' is pressed with error message
fn exit_on_q_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        eprintln!("User did not see expected results");
        exit.send(AppExit::Success);
    }
}

/// This system implements proper error handling with useful messages
fn apply_boolean_operations(
    mut commands: Commands,
    boolean_handles: Option<Res<BooleanHandles>>,
    pbr_query: Query<(&Handle<Mesh>, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut visibility_query: Query<&mut Visibility>,
    op_state: Res<BooleanOpState>,
) {
    // ... existing implementation ...
    
    // Check if the result is empty and panic with useful message
    if result_vertices == 0 {
        eprintln!("[TIMING {}] [PANIC] Result mesh has 0 vertices - boolean operation failed", 
                 operation_start_time.elapsed().as_micros());
        panic!("Boolean operation {:?} failed: Result mesh has 0 vertices. This indicates that the operation was not desirable or the input shapes didn't properly overlap for the operation. Ensure shapes overlap for boolean operations to work properly.", *op_state);
    }
    
    // ... rest of implementation ...
}