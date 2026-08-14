use napi::bindgen_prelude::Buffer;
use napi::{Error, Result};
use napi_derive::napi;
use rbx_mesh::mesh as rbx;
use rbx_mesh::union_graphics as graphics;

use crate::{mesh_version_name, parse_versioned};

fn f32s<const N: usize>(values: &[f32; N]) -> Vec<f64> {
	values.iter().map(|&value| value as f64).collect()
}

fn u8s<const N: usize>(values: &[u8; N]) -> Vec<u32> {
	values.iter().map(|&value| value as u32).collect()
}

fn i8s<const N: usize>(values: &[i8; N]) -> Vec<i32> {
	values.iter().map(|&value| value as i32).collect()
}

fn wrong_version(expected: &str, actual: &rbx::Mesh) -> Error {
	Error::from_reason(format!(
		"expected Roblox mesh version {expected}, got version {}",
		mesh_version_name(actual)
	))
}

// ---- Shared mesh primitives -------------------------------------------------

#[napi(object, js_name = "Vertex1")]
pub struct Vertex1Binding {
	pub pos: Vec<f64>,
	pub norm: Vec<f64>,
	/// All three texture values stored by mesh v1.
	pub tex: Vec<f64>,
}

impl From<&rbx::Vertex1> for Vertex1Binding {
	fn from(value: &rbx::Vertex1) -> Self {
		Self {
			pos: f32s(&value.pos),
			norm: f32s(&value.norm),
			tex: f32s(&value.tex),
		}
	}
}

#[napi(object, js_name = "Vertex2")]
pub struct Vertex2Binding {
	pub pos: Vec<f64>,
	pub norm: Vec<f64>,
	pub tex: Vec<f64>,
	pub tangent: Vec<i32>,
	pub color: Vec<u32>,
}

impl From<&rbx::Vertex2> for Vertex2Binding {
	fn from(value: &rbx::Vertex2) -> Self {
		Self {
			pos: f32s(&value.pos),
			norm: f32s(&value.norm),
			tex: f32s(&value.tex),
			tangent: i8s(&value.tangent),
			color: u8s(&value.color),
		}
	}
}

#[napi(object, js_name = "Vertex2Truncated")]
pub struct Vertex2TruncatedBinding {
	pub pos: Vec<f64>,
	pub norm: Vec<f64>,
	pub tex: Vec<f64>,
	pub tangent: Vec<i32>,
}

impl From<&rbx::Vertex2Truncated> for Vertex2TruncatedBinding {
	fn from(value: &rbx::Vertex2Truncated) -> Self {
		Self {
			pos: f32s(&value.pos),
			norm: f32s(&value.norm),
			tex: f32s(&value.tex),
			tangent: i8s(&value.tangent),
		}
	}
}

#[napi(
	discriminant = "kind",
	discriminant_case = "lowercase",
	js_name = "Vertices2"
)]
pub enum Vertices2Binding {
	Full {
		vertices: Vec<Vertex2Binding>,
	},
	Truncated {
		vertices: Vec<Vertex2TruncatedBinding>,
	},
}

impl From<&rbx::Vertices2> for Vertices2Binding {
	fn from(value: &rbx::Vertices2) -> Self {
		match value {
			rbx::Vertices2::Full(vertices) => Self::Full {
				vertices: vertices.iter().map(Vertex2Binding::from).collect(),
			},
			rbx::Vertices2::Truncated(vertices) => Self::Truncated {
				vertices: vertices.iter().map(Vertex2TruncatedBinding::from).collect(),
			},
		}
	}
}

#[napi(array, js_name = "Face2")]
pub struct Face2Binding(pub u32, pub u32, pub u32);

impl From<&rbx::Face2> for Face2Binding {
	fn from(value: &rbx::Face2) -> Self {
		Self(value.0[0].0, value.0[1].0, value.0[2].0)
	}
}

// ---- Union graphics CSGMDL2/4 ---------------------------------------------

#[napi(object, js_name = "UnionHash")]
pub struct UnionHashBinding {
	pub hash: Buffer,
	pub unknown: Buffer,
}

impl From<&graphics::Hash> for UnionHashBinding {
	fn from(value: &graphics::Hash) -> Self {
		Self {
			hash: value.hash.to_vec().into(),
			unknown: value._unknown.to_vec().into(),
		}
	}
}

#[napi(object, js_name = "UnionVertex2")]
pub struct UnionVertex2Binding {
	pub pos: Vec<f64>,
	pub norm: Vec<f64>,
	pub color: Vec<u32>,
	pub normal_id: u32,
	pub tex: Vec<f64>,
	pub tangent: Vec<f64>,
}

impl From<&graphics::Vertex> for UnionVertex2Binding {
	fn from(value: &graphics::Vertex) -> Self {
		Self {
			pos: f32s(&value.pos),
			norm: f32s(&value.norm),
			color: u8s(&value.color),
			normal_id: u32::from(&value.normal_id),
			tex: f32s(&value.tex),
			tangent: f32s(&value.tangent),
		}
	}
}

