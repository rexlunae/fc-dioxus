import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { root, files, verifySnapshot } from "./sync-upstream.mjs";

verifySnapshot();
const sources = ["crates", "demo", "xtask"].flatMap(p => files(join(root, p))).filter(p => p.endsWith(".rs"));
for (const path of sources) {
  const source = readFileSync(path, "utf8");
  const name = relative(root, path);
  if (name.endsWith("/lib.rs") || name.endsWith("/main.rs")) assert.ok(source.includes("#![forbid(unsafe_code)]"), `${name}: unsafe must be forbidden`);
  if (!name.includes("/tests/")) {
    assert.doesNotMatch(source, /\b(?:panic|unreachable|todo|dbg)!\s*\(|\.(?:unwrap|expect)\s*\(/, `${name}: panicking consumer path`);
  }
  if (name.includes("/components/")) {
    assert.doesNotMatch(source, /\b(?:web_sys|document::eval)\b|"data-fc-/, `${name}: bypassed typed platform/attribute helper`);
    const withoutBem = source.replace(/Bem::new\("fc-[^"]+"\)/g, "");
    assert.doesNotMatch(withoutBem, /"fc-[^"]*"/, `${name}: class prefix belongs in the BEM declaration`);
    assert.doesNotMatch(source, /\bstyle\s*:/, `${name}: no local inline CSS`);
  }
}
for (const path of files(root).filter(p => !p.includes("/target/") && !p.includes("/node_modules/") && !p.includes("/.git/"))) {
  if (path.endsWith(".css")) assert.ok(path.startsWith(join(root, "crates/fc-dioxus/assets/")), `authored CSS is forbidden: ${path}`);
}
for (const directory of ["crates/fc-dioxus", "crates/fc-a11y", "crates/fc-tokens", "demo", "xtask"]) {
  assert.match(readFileSync(join(root, directory, "Cargo.toml"), "utf8"), /\[lints\]\s*workspace = true/, `${directory}: inherit workspace lints`);
}
console.log("Architecture: no local CSS, no bypassed component contracts, all crates inherit safety lints.");
