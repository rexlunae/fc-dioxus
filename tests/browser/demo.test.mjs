import assert from "node:assert/strict";
import { before, after, test } from "node:test";
import { spawn } from "node:child_process";
import { existsSync, readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { serve } from "../../tools/serve.mjs";

const chromePath = [process.env.CHROME_PATH, "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser"].find(path => path && existsSync(path));
assert.ok(chromePath, "Chrome is required; no browser means failure, never a skipped suite. Set CHROME_PATH.");
const axe = readFileSync(new URL("../../node_modules/axe-core/axe.min.js", import.meta.url), "utf8");
let server, chrome, profile, ws;
let nextId = 0;
const pending = new Map();
const exceptions = [];
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
function send(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = ++nextId;
    const timeout = setTimeout(() => { pending.delete(id); reject(new Error(`CDP timeout: ${method}`)); }, 20000);
    pending.set(id, result => { clearTimeout(timeout); result.error ? reject(new Error(JSON.stringify(result.error))) : resolve(result.result); });
    ws.send(JSON.stringify({ id, method, params }));
  });
}
async function evaluate(expression) {
  const result = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
  return result.result.value;
}
async function until(expression) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) { if (await evaluate(expression)) return; await sleep(80); }
  throw new Error(`Page did not become ready: ${expression}; exceptions=${JSON.stringify(exceptions)}`);
}
// Colors transition when a theme changes, and axe or getComputedStyle mid-transition
// reports a blend of the old and new palettes that no user ever rests on.
const settle = async () => {
  const quiet = "new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))).then(() => !document.getAnimations().some(a => a instanceof CSSTransition && a.playState !== 'finished'))";
  await until(quiet);
  await until(quiet);
};
async function load(query, width = 320) {
  exceptions.length = 0;
  await send("Emulation.setDeviceMetricsOverride", { width, height: 1000, deviceScaleFactor: 1, mobile: false });
  await send("Page.navigate", { url: "about:blank" });
  await until("location.href === 'about:blank'");
  await send("Page.navigate", { url: `${server.origin}/?${query}` });
  await until("document.querySelectorAll('.fc-divider').length === 6 && ['tokens','themes','button','card','badge','alert','divider'].every(name => [...document.styleSheets].some(s => s.href?.includes('/'+name+'-'))) ");
  await evaluate("document.fonts.ready.then(() => true)");
  await settle();
  await evaluate(axe);
}

before(async () => {
  server = await serve();
  profile = await mkdtemp(join(tmpdir(), "fc-dioxus-browser-"));
  chrome = spawn(chromePath, ["--headless=new", "--disable-gpu", "--no-sandbox", "--no-first-run", `--user-data-dir=${profile}`, "--remote-debugging-port=0", "about:blank"], { stdio: ["ignore", "ignore", "pipe"] });
  const endpoint = await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("Chrome did not start")), 20000);
    chrome.stderr.on("data", bytes => { const match = /ws:\/\/\S+/.exec(String(bytes)); if (match) { clearTimeout(timer); resolve(match[0]); } });
    chrome.on("error", error => { clearTimeout(timer); reject(error); });
  });
  const origin = endpoint.replace(/^ws:/, "http:").replace(/\/devtools\/browser\/.*$/, "");
  const targets = await (await fetch(`${origin}/json/list`)).json();
  const target = targets.find(target => target.type === "page");
  assert.ok(target, "Chrome exposes a page target");
  ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { ws.addEventListener("open", resolve, { once: true }); ws.addEventListener("error", reject, { once: true }); });
  ws.addEventListener("message", event => {
    const message = JSON.parse(event.data);
    if (message.id) { pending.get(message.id)?.(message); pending.delete(message.id); }
    if (message.method === "Runtime.exceptionThrown") exceptions.push(message.params);
  });
  await send("Runtime.enable");
  await send("Page.enable");
  await send("DOM.enable");
  await send("CSS.enable");
});
after(async () => {
  ws?.close();
  if (chrome && chrome.exitCode === null) { const exited = new Promise(resolve => chrome.once("exit", resolve)); chrome.kill(); await exited; }
  await server?.close();
  if (profile) await rm(profile, { recursive: true, force: true });
});