#[napi(object, js_name = "UnionMesh2")]
pub struct UnionMesh2Binding {
	pub vertex_count: u32,
	pub face_count: u32,
	pub vertices: Vec<UnionVertex2Binding>,
	pub faces: Vec<Face2Binding>,
}

impl From<&graphics::Mesh2> for UnionMesh2Binding {
	fn from(value: &graphics::Mesh2) -> Self {
		Self {
			vertex_count: value.vertex_count,
			face_count: value.face_count,
			vertices: value
				.vertices
				.iter()
				.map(UnionVertex2Binding::from)
				.collect(),
			faces: value
				.faces
				.iter()
				.map(|face| Face2Binding(face[0].0, face[1].0, face[2].0))
				.collect(),
		}
	}
}

#[napi(object, js_name = "CsgMdl2")]
pub struct CsgMdl2Binding {
	pub hash: UnionHashBinding,
	pub mesh: UnionMesh2Binding,
}

impl From<&graphics::CSGMDL2> for CsgMdl2Binding {
	fn from(value: &graphics::CSGMDL2) -> Self {
		Self {
			hash: UnionHashBinding::from(&value.hash),
			mesh: UnionMesh2Binding::from(&value.mesh),
		}
	}
}

#[napi(object, js_name = "CsgMdl4")]
pub struct CsgMdl4Binding {
	pub hash: UnionHashBinding,
	pub mesh: UnionMesh2Binding,
	pub unknown1_count: u32,
	pub unknown1: Vec<u32>,
}

impl From<&graphics::CSGMDL4> for CsgMdl4Binding {
	fn from(value: &graphics::CSGMDL4) -> Self {
		Self {
			hash: UnionHashBinding::from(&value.hash),
			mesh: UnionMesh2Binding::from(&value.mesh),
			unknown1_count: value._unknown1_count,
			unknown1: value._unknown1_list.clone(),
		}
	}
}

#[napi(object, js_name = "CsgMdl5Faces")]
pub struct CsgMdl5FacesBinding {
	pub indices: Vec<u32>,
	pub unknown: Vec<Vec<u32>>,
}

impl From<&graphics::Faces5> for CsgMdl5FacesBinding {
	fn from(value: &graphics::Faces5) -> Self {
		Self {
			indices: value.indices.clone(),
			unknown: value._unknown.clone(),
		}
	}
}

#[napi(object, js_name = "CsgMdl5")]
pub struct CsgMdl5Binding {
	pub position_count: u32,
	pub normal_count: u32,
	pub normals_len: u32,
	pub color_count: u32,
	pub normal_id_count: u32,
	pub tex_count: u32,
	pub tangent_count: u32,
	pub tangents_len: u32,
	pub positions: Vec<Vec<f64>>,
	pub normals: Vec<Vec<f64>>,
	pub colors: Vec<Vec<u32>>,
	pub normal_ids: Vec<u32>,
	pub tex: Vec<Vec<f64>>,
	pub tangents: Vec<Vec<f64>>,
	pub faces: CsgMdl5FacesBinding,
}

impl From<&graphics::CSGMDL5> for CsgMdl5Binding {
	fn from(value: &graphics::CSGMDL5) -> Self {
		Self {
			position_count: value.pos_count as u32,
			normal_count: value.normals_count as u32,
			normals_len: value.normals_len,
			color_count: value.color_count as u32,
			normal_id_count: value.normal_id_count as u32,
			tex_count: value.tex_count as u32,
			tangent_count: value.tangents_count as u32,
			tangents_len: value.tangents_len,
			positions: value.positions.iter().map(f32s).collect(),
			normals: value.normals.iter().map(|value| f32s(&value.0)).collect(),
			colors: value.colors.iter().map(u8s).collect(),
			normal_ids: value
				.normal_ids
				.iter()
				.map(|value| u8::from(value) as u32)
				.collect(),
			tex: value.tex.iter().map(f32s).collect(),
			tangents: value.tangents.iter().map(|value| f32s(&value.0)).collect(),
			faces: CsgMdl5FacesBinding::from(&value.faces),
		}
	}
}

#[napi(transparent, js_name = "Lod3")]
pub struct Lod3Binding(pub u32);

impl From<&rbx::Lod3> for Lod3Binding {
	fn from(value: &rbx::Lod3) -> Self {
		Self(value.0)
	}
}

#[napi(string_enum, js_name = "LodType4")]
pub enum LodType4Binding {
	None,
	Unknown,
	RbxSimplifier,
	ZeuxMeshOptimizer,
	Type4,
}

