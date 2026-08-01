use super::v2::{Face2, Vertex2};
use super::v4::{Bone4, Envelope4, Subset4};
use super::v5::{QuantizedTransforms5, ThreePoseCorrective5, TwoPoseCorrective5};

fn read_attribute_bytes(
	attribute: &draco_core::geometry_attribute::PointAttribute,
	point: usize,
	components: usize,
) -> Option<Vec<u8>> {
	let value = attribute.mapped_index(draco_core::PointIndex(point as u32));
	let stride = usize::try_from(attribute.byte_stride()).ok()?;
	let offset = usize::try_from(value.0).ok()?.checked_mul(stride)?;
	let end = offset.checked_add(components)?;
	attribute.buffer().data().get(offset..end).map(Vec::from)
}

fn read_attribute_f32(
	attribute: &draco_core::geometry_attribute::PointAttribute,
	point: usize,
	components: usize,
) -> Option<Vec<f32>> {
	if attribute.data_type() != draco_core::DataType::Float32
		|| attribute.num_components() as usize != components
	{
		return None;
	}
	let bytes = read_attribute_bytes(attribute, point, components * size_of::<f32>())?;
	Some(
		bytes
			.chunks_exact(size_of::<f32>())
			.map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
			.collect(),
	)
}

fn read_attribute_u8(
	attribute: &draco_core::geometry_attribute::PointAttribute,
	point: usize,
	components: usize,
) -> Option<Vec<u8>> {
	if attribute.data_type() != draco_core::DataType::Uint8
		|| attribute.num_components() as usize != components
	{
		return None;
	}
	read_attribute_bytes(attribute, point, components)
}

fn decode_vertices(draco: &[u8]) -> Vec<Vertex2> {
	let Some(draco_stream) = draco.get(4..) else {
		return Vec::new();
	};
	let mut buffer = draco_core::DecoderBuffer::new(draco_stream);
	let mut decoded = draco_core::Mesh::new();
	if draco_core::MeshDecoder::new()
		.decode(&mut buffer, &mut decoded)
		.is_err()
	{
		return Vec::new();
	}

	let Some(position) = decoded.attribute_by_unique_id(0) else {
		return Vec::new();
	};
	let Some(normal) = decoded.attribute_by_unique_id(1) else {
		return Vec::new();
	};
	let Some(tex) = decoded.attribute_by_unique_id(2) else {
		return Vec::new();
	};
	let Some(tangent) = decoded.attribute_by_unique_id(3) else {
		return Vec::new();
	};
	let Some(color) = decoded.attribute_by_unique_id(4) else {
		return Vec::new();
	};

	(0..decoded.num_points())
		.filter_map(|point| {
			let position = read_attribute_f32(position, point, 3)?;
			let normal = read_attribute_f32(normal, point, 3)?;
			let tex = read_attribute_f32(tex, point, 2)?;
			let tangent = read_attribute_u8(tangent, point, 4)?;
			let color = read_attribute_u8(color, point, 4)?;
			Some(Vertex2 {
				pos: position.try_into().ok()?,
				norm: normal.try_into().ok()?,
				tex: tex.try_into().ok()?,
				tangent: tangent
					.into_iter()
					.map(|value| value as i8)
					.collect::<Vec<_>>()
					.try_into()
					.ok()?,
				color: color.try_into().ok()?,
			})
		})
		.collect()
}