for (const theme of ["light", "dark"]) for (const dir of ["ltr", "rtl"]) {
  test(`fixtures: ${theme}/${dir}, 320px, visible dividers and axe WCAG AA`, async () => {
    await load(`fixtures=true&theme=${theme}&dir=${dir}`);
    const layout = await evaluate(`(() => {
      const root = document.querySelector('[data-fc-version]');
      return { theme: root.dataset.fcTheme, dir: root.dir,
        width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
        dividers: [...document.querySelectorAll('.fc-divider')].map(node => {
          const css = getComputedStyle(node), box = node.getBoundingClientRect();
          return { width: box.width, height: box.height, border: parseFloat(css.borderBlockStartWidth) + parseFloat(css.borderInlineStartWidth) };
        }) };
    })()`);
    assert.equal(layout.theme, theme);
    assert.equal(layout.dir, dir);
    assert.ok(layout.scroll <= layout.width + 1, JSON.stringify(layout));
    assert.equal(layout.dividers.length, 6);
    assert.ok(layout.dividers.every(d => d.width > 0 && d.height > 0 && d.border > 0), JSON.stringify(layout.dividers));
    const audit = await evaluate("axe.run(document, { runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'] } }).then(r => r.violations.map(v => ({id:v.id, impact:v.impact, nodes:v.nodes.map(n => [n.target, n.any.map(a => a.message)])})))");
    assert.deepEqual(audit, [], "axe must report zero WCAG A/AA violations");
    assert.deepEqual(exceptions, [], "no uncaught browser errors");
  });
}

test("theme controls, direction, source disclosure, focus visibility and copy feedback work", async () => {
  await load("fixtures=false&theme=light&dir=ltr", 1200);
  assert.equal(await evaluate("document.getElementById('design').value"), "helix", "the initial control reflects the applied preset");
  await evaluate("document.getElementById('theme').value='dark'; document.getElementById('theme').dispatchEvent(new Event('change',{bubbles:true}))");
  await until("document.querySelector('[data-fc-version]').dataset.fcTheme === 'dark'");
  await evaluate("document.getElementById('direction').value='rtl'; document.getElementById('direction').dispatchEvent(new Event('change',{bubbles:true}))");
  await until("document.querySelector('[data-fc-version]').dir === 'rtl'");
  await evaluate("document.querySelector('summary').focus()");
  await send("Input.dispatchKeyEvent", { type: "rawKeyDown", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13 });
  await send("Input.dispatchKeyEvent", { type: "char", text: "\r", unmodifiedText: "\r", key: "Enter", windowsVirtualKeyCode: 13 });
  await send("Input.dispatchKeyEvent", { type: "keyUp", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13 });
  await until("document.querySelector('details').open");
  await send("DOM.getDocument");
  const { root } = await send("DOM.getDocument");
  const { nodeId } = await send("DOM.querySelector", { nodeId: root.nodeId, selector: "#theme" });
  await send("CSS.forcePseudoState", { nodeId, forcedPseudoClasses: ["focus-visible"] });
  assert.ok(await evaluate("(() => {const c=getComputedStyle(document.getElementById('theme'));return parseFloat(c.outlineWidth)>0 || c.boxShadow!=='none'})()"), "visible keyboard focus ring");
  await evaluate("document.querySelector('[data-copy]').click()");
  await until("document.querySelector('[role=status]').textContent.length > 0");
  mkdirSync("target/screenshots", { recursive: true });
  await evaluate("document.querySelector('details').open=false");
  const screenshot = await send("Page.captureScreenshot", { format: "png" });
  writeFileSync("target/screenshots/showcase-dark.png", Buffer.from(screenshot.data, "base64"));
  assert.deepEqual(exceptions, []);
});

test("forced colors retain separator borders", async () => {
  await send("Emulation.setEmulatedMedia", { features: [{ name: "forced-colors", value: "active" }] });
  await load("fixtures=true&theme=light&dir=ltr");
  assert.ok(await evaluate("[...document.querySelectorAll('.fc-divider')].every(n => { const c=getComputedStyle(n); return parseFloat(c.borderInlineStartWidth)+parseFloat(c.borderBlockStartWidth)>0; })"));
  await send("Emulation.setEmulatedMedia", { features: [] });
});