impl From<&rbx::LodType4> for LodType4Binding {
	fn from(value: &rbx::LodType4) -> Self {
		match value {
			rbx::LodType4::None => Self::None,
			rbx::LodType4::Unknown => Self::Unknown,
			rbx::LodType4::RbxSimplifier => Self::RbxSimplifier,
			rbx::LodType4::ZeuxMeshOptimizer => Self::ZeuxMeshOptimizer,
			rbx::LodType4::Type4 => Self::Type4,
		}
	}
}

#[napi(object, js_name = "Envelope4")]
pub struct Envelope4Binding {
	pub bones: Vec<u32>,
	pub weights: Vec<u32>,
}

impl From<&rbx::Envelope4> for Envelope4Binding {
	fn from(value: &rbx::Envelope4) -> Self {
		Self {
			bones: u8s(&value.bones),
			weights: u8s(&value.weights),
		}
	}
}

#[napi(object, js_name = "CFrame4")]
pub struct CFrame4Binding {
	pub r00: f64,
	pub r01: f64,
	pub r02: f64,
	pub r10: f64,
	pub r11: f64,
	pub r12: f64,
	pub r20: f64,
	pub r21: f64,
	pub r22: f64,
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

impl From<&rbx::CFrame4> for CFrame4Binding {
	fn from(value: &rbx::CFrame4) -> Self {
		Self {
			r00: value.r00 as f64,
			r01: value.r01 as f64,
			r02: value.r02 as f64,
			r10: value.r10 as f64,
			r11: value.r11 as f64,
			r12: value.r12 as f64,
			r20: value.r20 as f64,
			r21: value.r21 as f64,
			r22: value.r22 as f64,
			x: value.x as f64,
			y: value.y as f64,
			z: value.z as f64,
		}
	}
}

#[napi(object, js_name = "Bone4")]
pub struct Bone4Binding {
	pub bone_name_pos: u32,
	pub parent: Option<u32>,
	pub lod_parent: Option<u32>,
	pub cull_distance: f64,
	pub cframe: CFrame4Binding,
}

impl From<&rbx::Bone4> for Bone4Binding {
	fn from(value: &rbx::Bone4) -> Self {
		Self {
			bone_name_pos: value.bone_name_pos,
			parent: value.parent.get().map(u32::from),
			lod_parent: value.lod_parent.get().map(u32::from),
			cull_distance: value.cull_distance as f64,
			cframe: CFrame4Binding::from(&value.cframe),
		}
	}
}

#[napi(object, js_name = "Subset4")]
pub struct Subset4Binding {
	pub faces_offset: u32,
	pub faces_len: u32,
	pub vertices_offset: u32,
	pub vertices_len: u32,
	pub bone_count: u32,
	/// `null` means the serialized 0xffff sentinel.
	pub bones: Vec<Option<u32>>,
}

impl From<&rbx::Subset4> for Subset4Binding {
	fn from(value: &rbx::Subset4) -> Self {
		Self {
			faces_offset: value.faces_offset,
			faces_len: value.faces_len,
			vertices_offset: value.vertices_offset,
			vertices_len: value.vertices_len,
			bone_count: value.bone_count,
			bones: value
				.bones
				.iter()
				.map(|bone| bone.get().map(u32::from))
				.collect(),
		}
	}
}

// ---- Mesh v1 ---------------------------------------------------------------

#[napi(string_enum, js_name = "Revision1")]
pub enum Revision1Binding {
	Version100,
	Version101,
}

impl From<&rbx::Revision1> for Revision1Binding {
	fn from(value: &rbx::Revision1) -> Self {
		match value {
			rbx::Revision1::Version100 => Self::Version100,
			rbx::Revision1::Version101 => Self::Version101,
		}
	}
}

#[napi(object, js_name = "Mesh1")]
pub struct Mesh1Binding {
	pub revision: Revision1Binding,
	pub vertices: Vec<Vertex1Binding>,
}

impl From<&rbx::Mesh1> for Mesh1Binding {
	fn from(value: &rbx::Mesh1) -> Self {
		Self {
			revision: Revision1Binding::from(&value.revision),
			vertices: value.vertices.iter().map(Vertex1Binding::from).collect(),
		}
	}
}

// ---- Mesh v2 ---------------------------------------------------------------

#[napi(string_enum, js_name = "Revision2")]
pub enum Revision2Binding {
	Version200,
}

#[napi(object, js_name = "Mesh2")]
pub struct Mesh2Binding {
	pub revision: Revision2Binding,
	/// Serialized vertex size: 36 for truncated vertices, 40 for full vertices.
	pub size_of_vertex: u32,
	pub vertex_count: u32,
	pub face_count: u32,
	pub vertices: Vertices2Binding,
	pub faces: Vec<Face2Binding>,
}

impl From<&rbx::Mesh2> for Mesh2Binding {
	fn from(value: &rbx::Mesh2) -> Self {
		let size_of_vertex = match &value.vertices {
			rbx::Vertices2::Full(_) => 40,
			rbx::Vertices2::Truncated(_) => 36,
		};
		Self {
			revision: Revision2Binding::Version200,
			size_of_vertex,
			vertex_count: value.vertex_count,
			face_count: value.face_count,
			vertices: Vertices2Binding::from(&value.vertices),
			faces: value.faces.iter().map(Face2Binding::from).collect(),
		}
	}
}

// ---- Mesh v3 ---------------------------------------------------------------

#[napi(string_enum, js_name = "Revision3")]
pub enum Revision3Binding {
	Version300,
	Version301,
}

impl From<&rbx::Revision3> for Revision3Binding {
	fn from(value: &rbx::Revision3) -> Self {
		match value {
			rbx::Revision3::Version300 => Self::Version300,
			rbx::Revision3::Version301 => Self::Version301,
		}
	}
}

#[napi(object, js_name = "Mesh3")]
pub struct Mesh3Binding {
	pub revision: Revision3Binding,
	/// Serialized vertex size: 36 for truncated vertices, 40 for full vertices.
	pub size_of_vertex: u32,
	pub lod_count: u32,
	pub vertex_count: u32,
	pub face_count: u32,
	pub vertices: Vertices2Binding,
	pub faces: Vec<Face2Binding>,
	pub lods: Vec<Lod3Binding>,
}

impl From<&rbx::Mesh3> for Mesh3Binding {
	fn from(value: &rbx::Mesh3) -> Self {
		let size_of_vertex = match &value.vertices {
			rbx::Vertices2::Full(_) => 40,
			rbx::Vertices2::Truncated(_) => 36,
		};
		Self {
			revision: Revision3Binding::from(&value.revision),
			size_of_vertex,
			lod_count: value.lod_count as u32,
			vertex_count: value.vertex_count,
			face_count: value.face_count,
			vertices: Vertices2Binding::from(&value.vertices),
			faces: value.faces.iter().map(Face2Binding::from).collect(),
			lods: value.lods.iter().map(Lod3Binding::from).collect(),
		}
	}
}

// ---- Mesh v4 ---------------------------------------------------------------

#[napi(string_enum, js_name = "Revision4")]
pub enum Revision4Binding {
	Version400,
	Version401,
}

impl From<&rbx::Revision4> for Revision4Binding {
	fn from(value: &rbx::Revision4) -> Self {
		match value {
			rbx::Revision4::Version400 => Self::Version400,
			rbx::Revision4::Version401 => Self::Version401,
		}
	}
}

#[napi(object, js_name = "Mesh4")]
pub struct Mesh4Binding {
	pub revision: Revision4Binding,
	pub lod_type: LodType4Binding,
	pub vertex_count: u32,
	pub face_count: u32,
	pub lod_count: u32,
	pub bone_count: u32,
	pub bone_names_len: u32,
	pub subset_count: u32,
	pub lod_hq_count: u32,
	pub padding: u32,
	pub vertices: Vec<Vertex2Binding>,
	pub envelopes: Vec<Envelope4Binding>,
	pub faces: Vec<Face2Binding>,
	pub lods: Vec<Lod3Binding>,
	pub bones: Vec<Bone4Binding>,
	pub bone_names: Buffer,
	pub subsets: Vec<Subset4Binding>,
}

impl From<&rbx::Mesh4> for Mesh4Binding {
	fn from(value: &rbx::Mesh4) -> Self {
		Self {
			revision: Revision4Binding::from(&value.revision),
			lod_type: LodType4Binding::from(&value.lod_type),
			vertex_count: value.vertex_count,
			face_count: value.face_count,
			lod_count: value.lod_count as u32,
			bone_count: value.bone_count as u32,
			bone_names_len: value.bone_names_len,
			subset_count: value.subset_count as u32,
			lod_hq_count: value.lod_hq_count as u32,
			padding: value._padding as u32,
			vertices: value.vertices.iter().map(Vertex2Binding::from).collect(),
			envelopes: value.envelopes.iter().map(Envelope4Binding::from).collect(),
			faces: value.faces.iter().map(Face2Binding::from).collect(),
			lods: value.lods.iter().map(Lod3Binding::from).collect(),
			bones: value.bones.iter().map(Bone4Binding::from).collect(),
			bone_names: value.bone_names.clone().into(),
			subsets: value.subsets.iter().map(Subset4Binding::from).collect(),
		}
	}
}

// ---- Mesh v5 ---------------------------------------------------------------

#[napi(string_enum, js_name = "Revision5")]
pub enum Revision5Binding {
	Version500,
}

#[napi(string_enum, js_name = "FacsFormat5")]
pub enum FacsFormat5Binding {
	Format1,
}

#[napi(
	discriminant = "kind",
	discriminant_case = "lowercase",
	js_name = "QuantizedMatrix5"
)]
pub enum QuantizedMatrix5Binding {
	Raw {
		x: u32,
		y: u32,
		matrix: Vec<f64>,
	},
	Quantized {
		x: u32,
		y: u32,
		lerp0: f64,
		lerp1: f64,
		matrix: Vec<u32>,
	},
}

