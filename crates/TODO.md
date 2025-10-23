We need to offer an alternative to foxtrot, because its triangulation is far from perfect:

I suggest an occt feature flag, dsiabled by default.

We can update the README.md to emphasise the benefits and drawbacks of each, ultimately OpenCascade is a well established group with very far along tooling, but it's just a CPP wrapper at the end of the day whereas Foxtrot is PURE RUST.

YMMV -- the foxtrot step parser is faster, the OCCT one is more robust.

The foxtrot triangulation is faster, the OCCT one is more robust (Particularly with NURBS).



```toml
opencascade = { version = "0.2.0", optional = true }
```

usage:
can be gleamed from this:
```rust
use clap::Parser;
use std::io::Write;
use std::time::SystemTime;
use tracing::trace;

mod app;
mod camera;
mod cli;
mod img_utils;
mod model;
mod wgpu_utils;

use cli::{Args, TriangulationEngine};

#[cfg(feature = "occt")]
use opencascade::primitives::Shape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let start_time = std::time::Instant::now();

    let args = Args::parse();
    trace!("Starting thumbnail generation for: {}", args.input);

    let engine = args.engine.clone();
    let thumbnail_size = args.thumbnail_size;
    let transparent = args.transparent;
    let decimation_ratio = args.decimation_ratio;
    let decimation_error = args.decimation_error;

    #[cfg(feature = "occt")]
    let occt_linear_deflection = args.occt_linear_deflection;
    #[cfg(feature = "occt")]
    let occt_angular_deflection = args.occt_angular_deflection;

    let input = args.input.clone();
    let engine_for_thread = engine.clone();
    let loader = std::thread::spawn(move || {
        let foxtrot_pipeline_start = std::time::Instant::now();
        use step::step_file::StepFile;
        use triangulate::triangulate::triangulate4 as triangulate;
        // use triangulate::wgpu_triangulate::wgpu_impl::triangulate;

        let data = std::fs::read(&input).expect("Could not open file");
        let flat = StepFile::strip_flatten(&data);
        let step = StepFile::parse(&flat);
        trace!("STEP file parsed in {:?}", foxtrot_pipeline_start.elapsed());

        match engine_for_thread {
            TriangulationEngine::Foxtrot => {
                let (mut mesh, _stats) = triangulate(&step);
                trace!(
                    "Mesh triangulation in {:?}, starting with {} vertices and {} triangles.",
                    foxtrot_pipeline_start.elapsed(),
                    mesh.verts.len(),
                    mesh.triangles.len()
                );

                let decimation_start = std::time::Instant::now();
                let original_vertex_count = mesh.verts.len();
                let original_triangle_count = mesh.triangles.len();

                trace!(
                    "Starting mesh decimation from {} vertices and {} triangles",
                    original_vertex_count, original_triangle_count
                );

                let vertices: Vec<f32> = mesh
                    .verts
                    .iter()
                    .flat_map(|v| [v.pos.x as f32, v.pos.y as f32, v.pos.z as f32])
                    .collect();

                let indices: Vec<u32> = mesh
                    .triangles
                    .iter()
                    .flat_map(|t| [t.verts.x, t.verts.y, t.verts.z])
                    .collect();

                let target_index_count = (indices.len() as f32 * decimation_ratio) as usize;
                let target_error = decimation_error;

                let vertex_adapter = meshopt::VertexDataAdapter::new(
                    bytemuck::cast_slice(&vertices),
                    3 * std::mem::size_of::<f32>(),
                    0,
                )
                .unwrap();

                let mut error_result: f32 = 0.0;
                let simplified_indices = meshopt::simplify(
                    &indices,
                    &vertex_adapter,
                    target_index_count,
                    target_error,
                    meshopt::SimplifyOptions::LockBorder,
                    Some(&mut error_result),
                );

                let new_triangles = simplified_indices
                    .chunks_exact(3)
                    .map(|chunk| triangulate::mesh::Triangle {
                        verts: nalgebra_glm::U32Vec3::new(chunk[0], chunk[1], chunk[2]),
                    })
                    .collect();

                mesh.triangles = new_triangles;

                trace!(
                    "Mesh decimation completed in {:?}. Decimated to {} vertices and {} triangles.",
                    decimation_start.elapsed(),
                    mesh.verts.len(),
                    mesh.triangles.len()
                );

                mesh
            }
            #[cfg(feature = "occt")]
            TriangulationEngine::Occt => {
                let occt_pipeline_start = std::time::Instant::now();
                trace!(
                    "Using OCCT engine with linear_deflection={} angular_deflection={}",
                    occt_linear_deflection, occt_angular_deflection
                );

                let shape_to_mesh = Shape::read_step(input).expect("OCCT failed to read STEP file");

                use opencascade::mesh::Mesher;
                let occt_mesh = Mesher::new(&shape_to_mesh).mesh();

                trace!(
                    "OCCT triangulation and mesh unification completed in {:?}",
                    occt_pipeline_start.elapsed()
                );

                let mut final_mesh = triangulate::mesh::Mesh {
                    verts: occt_mesh
                        .vertices
                        .iter()
                        .zip(occt_mesh.normals.iter())
                        .map(|(v, n)| triangulate::mesh::Vertex {
                            pos: nalgebra_glm::DVec3::new(v.x, v.y, v.z),
                            norm: nalgebra_glm::DVec3::new(n.x, n.y, n.z),
                            color: nalgebra_glm::DVec3::new(0.5, 0.5, 0.5),
                        })
                        .collect(),
                    triangles: occt_mesh
                        .indices
                        .chunks_exact(3)
                        .map(|tri| triangulate::mesh::Triangle {
                            verts: nalgebra_glm::U32Vec3::new(
                                tri[0] as u32,
                                tri[1] as u32,
                                tri[2] as u32,
                            ),
                        })
                        .collect(),
                };

                trace!(
                    "Converted to local mesh with {} vertices and {} triangles.",
                    final_mesh.verts.len(),
                    final_mesh.triangles.len()
                );

                let decimation_start = std::time::Instant::now();
                let original_vertex_count = final_mesh.verts.len();
                let original_triangle_count = final_mesh.triangles.len();

                trace!(
                    "Starting mesh decimation from {} vertices and {} triangles",
                    original_vertex_count, original_triangle_count
                );

                let vertices: Vec<f32> = final_mesh
                    .verts
                    .iter()
                    .flat_map(|v| [v.pos.x as f32, v.pos.y as f32, v.pos.z as f32])
                    .collect();

                let indices: Vec<u32> = final_mesh
                    .triangles
                    .iter()
                    .flat_map(|t| [t.verts.x, t.verts.y, t.verts.z])
                    .collect();

                let target_index_count = (indices.len() as f32 * decimation_ratio) as usize;
                let target_error = decimation_error;

                let vertex_adapter = meshopt::VertexDataAdapter::new(
                    bytemuck::cast_slice(&vertices),
                    3 * std::mem::size_of::<f32>(),
                    0,
                )
                .unwrap();

                let mut error_result: f32 = 0.0;
                let simplified_indices = meshopt::simplify(
                    &indices,
                    &vertex_adapter,
                    target_index_count,
                    target_error,
                    meshopt::SimplifyOptions::LockBorder,
                    Some(&mut error_result),
                );

                let new_triangles = simplified_indices
                    .chunks_exact(3)
                    .map(|chunk| triangulate::mesh::Triangle {
                        verts: nalgebra_glm::U32Vec3::new(chunk[0], chunk[1], chunk[2]),
                    })
                    .collect();

                final_mesh.triangles = new_triangles;

                trace!(
                    "Mesh decimation completed in {:?}. Decimated to {} vertices and {} triangles.",
                    decimation_start.elapsed(),
                    final_mesh.verts.len(),
                    final_mesh.triangles.len()
                );

                final_mesh
            }
        }
    });
/* SNIP */
}
```