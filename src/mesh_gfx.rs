//! Simple resolution-independent 2D meshes (plain shapes, no layered fluff).

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use std::f32::consts::{FRAC_PI_2, PI};

#[inline]
pub fn circle(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    radius: f32,
    color: Color,
) -> (Mesh2d, MeshMaterial2d<ColorMaterial>) {
    (
        Mesh2d(meshes.add(Circle::new(radius))),
        MeshMaterial2d(materials.add(color)),
    )
}

/// Annulus (ring) for HUD-style joystick rims (Fortnite-like outer ring).
/// Outer radius = 1.0 in mesh space; `inner_frac` is inner hole radius (0..1).
pub fn ring_mesh(inner_frac: f32, segments: u32) -> Mesh {
    let segs = segments.max(16);
    let inner = inner_frac.clamp(0.05, 0.95);
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((segs as usize) * 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity((segs as usize) * 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity((segs as usize) * 2);
    let mut indices: Vec<u32> = Vec::with_capacity((segs as usize) * 6);

    for i in 0..segs {
        let a = (i as f32 / segs as f32) * PI * 2.0;
        let c = a.cos();
        let s = a.sin();
        positions.push([c, s, 0.0]);
        positions.push([c * inner, s * inner, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.5 + 0.5 * c, 0.5 + 0.5 * s]);
        uvs.push([0.5 + 0.5 * c * inner, 0.5 + 0.5 * s * inner]);
    }
    for i in 0..segs {
        let i0 = i * 2;
        let i1 = i0 + 1;
        let j0 = ((i + 1) % segs) * 2;
        let j1 = j0 + 1;
        indices.extend_from_slice(&[i0, j0, i1, j0, j1, i1]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Ring mesh with outer radius 1.0 — scale `Transform` by desired world radius.
#[inline]
pub fn ring(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    inner_frac: f32,
    color: Color,
) -> (Mesh2d, MeshMaterial2d<ColorMaterial>) {
    (
        Mesh2d(meshes.add(ring_mesh(inner_frac, 48))),
        MeshMaterial2d(materials.add(color)),
    )
}

#[inline]
pub fn poly(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    radius: f32,
    sides: u32,
    color: Color,
) -> (Mesh2d, MeshMaterial2d<ColorMaterial>) {
    (
        Mesh2d(meshes.add(RegularPolygon::new(radius, sides))),
        MeshMaterial2d(materials.add(color)),
    )
}

/// Classic 5-point star mesh.
pub fn five_point_star_mesh(outer_r: f32, inner_r: f32) -> Mesh {
    let tips = 5usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(tips * 2 + 1);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(tips * 2 + 1);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(tips * 2 + 1);
    let mut indices: Vec<u32> = Vec::with_capacity(tips * 2 * 3);

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..(tips * 2) {
        let angle = -FRAC_PI_2 + (i as f32) * (PI / tips as f32);
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        let x = r * angle.cos();
        let y = r * angle.sin();
        positions.push([x, y, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.5 + x / (2.0 * outer_r), 0.5 + y / (2.0 * outer_r)]);
    }

    let n = (tips * 2) as u32;
    for i in 1..=n {
        let next = if i == n { 1 } else { i + 1 };
        indices.extend_from_slice(&[0, i, next]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

#[inline]
pub fn star(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    outer_r: f32,
    color: Color,
) -> (Mesh2d, MeshMaterial2d<ColorMaterial>) {
    (
        Mesh2d(meshes.add(five_point_star_mesh(outer_r, outer_r * 0.42))),
        MeshMaterial2d(materials.add(color)),
    )
}

#[inline]
pub fn set_material_color(
    materials: &mut Assets<ColorMaterial>,
    handle: &MeshMaterial2d<ColorMaterial>,
    color: Color,
) {
    if let Some(mut mat) = materials.get_mut(&handle.0) {
        mat.color = color;
    }
}