impl From<&rbx::QuantizedMatrix5> for QuantizedMatrix5Binding {
	fn from(value: &rbx::QuantizedMatrix5) -> Self {
		match value {
			rbx::QuantizedMatrix5::Raw { x, y, matrix } => Self::Raw {
				x: *x,
				y: *y,
				matrix: matrix.iter().map(|&value| value as f64).collect(),
			},
			rbx::QuantizedMatrix5::Quantized {
				x,
				y,
				lerp0,
				lerp1,
				matrix,
			} => Self::Quantized {
				x: *x,
				y: *y,
				lerp0: *lerp0 as f64,
				lerp1: *lerp1 as f64,
				matrix: matrix.iter().map(|&value| value as u32).collect(),
			},
		}
	}
}

#[napi(object, js_name = "QuantizedTransforms5")]
pub struct QuantizedTransforms5Binding {
	pub px: QuantizedMatrix5Binding,
	pub py: QuantizedMatrix5Binding,
	pub pz: QuantizedMatrix5Binding,
	pub rx: QuantizedMatrix5Binding,
	pub ry: QuantizedMatrix5Binding,
	pub rz: QuantizedMatrix5Binding,
}

impl From<&rbx::QuantizedTransforms5> for QuantizedTransforms5Binding {
	fn from(value: &rbx::QuantizedTransforms5) -> Self {
		Self {
			px: QuantizedMatrix5Binding::from(&value.px),
			py: QuantizedMatrix5Binding::from(&value.py),
			pz: QuantizedMatrix5Binding::from(&value.pz),
			rx: QuantizedMatrix5Binding::from(&value.rx),
			ry: QuantizedMatrix5Binding::from(&value.ry),
			rz: QuantizedMatrix5Binding::from(&value.rz),
		}
	}
}

