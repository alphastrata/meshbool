use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    prelude::*,
    reflect::TypePath,
};
use bevy_mesh::{Indices, Mesh, PrimitiveTopology};

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

// The loader for STEP files
#[derive(Default)]
pub struct StepLoader;

impl AssetLoader for StepLoader {
    type Asset = StepAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn extensions(&self) -> &[&str] {
        &["step", "stp", "STEP", "STP"]
    }

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl futures::Future<Output = Result<Self::Asset, Self::Error>> + Send {
        async move {
            let mut bytes = Vec::new();
            use futures::AsyncReadExt;
            reader.read_to_end(&mut bytes).await?;

            // Parse and triangulate the STEP file
            let mesh = triangulate_step_file(&bytes)?;

            Ok(StepAsset { mesh })
        }
    }
}

/// Triangulate the STEP file data into a Bevy Mesh.
/// Depending on the feature flag, it uses either OpenCASCADE (opencascade) or Foxtrot library.
///
/// The 'opencascade' feature, means you'll build it via the wrapper, some cmake etc deps and fanalging may be required
/// however, it is SIGNIFICANTLY more robust and can handle a wider variety of STEP files, and their miscellaneous shitfuckery.
fn triangulate_step_file(step_data: &[u8]) -> Result<Mesh, anyhow::Error> {
    #[cfg(feature = "opencascade")]
    {
        triangulate_with_occt(step_data)
    }

    #[cfg(feature = "fallback")]
    {
        triangulate_with_foxtrot(step_data)
    }
    
    // Default case when neither feature is enabled
    #[cfg(not(any(feature = "opencascade", feature = "fallback")))]
    {
        Err(anyhow::anyhow!("No triangulation backend enabled. Enable either 'opencascade' or 'fallback' feature."))
    }
}

#[cfg(feature = "opencascade")]
fn triangulate_with_occt(step_data: &[u8]) -> Result<Mesh, anyhow::Error> {
    use opencascade::primitives::Shape;
    use opencascade::mesh::Mesher;

    let temp_path = std::env::temp_dir().join("temp_step_file.step");
    std::fs::write(&temp_path, step_data)?;

    let shape_to_mesh = Shape::read_step(temp_path.to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("OCCT failed to read STEP file: {:?}", e))?;

    let occt_mesh = Mesher::new(&shape_to_mesh).mesh();

    let vertices: Vec<[f32; 3]> = occt_mesh
        .vertices
        .iter()
        .map(|v| [v.x as f32, v.y as f32, v.z as f32])
        .collect();

    let indices: Vec<u32> = occt_mesh.indices.iter().map(|&i| i as u32).collect();

    let mut bevy_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::all(), // Using the asset API directly
    );
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    bevy_mesh.insert_indices(Indices::U32(indices));

    Ok(bevy_mesh)
}

#[cfg(feature = "fallback")]
fn triangulate_with_foxtrot(step_data: &[u8]) -> Result<Mesh, anyhow::Error> {
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
        PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::all(), // Using the asset API directly
    );
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    bevy_mesh.insert_indices(Indices::U32(indices));

    Ok(bevy_mesh)
}