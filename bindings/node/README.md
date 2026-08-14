# rbx_mesh Node.js bindings

Native Node.js bindings for the parent `rbx_mesh` crate using napi-rs.

## Requirements

- Rust 1.94 or newer (required by the parent crate)
- Node.js 22.13 or newer (required by the current napi-rs CLI)

## Build

```sh
npm install
npm run build
```

The napi-rs CLI generates `index.js`, `index.d.ts`, and the platform-specific native addon.

## Install

```sh
npm install rbx-mesh
```

The native addon is distributed through napi-rs platform packages and supports Windows x64, Linux x64 (glibc and musl), macOS x64, and macOS arm64.

## High-level API

```js
const fs = require("node:fs");
const { parseMesh } = require("./index.js");

const mesh = parseMesh(fs.readFileSync("../../meshes/torso.mesh"));

console.log(mesh.version);
console.log(mesh.positions); // Float32Array, xyzxyz...
console.log(mesh.normals); // Float32Array, xyzxyz...
console.log(mesh.uvs); // Float32Array, uvuv...
console.log(mesh.tangents); // Int8Array | undefined, xyzwxyzw...
console.log(mesh.colors); // Uint8Array | undefined, rgbargba...
console.log(mesh.indices); // Uint32Array, triangle indices
```

## Low-level API

`parseMeshVersioned()` returns a discriminated union containing the version-specific format object. Dedicated helpers are also exported:

- `parseMesh1`
- `parseMesh2`
- `parseMesh3`
- `parseMesh4`
- `parseMesh5`
- `parseMesh7`

These objects retain format-specific data such as LODs, skinning, bones, FACS data, and v7 coremesh information.

### Versioned return shape

`parseMeshVersioned()` returns an object discriminated by `version` (`v1`, `v2`, `v3`, `v4`, `v5`, or `v7`). Its `mesh` field is the corresponding `Mesh1`/`Mesh2`/`Mesh3`/`Mesh4`/`Mesh5`/`Mesh7` object. Nested binary-format structures such as `Vertices2`, `Face2`, `Lod3`, `Bone4`, `Subset4`, FACS structures, and v7 `Coremesh` are also represented.

The dedicated `parseMeshN()` helpers reject a buffer whose detected format version does not match `N`.