#[napi(array, js_name = "TwoPoseCorrective5")]
pub struct TwoPoseCorrective5Binding(pub u32, pub u32);

impl From<&rbx::TwoPoseCorrective5> for TwoPoseCorrective5Binding {
	fn from(value: &rbx::TwoPoseCorrective5) -> Self {
		Self(value.0[0].0 as u32, value.0[1].0 as u32)
	}
}

#[napi(array, js_name = "ThreePoseCorrective5")]
pub struct ThreePoseCorrective5Binding(pub u32, pub u32, pub u32);

impl From<&rbx::ThreePoseCorrective5> for ThreePoseCorrective5Binding {
	fn from(value: &rbx::ThreePoseCorrective5) -> Self {
		Self(
			value.0[0].0 as u32,
			value.0[1].0 as u32,
			value.0[2].0 as u32,
		)
	}
}

#[napi(object, js_name = "Facs5")]
pub struct Facs5Binding {
	pub face_bone_names_len: u32,
	pub face_control_names_len: u32,
	/// Serialized byte length of all six quantized transform matrices.
	pub quantized_transforms_len: i64,
	/// Serialized byte length (not element count).
	pub two_pose_correctives_len: u32,
	/// Serialized byte length (not element count).
	pub three_pose_correctives_len: u32,
	pub face_bone_names: Buffer,
	pub face_control_names: Buffer,
	pub quantized_transforms: QuantizedTransforms5Binding,
	pub two_pose_correctives: Vec<TwoPoseCorrective5Binding>,
	pub three_pose_correctives: Vec<ThreePoseCorrective5Binding>,
}

impl From<&rbx::Facs5> for Facs5Binding {
	fn from(value: &rbx::Facs5) -> Self {
		Self {
			face_bone_names_len: value.face_bone_names_len,
			face_control_names_len: value.face_control_names_len,
			quantized_transforms_len: value.quantized_transforms_len as i64,
			two_pose_correctives_len: value.two_pose_correctives_len,
			three_pose_correctives_len: value.three_pose_correctives_len,
			face_bone_names: value.face_bone_names.clone().into(),
			face_control_names: value.face_control_names.clone().into(),
			quantized_transforms: QuantizedTransforms5Binding::from(&value.quantized_transforms),
			two_pose_correctives: value
				.two_pose_correctives
				.iter()
				.map(TwoPoseCorrective5Binding::from)
				.collect(),
			three_pose_correctives: value
				.three_pose_correctives
				.iter()
				.map(ThreePoseCorrective5Binding::from)
				.collect(),
		}
	}
}