fn decode_faces(draco: &[u8]) -> Vec<Face2> {
	let Some(draco_stream) = draco.get(4..) else {
		return Vec::new();
	};
	let mut buffer = draco_core::DecoderBuffer::new(draco_stream);
	let mut decoded = draco_core::Mesh::new();
	if draco_core::MeshDecoder::new()
		.decode(&mut buffer, &mut decoded)
		.is_err()
	{
		return Vec::new();
	}

	(0..decoded.num_faces())
		.map(|face| {
			let face = decoded.face(draco_core::FaceIndex(face as u32));
			Face2([
				crate::mesh::VertexId2(face[0].0),
				crate::mesh::VertexId2(face[1].0),
				crate::mesh::VertexId2(face[2].0),
			])
		})
		.collect()
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub enum Revision7 {
	#[brw(magic = b"version 7.00")]
	Version700,
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub enum Coremesh {
	V1(Coremesh1),
	V2(Coremesh2),
}

#[binrw::binrw]
#[brw(little)]
#[brw(magic = b"COREMESH\x01\0\0\0")]
#[derive(Debug, Clone)]
pub struct Coremesh1 {
	#[br(temp)]
	#[bw(try_calc = (vertices.len()*size_of::<Vertex2>() + faces.len()*size_of::<Face2>()).try_into())]
	pub len: u32,
	#[br(temp)]
	#[bw(try_calc = vertices.len().try_into())]
	pub vertex_count: u32,
	#[br(count = vertex_count)]
	pub vertices: Vec<Vertex2>,
	#[br(temp)]
	#[bw(try_calc = faces.len().try_into())]
	pub face_count: u32,
	#[br(count = face_count)]
	pub faces: Vec<Face2>,
}

#[binrw::binrw]
#[brw(little)]
#[brw(magic = b"COREMESH\x02\0\0\0")]
#[derive(Debug, Clone)]
pub struct Coremesh2 {
	pub draco_len: u32,
	#[br(count = draco_len)]
	pub draco: Vec<u8>,
}

impl Coremesh2 {
	pub fn vertices(&self) -> Vec<Vertex2> {
		decode_vertices(&self.draco)
	}

	pub fn faces(&self) -> Vec<Face2> {
		decode_faces(&self.draco)
	}
}

#[binrw::binrw]
#[brw(little)]
#[brw(magic = b"LODS")]
#[derive(Debug, Clone)]
pub struct Lods {
	// version 6.00 LODS
	// pub lod_type: u16,
	// pub num_high_quality_lods: u8,
	// pub lod_offsets_count: u32,
	// #[br(count = lod_offsets_count)]
	// pub lod_offsets: Vec<u32>,
	pub unknown1: u32,     // 0, 0, 0, 0,
	pub unknown2: u32,     // 1, 0, 0, 0,
	pub unknown3_len: u32, // 15, 0, 0, 0,
	#[br(count = unknown3_len)]
	pub unknown3: Vec<u8>, // [0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct Skinning {
	pub len: u32,
	#[br(temp)]
	#[bw(try_calc = envelopes.len().try_into())]
	pub envelope_count: u32,
	#[br(count = envelope_count)]
	pub envelopes: Vec<Envelope4>,

	#[br(temp)]
	#[bw(try_calc=bones.len().try_into())]
	pub bone_count: u32,
	#[br(count=bone_count)]
	pub bones: Vec<Bone4>,

	#[br(temp)]
	#[bw(try_calc=bone_names.len().try_into())]
	pub bone_names_len: u32,
	#[br(count=bone_names_len)]
	pub bone_names: Vec<u8>,

	#[br(temp)]
	#[bw(try_calc=subsets.len().try_into())]
	pub subset_count: u32,
	#[br(count=subset_count)]
	pub subsets: Vec<Subset4>,
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct Facs7 {
	#[br(temp)]
	#[bw(ignore)]
	#[brw(magic = 0u32)]
	_ignore1: (),
	#[br(temp)]
	#[bw(ignore)]
	#[brw(magic = 1u32)]
	_ignore2: (),
	pub bytes_remaining1: u32, // 59186 -> remaining bytes in file after this number
	pub bytes_remaining2: u32, // 59182 -> remaining bytes in file after this number
	pub face_bone_names_len: u32, // 576
	pub face_control_names_len: u32, // 280
	pub unknown_count5: u32,   // 58068
	#[br(temp)]
	#[bw(ignore)]
	#[brw(magic = 0u32)]
	_ignore3: (),
	pub two_pose_correctives_len: u32,   // 192
	pub three_pose_correctives_len: u32, // 42
	#[br(count=face_bone_names_len)]
	pub face_bone_names: Vec<u8>,
	#[br(count=face_control_names_len)]
	pub face_control_names: Vec<u8>,
	pub quantized_transforms: QuantizedTransforms5,
	#[br(count=two_pose_correctives_len as usize/size_of::<TwoPoseCorrective5>())]
	pub two_pose_correctives: Vec<TwoPoseCorrective5>,
	#[br(count=three_pose_correctives_len as usize/size_of::<ThreePoseCorrective5>())]
	pub three_pose_correctives: Vec<ThreePoseCorrective5>,
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct Mesh7Ext {
	#[brw(magic = b"SKINNING")]
	#[br(temp)]
	#[bw(try_calc = skinnings.len().try_into())]
	pub skinning_count: u32,
	#[br(count = skinning_count)]
	pub skinnings: Vec<Skinning>,
	#[brw(magic = b"FACS")]
	pub facs: Facs7,
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct Mesh7 {
	pub revision: Revision7,
	#[br(temp)]
	#[bw(ignore)]
	#[brw(magic = b"\n")]
	_newline: (),
	pub coremesh: Coremesh,
	#[br(calc = match &coremesh {
		Coremesh::V1(coremesh1) => coremesh1.vertices.clone(),
		Coremesh::V2(coremesh2) => coremesh2.vertices(),
	})]
	#[bw(ignore)]
	pub vertices: Vec<Vertex2>,
	#[br(calc = match &coremesh {
		Coremesh::V1(coremesh1) => coremesh1.faces.clone(),
		Coremesh::V2(coremesh2) => coremesh2.faces(),
	})]
	#[bw(ignore)]
	pub faces: Vec<Face2>,
	// <- 0x27E2
	pub lods: Lods,
	#[br(try)]
	pub ext: Option<Mesh7Ext>,
}

