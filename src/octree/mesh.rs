use amethyst::renderer::rendy::mesh::MeshBuilder;
use amethyst::renderer::rendy::mesh::{Position, Normal, Tangent, TexCoord, Indices};

pub fn gen() -> MeshBuilder<'static> {
  let bd = MeshBuilder::new();
    bd
        .with_vertices(vec![
          Position([0.0, 0.0, 0.0]),
          Position([0.0, 1.0, 0.0]),
          Position([0.0, 0.0, 1.0]),
        ])
        .with_vertices(vec![
          Normal([0.0, 0.0, 0.0]),
          Normal([0.0, 0.0, 0.0]),
          Normal([0.0, 0.0, 0.0]),
        ])
        .with_indices(Indices::U16(vec![0, 1, 2].into()))
}