#[napi(object, js_name = "Mesh5")]
pub struct Mesh5Binding {
	pub revision: Revision5Binding,
	pub lod_type: LodType4Binding,
	pub vertex_count: u32,
	pub face_count: u32,
	pub lod_count: u32,
	pub bone_count: u32,
	pub bone_names_len: u32,
	pub subset_count: u32,
	pub lod_hq_count: u32,
	pub facs_format: FacsFormat5Binding,
	pub sizeof_facs: u32,
	pub vertices: Vec<Vertex2Binding>,
	pub envelopes: Vec<Envelope4Binding>,
	pub faces: Vec<Face2Binding>,
	pub lods: Vec<Lod3Binding>,
	pub bones: Vec<Bone4Binding>,
	pub bone_names: Buffer,
	pub subsets: Vec<Subset4Binding>,
	pub facs: Facs5Binding,
}

impl From<&rbx::Mesh5> for Mesh5Binding {
	fn from(value: &rbx::Mesh5) -> Self {
		Self {
			revision: Revision5Binding::Version500,
			lod_type: LodType4Binding::from(&value.lod_type),
			vertex_count: value.vertex_count,
			face_count: value.face_count,
			lod_count: value.lod_count as u32,
			bone_count: value.bone_count as u32,
			bone_names_len: value.bone_names_len,
			subset_count: value.subset_count as u32,
			lod_hq_count: value.lod_hq_count as u32,
			facs_format: FacsFormat5Binding::Format1,
			sizeof_facs: value.sizeof_facs,
			vertices: value.vertices.iter().map(Vertex2Binding::from).collect(),
			envelopes: value.envelopes.iter().map(Envelope4Binding::from).collect(),
			faces: value.faces.iter().map(Face2Binding::from).collect(),
			lods: value.lods.iter().map(Lod3Binding::from).collect(),
			bones: value.bones.iter().map(Bone4Binding::from).collect(),
			bone_names: value.bone_names.clone().into(),
			subsets: value.subsets.iter().map(Subset4Binding::from).collect(),
			facs: Facs5Binding::from(&value.facs),
		}
	}
}

// ---- Mesh v7 ---------------------------------------------------------------

#[napi(string_enum, js_name = "Revision7")]
pub enum Revision7Binding {
	Version700,
}

#[napi(object, js_name = "Coremesh1")]
pub struct Coremesh1Binding {
	pub len: u32,
	pub vertex_count: u32,
	pub vertices: Vec<Vertex2Binding>,
	pub face_count: u32,
	pub faces: Vec<Face2Binding>,
}

impl From<&rbx::Coremesh1> for Coremesh1Binding {
	fn from(value: &rbx::Coremesh1) -> Self {
		Self {
			len: value.len,
			vertex_count: value.vertex_count,
			vertices: value.vertices.iter().map(Vertex2Binding::from).collect(),
			face_count: value.face_count,
			faces: value.faces.iter().map(Face2Binding::from).collect(),
		}
	}
}

#[napi(object, js_name = "Coremesh2")]
pub struct Coremesh2Binding {
	pub draco_len: u32,
	pub draco: Buffer,
}

impl From<&rbx::Coremesh2> for Coremesh2Binding {
	fn from(value: &rbx::Coremesh2) -> Self {
		Self {
			draco_len: value.draco_len,
			draco: value.draco.clone().into(),
		}
	}
}

#[napi(
	discriminant = "kind",
	discriminant_case = "lowercase",
	js_name = "Coremesh"
)]
pub enum CoremeshBinding {
	V1 { coremesh: Coremesh1Binding },
	V2 { coremesh: Coremesh2Binding },
}

impl From<&rbx::Coremesh> for CoremeshBinding {
	fn from(value: &rbx::Coremesh) -> Self {
		match value {
			rbx::Coremesh::V1(coremesh) => Self::V1 {
				coremesh: Coremesh1Binding::from(coremesh),
			},
			rbx::Coremesh::V2(coremesh) => Self::V2 {
				coremesh: Coremesh2Binding::from(coremesh),
			},
		}
	}
}

#[napi(object, js_name = "Lods7")]
pub struct Lods7Binding {
	pub unknown1: u32,
	pub unknown2: u32,
	pub unknown3_len: u32,
	pub unknown3: Buffer,
}

impl From<&rbx::Lods> for Lods7Binding {
	fn from(value: &rbx::Lods) -> Self {
		Self {
			unknown1: value.unknown1,
			unknown2: value.unknown2,
			unknown3_len: value.unknown3_len,
			unknown3: value.unknown3.clone().into(),
		}
	}
}

#[napi(object, js_name = "Skinning7")]
pub struct Skinning7Binding {
	pub len: u32,
	pub envelope_count: u32,
	pub envelopes: Vec<Envelope4Binding>,
	pub bone_count: u32,
	pub bones: Vec<Bone4Binding>,
	pub bone_names_len: u32,
	pub bone_names: Buffer,
	pub subset_count: u32,
	pub subsets: Vec<Subset4Binding>,
}

