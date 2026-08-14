use std::io::Cursor;

use napi::{Error, Result};

pub mod high_level;
pub mod low_level;

pub(crate) fn parse_versioned(data: &[u8]) -> Result<rbx_mesh::mesh::Mesh> {
	rbx_mesh::read_mesh_versioned(Cursor::new(data))
		.map_err(|err| Error::from_reason(format!("failed to parse Roblox mesh: {err}")))
}

pub(crate) fn mesh_version_name(mesh: &rbx_mesh::mesh::Mesh) -> &'static str {
	use rbx_mesh::mesh::Mesh;
	match mesh {
		Mesh::V1(_) => "1",
		Mesh::V2(_) => "2",
		Mesh::V3(_) => "3",
		Mesh::V4(_) => "4",
		Mesh::V5(_) => "5",
		Mesh::V7(_) => "7",
	}
}