fn _math() {
	const _A: u32 = 1660;
}

#[test]
fn read_mesh7_127279296594138() {
	use binrw::BinReaderExt;
	let data = std::fs::read("meshes/mesh7_127279296594138.bin").unwrap();
	let mut bytes = std::io::Cursor::new(data.as_slice());
	let mesh: Mesh7 = bytes.read_le().unwrap();
	println!("data.len() = {}", data.len());
	assert_eq!(data.len() as u64, bytes.position());
	assert_eq!(mesh.vertices.len(), 408);
	assert_eq!(mesh.faces.len(), 268);
	assert!(mesh.vertices.iter().all(|vertex| {
		vertex
			.pos
			.iter()
			.chain(vertex.norm.iter())
			.all(|value| value.is_finite())
	}));
	assert!(
		mesh.vertices
			.iter()
			.any(|vertex| vertex.pos.iter().any(|value| *value != 0.0))
	);
	assert!(mesh.vertices.iter().any(|vertex| {
		vertex.norm.iter().any(|value| *value != 0.0)
			&& vertex.tex.iter().any(|value| *value != 0.0)
			&& vertex.tangent.iter().any(|value| *value != 0)
			&& vertex.color.iter().any(|value| *value != 0)
	}));
	let bounds = mesh.vertices.iter().fold(
		([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]),
		|(mut min, mut max), vertex| {
			for axis in 0..3 {
				min[axis] = min[axis].min(vertex.pos[axis]);
				max[axis] = max[axis].max(vertex.pos[axis]);
			}
			(min, max)
		},
	);
	println!(
		"decoded vertices: count={}, first={:?}, bounds={bounds:?}",
		mesh.vertices.len(),
		mesh.vertices[0]
	);

	let Coremesh::V2(coremesh2) = mesh.coremesh else {
		panic!();
	};
	println!("lods = {:?}", mesh.lods);
	assert_eq!(coremesh2.draco.len(), 10181);
}

#[test]
fn read_mesh7_86389496539231() {
	use binrw::BinReaderExt;
	let data = std::fs::read("meshes/mesh7_86389496539231.bin").unwrap();
	let mut bytes = std::io::Cursor::new(data.as_slice());
	let _mesh: Mesh7 = bytes.read_le().unwrap();
	println!("data.len() = {}", data.len());
	assert_eq!(data.len() as u64, bytes.position());
}

#[test]
fn read_mesh7_112807239761722() {
	use binrw::BinReaderExt;
	let data = std::fs::read("meshes/mesh7_112807239761722.bin").unwrap();
	let mut bytes = std::io::Cursor::new(data.as_slice());
	let _mesh: Mesh7 = bytes.read_le().unwrap();
	println!("data.len() = {}", data.len());
	assert_eq!(data.len() as u64, bytes.position());
}

#[test]
fn read_mesh7_100025761449828_skinning() {
	use binrw::BinReaderExt;
	let data = std::fs::read("meshes/mesh7_100025761449828.bin").unwrap();
	let mut cursor = std::io::Cursor::new(data.as_slice());
	let _mesh: Mesh7 = cursor.read_le().unwrap();
	assert_eq!(data.len() as u64, cursor.position());
}
