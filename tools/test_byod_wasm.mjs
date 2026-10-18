import { readFile, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
import { resolve } from "node:path";

const root = resolve(new URL("..", import.meta.url).pathname);
const pkg = resolve(root, "demo/wasm/pkg");
const module = await import(pathToFileURL(resolve(pkg, "athlesia_causal_wasm.js")));
const bytes = await readFile(resolve(pkg, "athlesia_causal_wasm_bg.wasm"));
await module.default({ module_or_path: bytes });

for (const name of ["run_byod_break_test_json", "verify_byod_break_test_json"]) {
  if (typeof module[name] !== "function") throw new Error(`generated WASM lacks ${name}`);
}

const source = `episode,action,before.power,before.coolant,before.open,after.power,after.coolant,after.open
1,press,1,0,0,1,0,1
2,press,0,1,0,0,1,1
3,press,1,1,0,1,1,1
`;
const target = `episode,action,after.commit,before.replica,after.quorum,before.commit,after.replica,before.quorum
1,write,1,0,1,0,0,1
2,write,1,1,0,0,1,0
3,write,1,0,0,0,0,0
`;
const harmless = target.replace("2,write,1,1,0,0,1,0", "2,write,1,0,0,0,0,0");
const contradiction = target.replace("3,write,1,0,0,0,0,0", "3,write,0,0,0,0,0,0");
const starvation = `${target.trimEnd().split("\n").slice(0, 2).join("\n")}\n`;

const run = input => {
  const raw = module.run_byod_break_test_json(source, input);
  return { raw, bundle: JSON.parse(raw) };
};
const baseline = run(target);
const changed = run(harmless);
const broken = run(contradiction);
const starved = run(starvation);

if (baseline.bundle.report.status !== "VERIFIED") throw new Error("baseline was not verified");
if (changed.bundle.report.status !== "VERIFIED" || changed.bundle.proof_id === baseline.bundle.proof_id) throw new Error("harmless edit did not produce an independent verified proof");
if (broken.bundle.report.status !== "CONTRADICTED" || broken.bundle.report.conflicting_episode !== 3) throw new Error("changed outcome did not name its contradiction");
if (starved.bundle.report.status !== "ABSTAIN") throw new Error("missing holdouts did not abstain");
if (baseline.bundle.report.target_training_updates !== 0) throw new Error("target training occurred");

const replay = JSON.parse(module.verify_byod_break_test_json(baseline.raw));
if (!replay.verified) throw new Error("exact WASM replay failed");
const tampered = structuredClone(baseline.bundle);
tampered.report.reason += " tampered";
const rejected = JSON.parse(module.verify_byod_break_test_json(JSON.stringify(tampered)));
if (rejected.verified) throw new Error("tampered proof was accepted");

if (process.argv[2]) await writeFile(resolve(process.argv[2]), baseline.raw);
console.log(JSON.stringify({
  baseline: baseline.bundle.report.status,
  harmless_edit: changed.bundle.report.status,
  contradiction: broken.bundle.report.status,
  starvation: starved.bundle.report.status,
  target_training_updates: baseline.bundle.report.target_training_updates,
  proof_id: baseline.bundle.proof_id,
  wasm_replay: replay.verified,
  tamper_rejected: !rejected.verified,
}, null, 2));
