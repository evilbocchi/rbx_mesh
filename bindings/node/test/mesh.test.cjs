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

test("CSGMDL2 parser decodes raw union graphics", () => {
  const union = binding.parseCsgMdl2(readMesh("385416572.meshdata"));

  assert.equal(union.hash.hash.length, 16);
  assert.equal(union.hash.unknown.length, 16);
  assert.equal(union.mesh.vertexCount, union.mesh.vertices.length);
  assert.equal(union.mesh.faceCount, union.mesh.faces.length);
  assert.equal(union.mesh.vertices[0].pos.length, 3);
  assert.equal(union.mesh.vertices[0].norm.length, 3);
  assert.equal(union.mesh.vertices[0].color.length, 4);
  assert.equal(union.mesh.vertices[0].tex.length, 2);
  assert.equal(union.mesh.vertices[0].tangent.length, 3);
});

test("CSGMDL4 parser decodes raw union graphics", () => {
  const union = binding.parseCsgMdl4(readMesh("4500696697_4.meshdata"));

  assert.equal(union.hash.hash.length, 16);
  assert.equal(union.hash.unknown.length, 16);
  assert.equal(union.mesh.vertexCount, union.mesh.vertices.length);
  assert.equal(union.mesh.faceCount, union.mesh.faces.length);
  assert.ok(Array.isArray(union.unknown1));
});

test("CSGMDL5 parser decodes raw union graphics", () => {
  const union = binding.parseCsgMdl5(readMesh("14846974687_5.meshdata"));

  assert.equal(union.positionCount, union.positions.length);
  assert.equal(union.normalCount, union.normals.length);
  assert.equal(union.colorCount, union.colors.length);
  assert.equal(union.normalIdCount, union.normalIds.length);
  assert.equal(union.texCount, union.tex.length);
  assert.equal(union.tangentCount, union.tangents.length);
  assert.equal(union.positions[0].length, 3);
  assert.equal(union.normals[0].length, 3);
  assert.equal(union.colors[0].length, 4);
  assert.equal(union.tex[0].length, 2);
  assert.equal(union.tangents[0].length, 3);
  assert.equal(union.faces.indices.length % 3, 0);
  assert.ok(Array.isArray(union.faces.unknown));
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