impl From<&rbx::Skinning> for Skinning7Binding {
	fn from(value: &rbx::Skinning) -> Self {
		Self {
			len: value.len,
			envelope_count: value.envelope_count,
			envelopes: value.envelopes.iter().map(Envelope4Binding::from).collect(),
			bone_count: value.bone_count,
			bones: value.bones.iter().map(Bone4Binding::from).collect(),
			bone_names_len: value.bone_names_len,
			bone_names: value.bone_names.clone().into(),
			subset_count: value.subset_count,
			subsets: value.subsets.iter().map(Subset4Binding::from).collect(),
		}
	}
}

#[napi(object, js_name = "Facs7")]
pub struct Facs7Binding {
	pub bytes_remaining1: u32,
	pub bytes_remaining2: u32,
	pub face_bone_names_len: u32,
	pub face_control_names_len: u32,
	pub unknown_count5: u32,
	pub two_pose_correctives_len: u32,
	pub three_pose_correctives_len: u32,
	pub face_bone_names: Buffer,
	pub face_control_names: Buffer,
	pub quantized_transforms: QuantizedTransforms5Binding,
	pub two_pose_correctives: Vec<TwoPoseCorrective5Binding>,
	pub three_pose_correctives: Vec<ThreePoseCorrective5Binding>,
}

impl From<&rbx::Facs7> for Facs7Binding {
	fn from(value: &rbx::Facs7) -> Self {
		Self {
			bytes_remaining1: value.bytes_remaining1,
			bytes_remaining2: value.bytes_remaining2,
			face_bone_names_len: value.face_bone_names_len,
			face_control_names_len: value.face_control_names_len,
			unknown_count5: value.unknown_count5,
			two_pose_correctives_len: value.two_pose_correctives_len,
			three_pose_correctives_len: value.three_pose_correctives_len,
			face_bone_names: value.face_bone_names.clone().into(),
			face_control_names: value.face_control_names.clone().into(),
			quantized_transforms: QuantizedTransforms5Binding::from(&value.quantized_transforms),
			two_pose_correctives: value
				.two_pose_correctives
				.iter()
				.map(TwoPoseCorrective5Binding::from)
				.collect(),
			three_pose_correctives: value
				.three_pose_correctives
				.iter()
				.map(ThreePoseCorrective5Binding::from)
				.collect(),
		}
	}
}

#[napi(object, js_name = "Mesh7Ext")]
pub struct Mesh7ExtBinding {
	pub skinning_count: u32,
	pub skinnings: Vec<Skinning7Binding>,
	pub facs: Facs7Binding,
}

impl From<&rbx::Mesh7Ext> for Mesh7ExtBinding {
	fn from(value: &rbx::Mesh7Ext) -> Self {
		Self {
			skinning_count: value.skinning_count,
			skinnings: value.skinnings.iter().map(Skinning7Binding::from).collect(),
			facs: Facs7Binding::from(&value.facs),
		}
	}
}

#[napi(object, js_name = "Mesh7")]
pub struct Mesh7Binding {
	pub revision: Revision7Binding,
	pub coremesh: CoremeshBinding,
	/// Decoded vertices, including when coremesh v2 uses Draco internally.
	pub vertices: Vec<Vertex2Binding>,
	/// Decoded faces, including when coremesh v2 uses Draco internally.
	pub faces: Vec<Face2Binding>,
	pub lods: Lods7Binding,
	pub ext: Option<Mesh7ExtBinding>,
}

impl From<&rbx::Mesh7> for Mesh7Binding {
	fn from(value: &rbx::Mesh7) -> Self {
		Self {
			revision: Revision7Binding::Version700,
			coremesh: CoremeshBinding::from(&value.coremesh),
			vertices: value.vertices.iter().map(Vertex2Binding::from).collect(),
			faces: value.faces.iter().map(Face2Binding::from).collect(),
			lods: Lods7Binding::from(&value.lods),
			ext: value.ext.as_ref().map(Mesh7ExtBinding::from),
		}
	}
}

// ---- Versioned low-level API ----------------------------------------------

#[napi(
	discriminant = "version",
	discriminant_case = "lowercase",
	js_name = "MeshVersioned"
)]
pub enum MeshVersionedBinding {
	V1 { mesh: Mesh1Binding },
	V2 { mesh: Mesh2Binding },
	V3 { mesh: Mesh3Binding },
	V4 { mesh: Mesh4Binding },
	V5 { mesh: Mesh5Binding },
	V7 { mesh: Mesh7Binding },
}

