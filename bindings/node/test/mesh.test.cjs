const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const binding = require("../index.js");

const meshes = path.resolve(__dirname, "../../../meshes");
const readMesh = (name) => fs.readFileSync(path.join(meshes, name));

test("high-level parseMesh returns typed arrays", () => {
  const mesh = binding.parseMesh(readMesh("torso.mesh"));

  assert.equal(mesh.version, "2.00");
  assert.ok(mesh.positions instanceof Float32Array);
  assert.ok(mesh.normals instanceof Float32Array);
  assert.ok(mesh.uvs instanceof Float32Array);
  assert.ok(mesh.tangents instanceof Int8Array);
  assert.ok(mesh.indices instanceof Uint32Array);
  assert.equal(mesh.positions.length % 3, 0);
  assert.equal(mesh.normals.length, mesh.positions.length);
  assert.equal(mesh.uvs.length, (mesh.positions.length / 3) * 2);
  assert.equal(mesh.indices.length % 3, 0);
});

test("low-level v1 parser mirrors v1 vertex fields", () => {
  const mesh = binding.parseMesh1(readMesh("158071912"));

  assert.equal(mesh.revision, "Version100");
  assert.equal(mesh.vertices[0].pos.length, 3);
  assert.equal(mesh.vertices[0].norm.length, 3);
  assert.equal(mesh.vertices[0].tex.length, 3);
});

test("low-level v2 parser retains versioned structure", () => {
  const mesh = binding.parseMesh2(readMesh("torso.mesh"));

  assert.equal(mesh.revision, "Version200");
  assert.equal(mesh.faceCount, mesh.faces.length);
  assert.equal(mesh.vertexCount, mesh.vertices.vertices.length);
  assert.ok(
    mesh.vertices.kind === "full" || mesh.vertices.kind === "truncated",
  );
  assert.equal(mesh.sizeOfVertex, mesh.vertices.kind === "full" ? 40 : 36);
  assert.equal(mesh.vertices.vertices[0].pos.length, 3);
  assert.equal(mesh.vertices.vertices[0].tex.length, 2);
});

test("dedicated v3 parser handles both known v3 revisions", () => {
  assert.equal(
    binding.parseMesh3(readMesh("5115672913")).revision,
    "Version300",
  );
  assert.equal(
    binding.parseMesh3(readMesh("5648093777")).revision,
    "Version301",
  );
});

test("parseMeshVersioned is discriminated", () => {
  const parsed = binding.parseMeshVersioned(readMesh("sphere.mesh"));

  assert.equal(parsed.version, "v4");
  assert.equal(parsed.mesh.revision, "Version401");
  assert.equal(parsed.mesh.vertexCount, parsed.mesh.vertices.length);
});

test("v5 exposes FACS and skinning-era structures", () => {
  const mesh = binding.parseMesh5(readMesh("13674780763"));

  assert.equal(mesh.revision, "Version500");
  assert.equal(mesh.vertexCount, mesh.vertices.length);
  assert.equal(mesh.faceCount, mesh.faces.length);
  assert.equal(mesh.facs.faceBoneNamesLen, mesh.facs.faceBoneNames.length);
  assert.equal(
    mesh.facs.faceControlNamesLen,
    mesh.facs.faceControlNames.length,
  );
});

test("v7 exposes raw coremesh plus decoded geometry", () => {
  const mesh = binding.parseMesh7(readMesh("mesh7_127279296594138.bin"));

  assert.equal(mesh.revision, "Version700");
  assert.ok(mesh.vertices.length > 0);
  assert.ok(mesh.faces.length > 0);
  assert.ok(mesh.coremesh.kind === "v1" || mesh.coremesh.kind === "v2");
});

test("dedicated parser rejects a mismatched version", () => {
  assert.throws(
    () => binding.parseMesh5(readMesh("torso.mesh")),
    /expected Roblox mesh version 5/,
  );
});