test("every upstream preset changes rendered styling in light and dark and passes axe", async () => {
  await send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  await load("fixtures=false&theme=light&design=helix", 1200);
  const presets = await evaluate("[...document.getElementById('design').options].map(o => o.value)");
  const upstream = readFileSync("crates/fc-dioxus/assets/themes.css", "utf8");
  const expected = [...new Set([...upstream.matchAll(/data-fc-design=\"([^\"]+)\"/g)].map(match => match[1]))];
  assert.deepEqual([...presets].sort(), expected.sort());
  for (const scheme of ["light", "dark"]) {
    await evaluate(`document.getElementById('theme').value=${JSON.stringify(scheme)}; document.getElementById('theme').dispatchEvent(new Event('change',{bubbles:true}))`);
    const appearances = new Set();
    for (const preset of presets) {
      await evaluate(`document.getElementById('design').value=${JSON.stringify(preset)}; document.getElementById('design').dispatchEvent(new Event('change',{bubbles:true}))`);
      await until(`document.querySelector('[data-fc-version]').dataset.fcDesign === ${JSON.stringify(preset)}`);
      await settle();
      const appearance = await evaluate(`(() => {
        const root = document.querySelector('[data-fc-version]');
        const card = getComputedStyle(root.querySelector('.fc-card--feature'));
        const button = getComputedStyle(root.querySelector('#button .fc-btn--primary'));
        return { preset: document.getElementById('design').value, scheme: root.dataset.fcTheme,
          appearance: [card.backgroundColor, card.backgroundImage, card.borderRadius, card.borderWidth, card.boxShadow, card.fontFamily, button.backgroundColor, button.borderRadius],
          overflow: document.documentElement.scrollWidth > document.documentElement.clientWidth + 1 };
      })()`);
      assert.equal(appearance.preset, preset);
      assert.equal(appearance.scheme, scheme);
      assert.equal(appearance.overflow, false, `${preset}/${scheme} overflows`);
      appearances.add(JSON.stringify(appearance.appearance));
      const violations = await evaluate("axe.run(document,{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21a','wcag21aa','wcag22aa']}}).then(r=>r.violations.map(v=>({id:v.id,nodes:v.nodes.map(n=>[n.target,n.any.map(a=>a.message)])})))");
      assert.deepEqual(violations, [], `${preset}/${scheme} accessibility`);
    }
    assert.equal(appearances.size, presets.length, "each stock design must have a distinct computed appearance");
  }
  await send("Emulation.setEmulatedMedia", { features: [] });
});

test("each design applies its paired palette and every palette recolors the page", async () => {
  // Buttons transition their background; a mid-transition color would read as a repeat.
  await send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  await load("fixtures=false&theme=light&design=helix", 1200);
  const presets = readFileSync("upstream/theme-presets.ts", "utf8");
  const pairs = [...presets.matchAll(/id:'([^']+)',name:'[^']*',description:'[^']*',palette:'([^']+)'/g)].map(m => [m[1], m[2]]);
  assert.equal(pairs.length, 21, "upstream changed its design catalog");
  const accent = "getComputedStyle(document.querySelector('#button .fc-btn--primary')).backgroundColor";
  const pick = (id, value) => `document.getElementById('${id}').value=${JSON.stringify(value)}; document.getElementById('${id}').dispatchEvent(new Event('change',{bubbles:true}))`;
  for (const [design, palette] of pairs) {
    await evaluate(pick("design", design));
    await until(`document.querySelector('[data-fc-version]').dataset.fcDesign === ${JSON.stringify(design)}`);
    assert.equal(await evaluate("document.querySelector('[data-fc-version]').dataset.fcPalette"), palette, `${design} pairs with ${palette}`);
  }
  // A chosen palette overrides the design's, and a new design returns to its pairing.
  await evaluate(pick("design", "ocean"));
  await until("document.querySelector('[data-fc-version]').dataset.fcPalette === 'teal'");
  const colors = new Set();
  const palettes = await evaluate("[...document.getElementById('palette').options].map(o => o.value).filter(Boolean)");
  assert.equal(palettes.length, 10);
  for (const palette of palettes) {
    await evaluate(pick("palette", palette));
    await until(`document.querySelector('[data-fc-version]').dataset.fcPalette === ${JSON.stringify(palette)}`);
    await settle();
    colors.add(await evaluate(accent));
  }
  assert.equal(colors.size, palettes.length, "each palette must give primary actions a distinct color");
  await evaluate(pick("design", "forest"));
  await until("document.querySelector('[data-fc-version]').dataset.fcPalette === 'forest'");
  assert.equal(await evaluate("document.getElementById('palette').value"), "");
  await send("Emulation.setEmulatedMedia", { features: [] });
});

test("checkboxes, radios, switches, and list rows keep native behaviour", async () => {
  await load("fixtures=false&theme=light", 1200);
  const boxes = "[...document.querySelectorAll('#check .fc-check__input[type=checkbox]')]";
  assert.equal(await evaluate(`${boxes}[2].indeterminate`), true, "a mixed checkbox sets the DOM property");
  await evaluate(`${boxes}[2].click()`);
  await until(`!${boxes}[2].indeterminate && ${boxes}[2].checked`);
  await evaluate(`${boxes}[3].click()`);
  assert.equal(await evaluate(`${boxes}[3].checked`), false, "a disabled checkbox cannot change");
  const radios = "[...document.querySelectorAll('#check input[type=radio]')]";
  assert.equal(await evaluate(`new Set(${radios}.map(r => r.name)).size`), 1, "radios share one name");
  await evaluate(`${radios}[1].click()`);
  await until(`${radios}[1].checked && !${radios}[0].checked`);
  const row = "document.querySelector('#list label.fc-list__item input[role=switch]')";
  assert.equal(await evaluate(`${row}.checked`), true);
  await evaluate(`${row}.closest('label').querySelector('.fc-list__headline').click()`);
  await until(`!${row}.checked`);
  assert.equal(await evaluate("document.querySelector('#list a[aria-disabled=true]').hasAttribute('href')"), false, "an unavailable link has no destination");
});

test("native keyboard activation, unavailable guards, toggle tags, and dismiss callbacks work", async () => {
  await load("fixtures=false&theme=light", 1200);
  const enter = async () => {
    await send("Input.dispatchKeyEvent", { type: "rawKeyDown", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13 });
    await send("Input.dispatchKeyEvent", { type: "char", text: "\r", unmodifiedText: "\r", key: "Enter", windowsVirtualKeyCode: 13 });
    await send("Input.dispatchKeyEvent", { type: "keyUp", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13 });
  };
  await evaluate("document.querySelector('#button .fc-btn--primary').focus()");
  await enter();
  await until("document.querySelector('[data-activations]').textContent.includes('1 times')");
  for (const selector of ['#button [aria-disabled=true]', '#button [aria-busy=true]']) {
    await evaluate(`document.querySelector(${JSON.stringify(selector)}).focus()`);
    await enter();
    await evaluate(`document.querySelector(${JSON.stringify(selector)}).click()`);
    assert.match(await evaluate("document.querySelector('[data-activations]').textContent"), /1 times/);
  }
  await evaluate("[...document.querySelectorAll('#badge button')].find(n=>n.textContent==='Rust').focus()");
  await send("Input.dispatchKeyEvent", { type: "rawKeyDown", key: " ", code: "Space", windowsVirtualKeyCode: 32 });
  await send("Input.dispatchKeyEvent", { type: "keyUp", key: " ", code: "Space", windowsVirtualKeyCode: 32 });
  await until("[...document.querySelectorAll('#badge button')].find(n=>n.textContent==='Rust').getAttribute('aria-pressed')==='true'");
  await evaluate("document.querySelector('#alert [aria-label=\"Dismiss warning\"]').focus()");
  await enter();
  await until("document.querySelector('#alert [role=status]').textContent.includes('Warning acknowledged')");
  assert.deepEqual(exceptions, []);
});