impl From<&rbx::Mesh> for MeshVersionedBinding {
	fn from(value: &rbx::Mesh) -> Self {
		match value {
			rbx::Mesh::V1(mesh) => Self::V1 {
				mesh: Mesh1Binding::from(mesh),
			},
			rbx::Mesh::V2(mesh) => Self::V2 {
				mesh: Mesh2Binding::from(mesh),
			},
			rbx::Mesh::V3(mesh) => Self::V3 {
				mesh: Mesh3Binding::from(mesh),
			},
			rbx::Mesh::V4(mesh) => Self::V4 {
				mesh: Mesh4Binding::from(mesh),
			},
			rbx::Mesh::V5(mesh) => Self::V5 {
				mesh: Mesh5Binding::from(mesh),
			},
			rbx::Mesh::V7(mesh) => Self::V7 {
				mesh: Mesh7Binding::from(mesh),
			},
		}
	}
}

/// Parse any supported mesh and retain its exact version-specific structure.
#[napi]
pub fn parse_mesh_versioned(data: &[u8]) -> Result<MeshVersionedBinding> {
	let mesh = parse_versioned(data)?;
	Ok(MeshVersionedBinding::from(&mesh))
}

#[napi]
pub fn parse_mesh1(data: &[u8]) -> Result<Mesh1Binding> {
	let mesh = parse_versioned(data)?;
	match &mesh {
		rbx::Mesh::V1(value) => Ok(Mesh1Binding::from(value)),
		_ => Err(wrong_version("1", &mesh)),
	}
}

#[napi]
pub fn parse_mesh2(data: &[u8]) -> Result<Mesh2Binding> {
	let mesh = parse_versioned(data)?;
	match &mesh {
		rbx::Mesh::V2(value) => Ok(Mesh2Binding::from(value)),
		_ => Err(wrong_version("2", &mesh)),
	}
}

#[napi]
pub fn parse_mesh3(data: &[u8]) -> Result<Mesh3Binding> {
	let mesh = parse_versioned(data)?;
	match &mesh {
		rbx::Mesh::V3(value) => Ok(Mesh3Binding::from(value)),
		_ => Err(wrong_version("3", &mesh)),
	}
}

#[napi]
pub fn parse_mesh4(data: &[u8]) -> Result<Mesh4Binding> {
	let mesh = parse_versioned(data)?;
	match &mesh {
		rbx::Mesh::V4(value) => Ok(Mesh4Binding::from(value)),
		_ => Err(wrong_version("4", &mesh)),
	}
}

#[napi]
pub fn parse_mesh5(data: &[u8]) -> Result<Mesh5Binding> {
	let mesh = parse_versioned(data)?;
	match &mesh {
		rbx::Mesh::V5(value) => Ok(Mesh5Binding::from(value)),
		_ => Err(wrong_version("5", &mesh)),
	}
}

#[napi]
pub fn parse_mesh7(data: &[u8]) -> Result<Mesh7Binding> {
	let mesh = parse_versioned(data)?;
	match &mesh {
		rbx::Mesh::V7(value) => Ok(Mesh7Binding::from(value)),
		_ => Err(wrong_version("7", &mesh)),
	}
}

fn parse_union_graphics(data: &[u8]) -> Result<graphics::UnionGraphics> {
	rbx_mesh::read_union_graphics_versioned(std::io::Cursor::new(data))
		.map_err(|err| Error::from_reason(format!("failed to parse Roblox union graphics: {err}")))
}

fn union_graphics_version_name(union: &graphics::UnionGraphics) -> &'static str {
	match union {
		graphics::UnionGraphics::CSGK(_) => "CSGK",
		graphics::UnionGraphics::V2(_) => "CSGMDL2",
		graphics::UnionGraphics::V4(_) => "CSGMDL4",
		graphics::UnionGraphics::V5(_) => "CSGMDL5",
	}
}

fn wrong_union_graphics_version(expected: &str, actual: &graphics::UnionGraphics) -> Error {
	Error::from_reason(format!(
		"expected Roblox union graphics version {expected}, got {}",
		union_graphics_version_name(actual)
	))
}

#[napi]
pub fn parse_csg_mdl2(data: &[u8]) -> Result<CsgMdl2Binding> {
	let union = parse_union_graphics(data)?;
	match &union {
		graphics::UnionGraphics::V2(value) => Ok(CsgMdl2Binding::from(value)),
		_ => Err(wrong_union_graphics_version("CSGMDL2", &union)),
	}
}

#[napi]
pub fn parse_csg_mdl4(data: &[u8]) -> Result<CsgMdl4Binding> {
	let union = parse_union_graphics(data)?;
	match &union {
		graphics::UnionGraphics::V4(value) => Ok(CsgMdl4Binding::from(value)),
		_ => Err(wrong_union_graphics_version("CSGMDL4", &union)),
	}
}

#[napi]
pub fn parse_csg_mdl5(data: &[u8]) -> Result<CsgMdl5Binding> {
	let union = parse_union_graphics(data)?;
	match &union {
		graphics::UnionGraphics::V5(value) => Ok(CsgMdl5Binding::from(value)),
		_ => Err(wrong_union_graphics_version("CSGMDL5", &union)),
	}
}
