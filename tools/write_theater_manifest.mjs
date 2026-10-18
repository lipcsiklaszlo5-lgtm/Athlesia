import { readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { resolve } from "node:path";

const [root, output] = process.argv.slice(2);
if (!root || !output) throw new Error("Expected repository root and generated WASM output directory.");
const cwd = resolve(root);
const directory = resolve(output);
const binary = await readFile(resolve(directory, "athlesia_causal_wasm_bg.wasm"));
const git = args => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
const manifest = {
  schema: "athlesia/theater-build/1",
  source_commit: git(["rev-parse", "HEAD"]),
  // A dirty flag is essential: HEAD alone does not identify uncommitted source.
  source_dirty: git(["status", "--porcelain", "--untracked-files=normal", "--", "crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo"]) !== "",
  source_paths_checked: ["crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo"],
  wasm_sha256: createHash("sha256").update(binary).digest("hex"),
  wasm_bytes: binary.byteLength,
  rustc: execFileSync("rustc", ["--version"], { encoding: "utf8" }).trim().replace(/^rustc /, ""),
  profile: "release",
  boundary: "TheaterReceiver / authorize_and_dispatch_trusted / CausalTrustMesh",
  p9_bindings_available: ["WasmTheaterOrigin", "WasmTheaterReceiver"],
  signature_algorithm: "Ed25519 / ed25519-dalek",
  note: "Build metadata and executable fingerprint; runtime proof is exported separately. Not remote attestation.",
};
await writeFile(resolve(directory, "theater-build.json"), JSON.stringify(manifest, null, 2) + "\n");
console.log(`WASM SHA-256: ${manifest.wasm_sha256}`);
