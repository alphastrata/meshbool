use bevy::asset::io::AsyncReadExt;
use bevy::utils::ConditionalSendFuture;
use bevy::{
    asset::{Asset, AssetLoader, AssetServer, LoadContext},
    prelude::*,
    reflect::TypePath,
    render::mesh::{Indices, Mesh},
};
use std::path::Path;

// The plugin to register the asset and loader
pub struct StepPlugin;

impl Plugin for StepPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StepAsset>()
            .register_asset_loader(StepLoader);
    }
}

// The asset representing a STEP file
#[derive(Asset, TypePath, Debug, Clone)]
pub struct StepAsset {
    pub mesh: Mesh,
}

// The loader for STEP files (standard asset loader)
#[derive(Default)]
pub struct StepLoader;

impl AssetLoader for StepLoader {
    type Asset = StepAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn extensions(&self) -> &[&str] {
        &["step", "stp"]
    }

    fn load<'s>(
        &'s self,
        reader: &'s mut bevy::asset::io::Reader,
        #[allow(unused_variables)] settings: &'s Self::Settings,
        #[allow(unused_variables)] load_context: &'s mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // Parse and triangulate the STEP file
            let mesh = triangulate_step_file(&bytes)?;

            Ok(StepAsset { mesh })
        })
    }
}

// Function to load a STEP file from an arbitrary path
pub fn load_step_file_from_path<P: AsRef<Path>>(path: P) -> Result<StepAsset, anyhow::Error> {
    let path = path.as_ref();
    
    // Read the file
    let step_data = std::fs::read(path)?;
    
    // Parse and triangulate the STEP file
    let mesh = triangulate_step_file(&step_data)?;
    
    Ok(StepAsset { mesh })
}

fn triangulate_step_file(step_data: &[u8]) -> Result<Mesh, anyhow::Error> {
    // Use foxtrot by default, or OCCT if the feature is enabled
    #[cfg(feature = "opencascade")]
    {
        use opencascade::primitives::Shape;

        // For OCCT, we need to write the data to a temporary file and then read it
        let temp_path = std::env::temp_dir().join("temp_step_file.step");
        std::fs::write(&temp_path, step_data)?;

        let shape_to_mesh = Shape::read_step(temp_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("OCCT failed to read STEP file: {:?}", e))?;

        use opencascade::mesh::Mesher;
        let occt_mesh = Mesher::new(&shape_to_mesh).mesh();

        // Convert OCCT mesh to Bevy mesh
        let vertices: Vec<[f32; 3]> = occt_mesh
            .vertices
            .iter()
            .map(|v| [v.x as f32, v.y as f32, v.z as f32])
            .collect();

        let indices: Vec<u32> = occt_mesh.indices.iter().map(|&i| i as u32).collect();

        let mut bevy_mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            Default::default(),
        );

        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        bevy_mesh.insert_indices(Indices::U32(indices));

        Ok(bevy_mesh)
    }

    #[cfg(not(feature = "opencascade"))]
    {
        // Use foxtrot backend by default
        use step::step_file::StepFile;
        use triangulate::triangulate::triangulate4 as triangulate;

        let flat = StepFile::strip_flatten(step_data);
        let step = StepFile::parse(&flat);
        let (triangulated_mesh, _stats) = triangulate(&step);

        let vertices: Vec<[f32; 3]> = triangulated_mesh
            .verts
            .iter()
            .map(|v| [v.pos.x as f32, v.pos.y as f32, v.pos.z as f32])
            .collect();

        let indices: Vec<u32> = triangulated_mesh
            .triangles
            .iter()
            .flat_map(|t| [t.verts.x, t.verts.y, t.verts.z])
            .collect();

        let mut bevy_mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            Default::default(),
        );

        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        bevy_mesh.insert_indices(Indices::U32(indices));

        Ok(bevy_mesh)
    }
}
