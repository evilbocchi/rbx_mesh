use napi::Result;
use napi::bindgen_prelude::{Float32Array, Int8Array, Uint8Array, Uint32Array};
use napi_derive::napi;
use rbx_mesh::mesh::{Face2, Mesh, Vertex2, Vertex2Truncated, Vertices2};

use crate::parse_versioned;

#[napi(object, js_name = "ParsedMesh")]
pub struct ParsedMesh {
	/// Roblox mesh format revision (for example "1.01", "4.01", "7.00").
	pub version: String,
	/// Flat xyz vertex positions.
	pub positions: Float32Array,
	/// Flat xyz vertex normals.
	pub normals: Float32Array,
	/// Flat uv texture coordinates.
	pub uvs: Float32Array,
	/// Flat xyzw tangent data. Mesh v1 has no tangent data.
	pub tangents: Option<Int8Array>,
	/// Flat rgba vertex colors. Truncated v2/v3 vertices and v1 have no color data.
	pub colors: Option<Uint8Array>,
	/// Flat triangle index buffer (three indices per face).
	pub indices: Uint32Array,
}

fn revision(mesh: &Mesh) -> String {
	use rbx_mesh::mesh::{Revision1, Revision2, Revision3, Revision4, Revision5, Revision7};

	match mesh {
		Mesh::V1(mesh) => match &mesh.revision {
			Revision1::Version100 => "1.00",
			Revision1::Version101 => "1.01",
		},
		Mesh::V2(mesh) => match &mesh.revision {
			Revision2::Version200 => "2.00",
		},
		Mesh::V3(mesh) => match &mesh.revision {
			Revision3::Version300 => "3.00",
			Revision3::Version301 => "3.01",
		},
		Mesh::V4(mesh) => match &mesh.revision {
			Revision4::Version400 => "4.00",
			Revision4::Version401 => "4.01",
		},
		Mesh::V5(mesh) => match &mesh.revision {
			Revision5::Version500 => "5.00",
		},
		Mesh::V7(mesh) => match &mesh.revision {
			Revision7::Version700 => "7.00",
		},
	}
	.to_owned()
}

#[derive(Default)]
struct FlatMesh {
	positions: Vec<f32>,
	normals: Vec<f32>,
	uvs: Vec<f32>,
	tangents: Option<Vec<i8>>,
	colors: Option<Vec<u8>>,
	indices: Vec<u32>,
}

impl FlatMesh {
	fn with_attributes(has_tangents: bool, has_colors: bool) -> Self {
		Self {
			tangents: has_tangents.then(Vec::new),
			colors: has_colors.then(Vec::new),
			..Self::default()
		}
	}

	fn push_vertex2(&mut self, vertex: &Vertex2) {
		self.positions.extend_from_slice(&vertex.pos);
		self.normals.extend_from_slice(&vertex.norm);
		self.uvs.extend_from_slice(&vertex.tex);
		if let Some(tangents) = &mut self.tangents {
			tangents.extend_from_slice(&vertex.tangent);
		}
		if let Some(colors) = &mut self.colors {
			colors.extend_from_slice(&vertex.color);
		}
	}

	fn push_vertex2_truncated(&mut self, vertex: &Vertex2Truncated) {
		self.positions.extend_from_slice(&vertex.pos);
		self.normals.extend_from_slice(&vertex.norm);
		self.uvs.extend_from_slice(&vertex.tex);
		if let Some(tangents) = &mut self.tangents {
			tangents.extend_from_slice(&vertex.tangent);
		}
	}

	fn push_face2(&mut self, face: &Face2) {
		self.indices.extend(face.0.iter().map(|index| index.0));
	}
}

fn flatten(mesh: &Mesh) -> FlatMesh {
	match mesh {
		Mesh::V1(mesh) => {
			let mut out = FlatMesh::with_attributes(false, false);
			out.positions.reserve(mesh.vertices.len() * 3);
			out.normals.reserve(mesh.vertices.len() * 3);
			out.uvs.reserve(mesh.vertices.len() * 2);
			out.indices.reserve(mesh.vertices.len());

			for (index, vertex) in mesh.vertices.iter().enumerate() {
				out.positions.extend_from_slice(&vertex.pos);
				out.normals.extend_from_slice(&vertex.norm);
				// The third texture component is part of the v1 format but is not a UV component.
				out.uvs.extend_from_slice(&vertex.tex[..2]);
				out.indices.push(index as u32);
			}
			out
		}
		Mesh::V2(mesh) => {
			let has_colors = matches!(&mesh.vertices, Vertices2::Full(_));
			let mut out = FlatMesh::with_attributes(true, has_colors);
			match &mesh.vertices {
				Vertices2::Full(vertices) => {
					for vertex in vertices {
						out.push_vertex2(vertex);
					}
				}
				Vertices2::Truncated(vertices) => {
					for vertex in vertices {
						out.push_vertex2_truncated(vertex);
					}
				}
			}
			for face in &mesh.faces {
				out.push_face2(face);
			}
			out
		}
		Mesh::V3(mesh) => {
			let has_colors = matches!(&mesh.vertices, Vertices2::Full(_));
			let mut out = FlatMesh::with_attributes(true, has_colors);
			match &mesh.vertices {
				Vertices2::Full(vertices) => {
					for vertex in vertices {
						out.push_vertex2(vertex);
					}
				}
				Vertices2::Truncated(vertices) => {
					for vertex in vertices {
						out.push_vertex2_truncated(vertex);
					}
				}
			}
			for face in &mesh.faces {
				out.push_face2(face);
			}
			out
		}
		Mesh::V4(mesh) => {
			let mut out = FlatMesh::with_attributes(true, true);
			for vertex in &mesh.vertices {
				out.push_vertex2(vertex);
			}
			for face in &mesh.faces {
				out.push_face2(face);
			}
			out
		}
		Mesh::V5(mesh) => {
			let mut out = FlatMesh::with_attributes(true, true);
			for vertex in &mesh.vertices {
				out.push_vertex2(vertex);
			}
			for face in &mesh.faces {
				out.push_face2(face);
			}
			out
		}
		Mesh::V7(mesh) => {
			let mut out = FlatMesh::with_attributes(true, true);
			for vertex in &mesh.vertices {
				out.push_vertex2(vertex);
			}
			for face in &mesh.faces {
				out.push_face2(face);
			}
			out
		}
	}
}

/// Parse any supported Roblox mesh revision into a version-independent, GPU-friendly representation.
/// Node Buffers are Uint8Arrays, so they can be passed directly.
#[napi]
pub fn parse_mesh(data: &[u8]) -> Result<ParsedMesh> {
	let mesh = parse_versioned(data)?;
	let version = revision(&mesh);
	let flat = flatten(&mesh);

	Ok(ParsedMesh {
		version,
		positions: Float32Array::from(flat.positions),
		normals: Float32Array::from(flat.normals),
		uvs: Float32Array::from(flat.uvs),
		tangents: flat.tangents.map(Int8Array::from),
		colors: flat.colors.map(Uint8Array::from),
		indices: Uint32Array::from(flat.indices),
	})
}
