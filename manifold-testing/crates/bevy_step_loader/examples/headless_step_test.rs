// Headless test to verify STEP file loading works correctly
use bevy::prelude::*;
use bevy_step_loader::{StepAsset, StepPlugin};

#[derive(Resource)]
struct TestResult {
    success: bool,
    message: String,
}

fn main() {
    let mut app = App::new();
    
    // Initialize minimal app with STEP plugin
    app.add_plugins((
        MinimalPlugins,
        StepPlugin,
    ))
    .insert_resource(TestResult {
        success: false,
        message: "Not started".to_string(),
    })
    .add_systems(Startup, setup_test)
    .add_systems(Update, (check_step_loading, exit_after_check));
    
    app.run();
    
    // Get the test result
    if let Some(result) = app.world.get_resource::<TestResult>() {
        if result.success {
            println!("✅ STEP file loading test PASSED: {}", result.message);
            std::process::exit(0);
        } else {
            println!("❌ STEP file loading test FAILED: {}", result.message);
            std::process::exit(1);
        }
    } else {
        println!("❌ Test result resource not found");
        std::process::exit(1);
    }
}

fn setup_test(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    println!("Setting up STEP file loading test...");
    
    // Try to load a STEP file - use a simple one that should exist
    let step_handle: Handle<StepAsset> = asset_server.load("real_parts/multifeature.step");
    
    commands.spawn((
        StepTestComponent {
            handle: step_handle,
            start_time: std::time::Instant::now(),
        },
    ));
    
    commands.insert_resource(TestResult {
        success: false,
        message: "Test started".to_string(),
    });
}

#[derive(Component)]
struct StepTestComponent {
    handle: Handle<StepAsset>,
    start_time: std::time::Instant,
}

fn check_step_loading(
    mut test_results: ResMut<TestResult>,
    step_assets: Res<Assets<StepAsset>>,
    query: Query<&StepTestComponent>,
) {
    if let Ok(step_test) = query.get_single() {
        // Check if 5 seconds have passed
        if step_test.start_time.elapsed().as_secs() > 5 {
            test_results.success = false;
            test_results.message = "Timeout: STEP file did not load within 5 seconds".to_string();
            return;
        }
        
        // Check if the STEP asset has loaded
        if let Some(step_asset) = step_assets.get(&step_test.handle) {
            let vertex_count = if let Some(positions) = step_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                match positions {
                    bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                    _ => 0,
                }
            } else {
                0
            };
            
            test_results.success = vertex_count > 0;
            if test_results.success {
                test_results.message = format!("STEP file loaded successfully with {} vertices", vertex_count);
            } else {
                test_results.message = "STEP file loaded but has 0 vertices".to_string();
            }
        } else {
            // Still loading
            test_results.message = "STEP file still loading...".to_string();
        }
    }
}

fn exit_after_check(
    test_results: Res<TestResult>,
    mut exit: EventWriter<AppExit>,
) {
    if test_results.message != "Test started" && test_results.message != "STEP file still loading..." {
        exit.send(AppExit::Success);
    }
}