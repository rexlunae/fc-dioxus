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
  // The fields example's error summary takes focus when it mounts. Wait for that, or
  // it lands after a test has put focus somewhere and silently moves it.
  await until("!document.querySelector('.fc-error-summary') || document.activeElement?.matches('.fc-error-summary')");
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
      // A preference inside a content card must not select the whole card.
      const cardPalette=()=>evaluate(`(() => {const card=document.getElementById('feature-composition'),s=getComputedStyle(card);return [s.backgroundColor,s.backgroundImage,s.color];})()`);
      const beforePreference=await cardPalette();
      await evaluate("document.querySelector('#feature-composition .fc-check__input').click()");
      await settle();
      assert.deepEqual(await cardPalette(),beforePreference,`${preset}/${scheme} preference must not select its card`);
      await evaluate("document.querySelector('#feature-composition .fc-check__input').click()");
      await settle();
      const contrast = await evaluate(`(() => {
        const card=document.getElementById('feature-composition'), ink=getComputedStyle(card).color;
        const unpainted=[...card.querySelectorAll('.fc-card__subtitle,.fc-card__body > p,.fc-card__body > .fc-dl dt,.fc-field__label,.fc-field__hint,.fc-check,.fc-check__hint,#feature-clear')];
        const c=document.createElement('canvas').getContext('2d');
        const lum=color=>{c.fillStyle=color;c.fillRect(0,0,1,1);const v=[...c.getImageData(0,0,1,1).data].slice(0,3).map(n=>n/255).map(n=>n<=.04045?n/12.92:((n+.055)/1.055)**2.4);return v[0]*.2126+v[1]*.7152+v[2]*.0722;};
        const pair=(node,surface)=>{const a=lum(getComputedStyle(node).color),b=lum(getComputedStyle(surface).backgroundColor);return (Math.max(a,b)+.05)/(Math.min(a,b)+.05);};
        const boxed=card.querySelector('.fc-disclosure--boxed'),group=card.querySelector('.fc-input-group');
        return {wrong:unpainted.filter(n=>getComputedStyle(n).color!==ink).map(n=>n.className),boxed:pair(boxed.querySelector('dt'),boxed),group:pair(group.querySelector('button'),group)};
      })()`);
      assert.deepEqual(contrast.wrong,[],`${preset}/${scheme} feature context`);
      assert.ok(contrast.boxed>=4.49 && contrast.group>=4.49,JSON.stringify({preset,scheme,...contrast}));
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

test("fields wire labels, descriptions, and errors, and the summary takes focus", async () => {
  await load("fixtures=false&theme=light", 1200);
  assert.ok(await evaluate("[...document.querySelectorAll('#field label[for]')].every(l => document.getElementById(l.htmlFor)?.matches('input, textarea, select'))"), "every field label targets its control");
  assert.ok(await evaluate("[...document.querySelectorAll('#field [aria-describedby]')].every(n => n.getAttribute('aria-describedby').split(' ').every(id => document.getElementById(id)))"), "descriptions resolve");
  assert.equal(await evaluate("document.activeElement?.classList.contains('fc-error-summary')"), true, "the summary takes focus when it appears");
  await evaluate("document.querySelector('#field .fc-error-summary__link').click()");
  const email = "document.getElementById(document.querySelector('#field .fc-error-summary__link').hash.slice(1))";
  assert.equal(await evaluate(`${email}.getAttribute('aria-invalid')`), "true");
  await evaluate(`(() => { const input = ${email}; input.focus(); input.value = 'ada@example.com'; input.dispatchEvent(new Event('input', { bubbles: true })); })()`);
  await until("!document.querySelector('#field .fc-error-summary')");
  assert.equal(await evaluate("[...document.querySelectorAll('#field input[type=email]')].pop().hasAttribute('aria-invalid')"), false);
});

test("pagination pages in place and search submits without navigating", async () => {
  await load("fixtures=false&theme=light", 1200);
  const first = "document.querySelector('#pagination nav')";
  await evaluate(`[...${first}.querySelectorAll('a')].find(a => a.textContent === '2').click()`);
  await until("document.querySelector('#pagination [role=status]').textContent.includes('page 2 of 12')");
  assert.equal(await evaluate(`${first}.querySelector('[aria-current=page]').textContent`), "2");
  assert.equal(await evaluate(`${first}.querySelector('.fc-pagination__link--prev').hasAttribute('aria-disabled')`), false, "previous is enabled after page 1");
  const before = await evaluate("location.pathname + location.search");
  await evaluate("(() => { const input = document.querySelector('#search input[type=search]'); input.value = 'tokens'; input.dispatchEvent(new Event('input', { bubbles: true })); input.form.requestSubmit(); })()");
  await until("document.querySelector('#search [role=status]').textContent.includes('tokens')");
  assert.equal(await evaluate("location.pathname + location.search"), before, "submitting does not navigate");
});

test("dialogs open modally and close on Escape, tooltips dismiss, toasts route and expire", async () => {
  await load("fixtures=false&theme=light", 1200);
  const open = "document.querySelector('#dialog dialog[open]')";
  await evaluate("[...document.querySelectorAll('#dialog button')].find(b => b.textContent.startsWith('Delete chat')).click()");
  await until(`${open}?.matches(':modal')`);
  assert.ok(await evaluate(`${open}.contains(document.activeElement)`), "focus moves into the modal");
  const violations = await evaluate("axe.run(document,{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa','wcag22aa']}}).then(r=>r.violations.map(v=>v.id))");
  assert.deepEqual(violations, [], "an open modal passes axe");
  await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 });
  await send("Input.dispatchKeyEvent", { type: "keyUp", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 });
  await until(`!${open}`);

  const tip = "document.querySelector('#popover .fc-tooltip')";
  await evaluate(`${tip}.querySelector('button').focus()`);
  assert.ok(await evaluate(`${tip}.querySelector('button').getAttribute('aria-describedby').split(' ').some(id => document.getElementById(id)?.role === 'tooltip')`), "the button is described by its bubble");
  await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 });
  await until(`${tip}.dataset.fcDismissed === 'true'`);
  assert.ok(await evaluate(`${tip}.contains(document.activeElement)`), "Escape leaves focus where it was");

  const button = text => `[...document.querySelectorAll('#toast button')].find(b => b.textContent === ${JSON.stringify(text)})`;
  await evaluate(`${button("Show an error toast")}.click()`);
  await until("document.querySelector('.fc-toaster [aria-live=assertive] .fc-toast--danger')");
  await evaluate(`${button("Show a success toast")}.click()`);
  await until("document.querySelector('.fc-toaster [aria-live=polite] .fc-toast--success')");
  await evaluate("document.querySelector('.fc-toaster [aria-live=assertive] .fc-toast__close').click()");
  await until("!document.querySelector('.fc-toaster [aria-live=assertive] .fc-toast')");
});

async function press(key, code = key, keyCode = 0) {
  // Enter activates a focused button only when the key event carries its text.
  const text = key === "Enter" ? "\r" : undefined;
  await send("Input.dispatchKeyEvent", { type: "keyDown", key, code, windowsVirtualKeyCode: keyCode, text });
  await send("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: keyCode });
}

for (const dir of ["ltr", "rtl"]) test(`tabs and toolbars rove with arrows in ${dir}`, async () => {
  await load(`fixtures=false&theme=light&dir=${dir}`, 1200);
  const forward = dir === "rtl" ? ["ArrowLeft", 37] : ["ArrowRight", 39];
  const tabs = "[...document.querySelectorAll('#tabs .fc-tabs')[0].querySelectorAll('[role=tab]')]";
  await evaluate(`${tabs}[0].focus()`);
  await press(forward[0], forward[0], forward[1]);
  await until(`document.activeElement === ${tabs}[1] && ${tabs}[1].getAttribute('aria-selected') === 'true'`);
  assert.ok(await evaluate("document.querySelector('#tabs .fc-tabs [role=tabpanel]').textContent.startsWith('Two problems')"), "automatic activation shows the panel");
  await press(forward[0], forward[0], forward[1]);
  await until(`document.activeElement === ${tabs}[0]`);
  assert.equal(await evaluate(`${tabs}.filter(t => t.tabIndex === 0).length`), 1, "one tab stop");
  await press("End", "End", 35);
  await until(`document.activeElement === ${tabs}[1]`);

  const manual = "[...document.querySelectorAll('#tabs .fc-tabs')[3].querySelectorAll('[role=tab]')]";
  await evaluate(`${manual}[0].focus()`);
  await press(forward[0], forward[0], forward[1]);
  await until(`document.activeElement === ${manual}[1]`);
  assert.equal(await evaluate(`${manual}[1].getAttribute('aria-selected')`), "false", "manual activation only moves focus");
  await press("Enter", "Enter", 13);
  await until(`${manual}[1].getAttribute('aria-selected') === 'true'`);

  const items = "[...document.querySelector('#toolbar [role=toolbar]').querySelectorAll('button')]";
  assert.equal(await evaluate(`${items}.filter(b => b.tabIndex === 0).length`), 1, "one toolbar stop");
  await evaluate(`${items}[0].focus()`);
  await press("End", "End", 35);
  await until(`document.activeElement === ${items}[3]`);
  assert.equal(await evaluate("document.activeElement.getAttribute('aria-disabled')"), "true", "aria-disabled items stay reachable; natively disabled ones are skipped");
  await press(forward[0], forward[0], forward[1]);
  await until(`document.activeElement === ${items}[0]`);
});

test("sliders fill to their value and carousels page between their ends", async () => {
  await load("fixtures=false&theme=light", 320);
  const slider = "document.querySelector('#slider input[type=range]')";
  assert.equal(await evaluate(`${slider}.style.getPropertyValue('--fc-slider-pct')`), "35%");
  await evaluate(`${slider}.focus()`);
  await press("ArrowRight", "ArrowRight", 39);
  await until(`${slider}.style.getPropertyValue('--fc-slider-pct') === '40%' && document.querySelector('#slider output').textContent === '0.8'`);

  const carousel = "document.querySelector('#carousel .fc-carousel')";
  const prev = `${carousel}.querySelector('[data-fc-carousel=prev]')`;
  const next = `${carousel}.querySelector('[data-fc-carousel=next]')`;
  await evaluate(`${carousel}.scrollIntoView()`);
  await until(`${next}.getAttribute('aria-disabled') === 'false'`);
  assert.equal(await evaluate(`${prev}.getAttribute('aria-disabled')`), "true", "previous is disabled at the start");
  await evaluate(`${next}.click()`);
  await until(`Math.abs(${carousel}.querySelector('.fc-carousel__track').scrollLeft) > 0`);
  await until(`!${prev}.hasAttribute('aria-disabled')`);
});

test("an RTL carousel's Next advances toward the inline end", async () => {
  await load("fixtures=false&theme=light&dir=rtl", 320);
  const carousel = "document.querySelector('#carousel .fc-carousel')";
  const track = `${carousel}.querySelector('.fc-carousel__track')`;
  const next = `${carousel}.querySelector('[data-fc-carousel=next]')`;
  await evaluate(`${carousel}.scrollIntoView()`);
  await until(`${next}.getAttribute('aria-disabled') === 'false'`);
  await evaluate(`${next}.click()`);
  // scrollLeft runs negative in RTL; moving forward makes it more negative.
  await until(`${track}.scrollLeft < 0`);
  await until(`!${carousel}.querySelector('[data-fc-carousel=prev]').hasAttribute('aria-disabled')`);
});

test("tables sort by one column at a time and keep row state", async () => {
  await load("fixtures=false&theme=light", 1200);
  const table = "document.querySelector('#table table')";
  const firstColumn = `[...${table}.tBodies[0].rows].map(r => r.cells[0].textContent)`;
  assert.deepEqual(await evaluate(firstColumn), ["Index docs", "Refactor parser", "Triage issues"]);
  await evaluate(`[...${table}.querySelectorAll('.fc-table__sort')].find(b => b.textContent === 'Tokens').click()`);
  await until(`${table}.querySelectorAll('[aria-sort]').length === 1 && ${table}.querySelector('[aria-sort]').textContent === 'Tokens'`);
  assert.deepEqual(await evaluate(firstColumn), ["Triage issues", "Index docs", "Refactor parser"], "numbers sort as numbers");
  await evaluate(`[...${table}.querySelectorAll('.fc-table__sort')].find(b => b.textContent === 'Tokens').click()`);
  await until(`${table}.querySelector('[aria-sort]').getAttribute('aria-sort') === 'descending'`);
  assert.equal(await evaluate(`${table}.querySelector('tr[aria-selected=true]').cells[0].textContent`), "Refactor parser", "selection follows its row");
});

test("the sidebar collapses to a titled rail and the multiselect filters, bulk-selects, and closes on Escape", async () => {
  await load("fixtures=false&theme=light", 1200);
  const nav = "document.querySelector('#nav .fc-nav')";
  const toggle = `${nav}.querySelector('.fc-nav__toggle')`;
  assert.equal(await evaluate(`${toggle}.getAttribute('aria-expanded')`), "true");
  await evaluate(`${toggle}.click()`);
  await until(`${toggle}.getAttribute('aria-expanded') === 'false' && ${nav}.querySelector('.fc-nav__link').title === 'Current page'`);
  await evaluate(`${toggle}.click()`);
  await until(`${toggle}.getAttribute('aria-expanded') === 'true' && !${nav}.querySelector('.fc-nav__link').hasAttribute('title')`);

  const ms = "document.querySelector('#multiselect details')";
  await evaluate(`(() => { const q = ${ms}.querySelector('[data-fc-filter]'); q.value = 'grace'; q.dispatchEvent(new Event('input', { bubbles: true })); })()`);
  await until(`[...${ms}.querySelectorAll('.fc-multiselect__option')].filter(o => !o.hidden).length === 1`);
  await evaluate(`${ms}.querySelector('[data-fc-selection-action=all]').click()`);
  await until(`${ms}.querySelector('[data-fc-selection-summary]').textContent === '2 selected'`);
  await evaluate(`${ms}.querySelector('[data-fc-selection-action=clear]').click()`);
  await until(`${ms}.querySelector('[data-fc-selection-summary]').textContent === 'None selected'`);
  assert.equal(await evaluate(`${ms}.querySelector('input[value=alan]').checked`), false, "unavailable options keep their state");
  await evaluate(`${ms}.querySelector('[data-fc-filter]').focus()`);
  await press("Escape", "Escape", 27);
  await until(`!${ms}.open && document.activeElement === ${ms}.querySelector('summary')`);
});

test("splitters move by key and pointer, collapse with Enter, and settle once", async () => {
  await load("fixtures=false&theme=light", 1200);
  const bar = "document.querySelector('#splitter [role=separator]')";
  const pane = `document.getElementById(${bar}.getAttribute('aria-controls'))`;
  await evaluate(`${bar}.scrollIntoView({ block: 'center' }); ${bar}.focus()`);
  await press("ArrowRight", "ArrowRight", 39);
  await until(`${bar}.getAttribute('aria-valuenow') === '40' && ${pane}.style.getPropertyValue('--fc-split-size') === '40%'`);
  await press("Enter", "Enter", 13);
  await until(`${bar}.getAttribute('aria-valuenow') === '15'`);
  await press("Enter", "Enter", 13);
  await until(`${bar}.getAttribute('aria-valuenow') === '40'`);
  await press("End", "End", 35);
  await until(`${bar}.getAttribute('aria-valuenow') === '70'`);
  await press("Home", "Home", 36);
  await until(`${bar}.getAttribute('aria-valuenow') === '15'`);

  const box = await evaluate(`(() => { const r = ${bar}.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
  await send("Input.dispatchMouseEvent", { type: "mousePressed", x: box.x, y: box.y, button: "left", clickCount: 1 });
  await sleep(200);
  for (const step of [20, 60, 100]) {
    await send("Input.dispatchMouseEvent", { type: "mouseMoved", x: box.x + step, y: box.y, button: "left", buttons: 1 });
    await sleep(60);
  }
  await send("Input.dispatchMouseEvent", { type: "mouseReleased", x: box.x + 100, y: box.y, button: "left", clickCount: 1 });
  await until(`Number(${bar}.getAttribute('aria-valuenow')) > 15`);
  await until(`document.querySelector('#splitter [role=status]').textContent.includes(${bar}.getAttribute('aria-valuenow') + '%')`);

  const collapsed = "[...document.querySelectorAll('#splitter [role=separator]')][2]";
  assert.equal(await evaluate(`document.getElementById(${collapsed}.getAttribute('aria-controls')).hidden`), true, "a pane at zero is hidden");
  const fixed = "[...document.querySelectorAll('#splitter [role=separator]')].pop()";
  await evaluate(`${fixed}.focus()`);
  await press("ArrowRight", "ArrowRight", 39);
  assert.equal(await evaluate(`${fixed}.getAttribute('aria-valuenow')`), "50", "a disabled bar does not move");
});

test("the date grid moves by day, week, and month, skips nothing, and refuses unavailable days", async () => {
  await load("fixtures=false&theme=light", 1200);
  const picker = "document.querySelector('#datepicker .fc-datepicker')";
  const focused = "document.activeElement.dataset.fcDate";
  assert.equal(await evaluate(`${picker}.dataset.fcValue`), "2026-10-21");
  assert.equal(await evaluate(`${picker}.querySelector('[aria-current=date]').dataset.fcDate`), "2026-10-10");
  await evaluate(`${picker}.querySelector('.fc-datepicker__day[tabindex="0"]').focus()`);
  await press("ArrowRight", "ArrowRight", 39);
  await until(`${focused} === '2026-10-22'`);
  await press("ArrowDown", "ArrowDown", 40);
  await until(`${focused} === '2026-10-29'`);
  await press("PageDown", "PageDown", 34);
  await until(`${focused} === '2026-11-29' && ${picker}.querySelector('h2').textContent === 'November 2026'`);
  await press("Enter", "Enter", 13);
  await until(`${picker}.dataset.fcValue === '2026-11-29'`);
  const delivery = "document.querySelectorAll('#datepicker .fc-datepicker')[1]";
  assert.equal(await evaluate(`${delivery}.querySelector('[data-fc-datepicker=prev]').getAttribute('aria-disabled')`), "true", "no earlier month to page to");
  await evaluate(`${delivery}.querySelector('[data-fc-date="2026-10-17"]').click()`);
  assert.equal(await evaluate(`${delivery}.dataset.fcValue`), undefined, "a weekend cannot be chosen");
});

test("links and link buttons navigate in-app inside a router and track the current route", async () => {
  await load("fixtures=false&theme=light", 1200);
  const section = "document.querySelector('#routing')";
  const heading = `${section}.querySelector('h3').textContent`;
  const before = await evaluate("location.href");
  assert.equal(await evaluate(heading), "Inbox");
  assert.equal(await evaluate(`${section}.querySelector('.fc-nav__link[aria-current=page]').textContent`), "Inbox");
  await evaluate(`[...${section}.querySelectorAll('.fc-nav__link')].find(a => a.textContent === 'Projects').click()`);
  await until(`${heading} === 'Projects' && ${section}.querySelector('.fc-nav__link[aria-current=page]').textContent === 'Projects'`);
  await evaluate(`${section}.querySelector('.fc-list__action').click()`);
  await until(`${heading} === 'Project 1' && ${section}.querySelector('.fc-breadcrumb [aria-current=page]')`);
  await evaluate(`[...${section}.querySelectorAll('.fc-pagination__link')].find(a => a.textContent === '3').click()`);
  await until(`${heading} === 'Project 3'`);
  await evaluate(`${section}.querySelector('a.fc-btn').click()`);
  await until(`${heading} === 'Inbox'`);
  assert.equal(await evaluate("location.href"), before, "no navigation left the page");
});

test("select keeps focus on its button, commits with Enter, and restores with Escape", async () => {
  await load("fixtures=false&theme=light", 1200);
  const invoker = "document.querySelector('#select [role=combobox]')";
  const list = `document.getElementById(${invoker}.id + '-listbox')`;
  const active = `document.getElementById(${invoker}.getAttribute('aria-activedescendant'))?.dataset.fcValue`;
  await evaluate(`${invoker}.focus()`);
  await press("ArrowDown", "ArrowDown", 40);
  await until(`${invoker}.getAttribute('aria-expanded') === 'true' && ${list}.matches(':popover-open')`);
  assert.equal(await evaluate(active), "opus", "opens on the committed value");
  await press("ArrowDown", "ArrowDown", 40);
  await until(`${active} === 'sonnet'`);
  assert.equal(await evaluate(`document.activeElement === ${invoker}`), true, "focus stays on the combobox");
  await press("Enter", "Enter", 13);
  await until(`${invoker}.getAttribute('aria-expanded') === 'false' && ${invoker}.dataset.fcValue === 'sonnet'`);
  await until("document.querySelector('#select [role=status]').textContent === 'Model: sonnet'");
  await press("ArrowDown", "ArrowDown", 40);
  await until(`${active} === 'sonnet'`);
  await press("End", "End", 35);
  await until(`${active} === 'haiku'`);
  await press("Escape", "Escape", 27);
  await until(`${invoker}.getAttribute('aria-expanded') === 'false'`);
  assert.equal(await evaluate(`${invoker}.dataset.fcValue`), "sonnet", "Escape restores the previous value");
  await press("ArrowDown", "ArrowDown", 40);
  await press("o", "KeyO", 79);
  await until(`${active} === 'opus'`);
  await press("Enter", "Enter", 13);
  await until(`${invoker}.dataset.fcValue === 'opus'`);
});

test("menus move focus in, skip what cannot be used, search by typing, and return focus", async () => {
  await load("fixtures=false&theme=light", 1200);
  const trigger = "document.querySelector('#menu [aria-haspopup=menu]')";
  const menu = `document.getElementById(${trigger}.getAttribute('aria-controls'))`;
  const focusedLabel = "document.activeElement.querySelector('.fc-menu__label')?.textContent";
  await evaluate(`${trigger}.focus()`);
  await press("ArrowDown", "ArrowDown", 40);
  await until(`${menu}.matches(':popover-open') && ${focusedLabel} === 'Cut'`);
  assert.equal(await evaluate(`${trigger}.getAttribute('aria-expanded')`), "true");
  await press("ArrowDown", "ArrowDown", 40);
  await until(`${focusedLabel} === 'Copy'`);
  await press("ArrowDown", "ArrowDown", 40);
  await until(`${focusedLabel} === 'Wrap lines'`);
  await press("d", "KeyD", 68);
  await until(`${focusedLabel} === 'Delete file'`);
  await press("Escape", "Escape", 27);
  await until(`!${menu}.matches(':popover-open') && document.activeElement === ${trigger}`);
  await press("ArrowUp", "ArrowUp", 38);
  await until(`${focusedLabel} === 'Delete file'`);
  await press("Home", "Home", 36);
  await until(`${focusedLabel} === 'Cut'`);
  await press("ArrowDown", "ArrowDown", 40);
  await press("Enter", "Enter", 13);
  await until("document.querySelector('#menu [role=status]').textContent === 'Chose: copy'");
  await until(`document.activeElement === ${trigger}`);

  const area = "document.querySelector('#menu .fc-card')";
  await evaluate(`(() => { const r = ${area}.getBoundingClientRect(); ${area}.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: r.x + 20, clientY: r.y + 10 })); })()`);
  await until("[...document.querySelectorAll('#menu [role=menu]')].some(m => m.matches(':popover-open'))");
  await press("Escape", "Escape", 27);
  await until(`document.activeElement === ${area}`);
});

// Real browser focus is required here: a pure select-model test cannot observe
// an asynchronous DOM focus restore stealing the next control's focus.
test("select blur preserves its destination and switching selectors still dismisses", async () => {
  await load("fixtures=false&theme=light", 1200);
  const first = "document.querySelectorAll('#select [role=combobox]')[0]";
  const second = "document.querySelectorAll('#select [role=combobox]')[1]";
  const destination = "document.querySelector('#composer textarea')";
  const popup = control => `document.getElementById(${control}.id + '-listbox')`;
  const click = async control => {
    const point = await evaluate(`(() => { const node = ${control}; node.scrollIntoView({block:'center',behavior:'instant'}); const box = node.getBoundingClientRect(); return {x:box.x+box.width/2,y:box.y+box.height/2}; })()`);
    await send("Input.dispatchMouseEvent", {type:"mousePressed", ...point, button:"left", clickCount:1});
    await send("Input.dispatchMouseEvent", {type:"mouseReleased", ...point, button:"left", clickCount:1});
  };
  await click(first);
  await until(`${popup(first)}.matches(':popover-open')`);
  await click(second);
  await until(`!${popup(first)}.matches(':popover-open') && ${popup(second)}.matches(':popover-open')`);
  await settle();
  assert.equal(await evaluate(`document.activeElement === ${second}`), true, "blur must not reclaim focus from the second selector");
  await evaluate(`${destination}.focus()`);
  await until(`!${popup(second)}.matches(':popover-open')`);
  await settle();
  assert.equal(await evaluate(`document.activeElement === ${destination}`), true, "blur must leave the destination focused");
  await click(first);
  await until(`${popup(first)}.matches(':popover-open')`);
  await press("Tab", "Tab", 9);
  await until(`!${popup(first)}.matches(':popover-open')`);
  await settle();
  assert.equal(await evaluate(`document.activeElement === ${second}`), true, "Tab must reach the next selector");
  assert.deepEqual(exceptions, []);
});

test("the context strip resolves models by provider and dims effort instead of dropping it", async () => {
  await load("fixtures=false&theme=light", 1200);
  const strip = "document.querySelector('#context-strip .fc-context-strip')";
  const combo = label => `[...${strip}.querySelectorAll('[role=combobox]')].find(b => b.getAttribute('aria-label') === '${label}')`;
  const choose = async (label, value) => {
    await evaluate(`${combo(label)}.click()`);
    await until(`document.getElementById(${combo(label)}.id + '-listbox').matches(':popover-open')`);
    await evaluate(`document.getElementById(${combo(label)}.id + '-listbox').querySelector('[data-fc-value="${value}"]').click()`);
    await until(`${combo(label)}.getAttribute('aria-expanded') === 'false'`);
  };
  assert.equal(await evaluate(`${strip}.dataset.fcModel`), "opus");
  assert.equal(await evaluate(`${strip}.dataset.fcEffort`), "medium");
  await choose("Model", "sonnet");
  await until(`${strip}.dataset.fcModel === 'sonnet'`);
  await choose("Provider", "local");
  await until(`${strip}.dataset.fcModel === 'llama' && !('fcEffort' in ${strip}.dataset)`);
  assert.equal(await evaluate(`${combo("Effort")}.getAttribute('aria-disabled')`), "true");
  assert.equal(await evaluate(`${strip}.querySelector('[data-fc-applicable]').dataset.fcApplicable`), "false");
  assert.ok(await evaluate(`${strip}.querySelector('.fc-context-strip__notice').textContent.length > 0`));
  await choose("Provider", "anthropic");
  await until(`${strip}.dataset.fcModel === 'sonnet' && ${strip}.dataset.fcEffort === 'medium'`);
});

test("the composer sends on Enter, keeps Shift+Enter, becomes Stop, and names each remove button", async () => {
  await load("fixtures=false&theme=light", 1200);
  const root = "document.querySelector('#composer .fc-composer')";
  const input = `${root}.querySelector('textarea')`;
  const sendButton = `${root}.querySelector('.fc-composer__send')`;
  assert.equal(await evaluate(`${root}.dataset.fcState`), "ready");
  assert.deepEqual(await evaluate(`[...${root}.querySelectorAll('.fc-composer__attachment-remove')].map(b => b.getAttribute('aria-label'))`),
    ["Remove notes.md", "Remove a-very-long-file-name-that-is-truncated.pdf"]);
  await evaluate(`${input}.focus()`);
  await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13, modifiers: 8, text: "\r" });
  await send("Input.dispatchKeyEvent", { type: "keyUp", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13, modifiers: 8 });
  await until(`${input}.value.includes('\\n')`);
  assert.equal(await evaluate(`${root}.dataset.fcState`), "ready", "Shift+Enter does not send");
  await press("Enter", "Enter", 13);
  await until(`${root}.dataset.fcState === 'running' && ${sendButton}.textContent === 'Stop' && ${input}.readOnly`);
  assert.equal(await evaluate(`${sendButton}.getAttribute('aria-label')`), "Stop generating");
  assert.equal(await evaluate(`${input}.value`), "", "the accepted draft is cleared");
  await evaluate(`${sendButton}.click()`);
  await until(`${root}.dataset.fcState === 'idle' && ${sendButton}.disabled`);
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

test("the git surface sorts conflicts first, roves with arrows, and stages without selecting", async () => {
  await load("fixtures=false&theme=light", 1200);
  const surface = "document.querySelector('#git .fc-git')";
  const rows = `[...${surface}.querySelectorAll('[role=row]')]`;
  assert.equal(await evaluate(`${rows}[0].dataset.fcPath`), "src/js/merge.ts", "the conflict is first");
  assert.equal(await evaluate(`${rows}[0].getAttribute('aria-selected')`), "true", "the first file starts selected");
  assert.equal(await evaluate(`${rows}.filter(r => r.tabIndex === 0).length`), 1, "one tab stop");
  assert.equal(await evaluate(`${surface}.dataset.fcConflicted`), "1");
  // Rows keep their own layout; grid.css styles only the grid's rows.
  assert.equal(await evaluate(`getComputedStyle(${rows}[0]).display`), "flex");
  await evaluate(`${rows}[0].focus()`);
  await press("ArrowDown", "ArrowDown", 40);
  await until(`${rows}[1].getAttribute('aria-selected') === 'true' && document.activeElement === ${rows}[1]`);
  await press("End", "End", 35);
  await until(`document.activeElement === ${rows}.at(-1)`);
  await until(`document.querySelector('#git [role=status]').textContent === 'Selected ' + ${rows}.at(-1).dataset.fcPath`);
  const stage = `${surface}.querySelector('.fc-git__stage-button')`;
  await evaluate(`${stage}.focus()`);
  await press("Enter", "Enter", 13);
  await until(`document.querySelector('#git [role=status]').textContent === 'Stage requested for ' + ${stage}.closest('[role=row]').dataset.fcPath`);
  assert.equal(await evaluate(`${rows}.at(-1).getAttribute('aria-selected')`), "true", "staging does not move the selection");
  assert.equal(await evaluate("document.querySelectorAll('#git .fc-git')[1].querySelector('.fc-git__empty').textContent"), "No changes");
  assert.equal(await evaluate("document.querySelectorAll('#git .fc-git')[2].dataset.fcBranchState"), "unborn");
});

test("the data grid lays cells in columns, extends selection with Shift, and edits in place", async () => {
  await load("fixtures=false&theme=light", 1200);
  const grid = "document.querySelector('#grid .fc-grid')";
  const cells = `[...${grid}.querySelectorAll('[role=gridcell]')]`;
  const at = (r, c) => `${cells}[${r} * 3 + ${c}]`;
  // Rows are display: contents, so without a template every cell stacks in one column.
  assert.equal(await evaluate(`new Set(${cells}.slice(0, 3).map(c => Math.round(c.getBoundingClientRect().top))).size`), 1, "a row's cells share a line");
  assert.equal(await evaluate(`${cells}.filter(c => c.tabIndex === 0).length`), 1, "one tab stop");
  await evaluate(`${at(0, 0)}.focus()`);
  await press("ArrowRight", "ArrowRight", 39);
  await until(`document.activeElement === ${at(0, 1)} && ${at(0, 1)}.dataset.fcActive === 'true'`);
  await send("Input.dispatchKeyEvent", { type: "keyDown", key: "ArrowDown", code: "ArrowDown", windowsVirtualKeyCode: 40, modifiers: 8 });
  await send("Input.dispatchKeyEvent", { type: "keyUp", key: "ArrowDown", code: "ArrowDown", windowsVirtualKeyCode: 40, modifiers: 8 });
  await until(`document.activeElement === ${at(1, 1)}`);
  assert.deepEqual(await evaluate(`${cells}.map(c => c.getAttribute('aria-selected'))`),
    ["false", "true", "false", "false", "true", "false", "false", "false", "false"], "Shift extends from the anchor");
  await press("ArrowDown", "ArrowDown", 40);
  await until(`document.activeElement === ${at(2, 1)} && ${cells}.filter(c => c.getAttribute('aria-selected') === 'true').length === 1`);
  await press("Enter", "Enter", 13);
  await until(`document.activeElement?.classList.contains('fc-grid__input')`);
  assert.equal(await evaluate("document.activeElement.value"), "anthropic");
  await evaluate("(() => { const i = document.activeElement; i.value = 'other'; i.dispatchEvent(new Event('input', { bubbles: true })); })()");
  await press("Escape", "Escape", 27);
  await until(`document.activeElement === ${at(2, 1)} && ${at(2, 1)}.textContent === 'anthropic'`);
  await press("F2", "F2", 113);
  await until(`document.activeElement?.classList.contains('fc-grid__input')`);
  // Clicking into the editor keeps it open, and it is the grid's only tab stop.
  await evaluate("document.activeElement.click()");
  await sleep(150);
  assert.equal(await evaluate("document.activeElement?.classList.contains('fc-grid__input')"), true, "a click inside the editor does not commit it");
  assert.equal(await evaluate(`${cells}.filter(c => c.tabIndex === 0).length`), 0, "the editing cell gives up its tab stop");
  await evaluate("(() => { const i = document.activeElement; i.value = 'local'; i.dispatchEvent(new Event('input', { bubbles: true })); })()");
  await press("Enter", "Enter", 13);
  await until(`${at(2, 1)}.textContent === 'local' && ${at(2, 1)}.dataset.fcDirty === 'true' && document.activeElement === ${at(2, 1)}`);
  await until("document.querySelector('#grid [role=status]').textContent === 'Edited row 3 to local'");
  const windowed = "document.querySelectorAll('#grid .fc-grid')[2]";
  assert.equal(await evaluate(`${windowed}.getAttribute('aria-rowcount')`), "1001", "the true size counts the header row");
  assert.equal(await evaluate(`${windowed}.querySelectorAll('[role=row]')[1].getAttribute('aria-rowindex')`), "501");
});

test("the terminal runs commands, recalls history, follows output, and disables a dropped session", async () => {
  await load("fixtures=false&theme=light", 1200);
  const term = "document.querySelector('#terminal .fc-terminal')";
  const input = `${term}.querySelector('.fc-terminal__input')`;
  const output = `${term}.querySelector('[role=log]')`;
  const lines = `[...${output}.querySelectorAll('.fc-terminal__line')]`;
  assert.equal(await evaluate(`${output}.getAttribute('aria-live')`), "polite");
  assert.equal(await evaluate(`${term}.dir`), "ltr", "output is left-to-right in every locale");
  const run = async command => {
    await evaluate(`(() => { const i = ${input}; i.focus(); i.value = ${JSON.stringify(command)}; i.dispatchEvent(new Event('input', { bubbles: true })); })()`);
    await until(`${input}.value === ${JSON.stringify(command)}`);
    await press("Enter", "Enter", 13);
  };
  await run("help");
  await until(`${lines}.at(-1).textContent.startsWith('commands:') && ${input}.value === ''`);
  await run("nope");
  await until(`${term}.querySelector('.fc-terminal__exit').textContent === 'exit 127' && ${term}.querySelector('.fc-terminal__exit').dataset.fcState === 'error'`);
  await press("ArrowUp", "ArrowUp", 38);
  await until(`${input}.value === 'nope'`);
  await press("ArrowUp", "ArrowUp", 38);
  await until(`${input}.value === 'help'`);
  await press("ArrowDown", "ArrowDown", 40);
  await until(`${input}.value === 'nope'`);
  await press("ArrowDown", "ArrowDown", 40);
  await until(`${input}.value === ''`);
  for (let i = 0; i < 12; i++) await run("colors");
  await until(`${output}.scrollHeight > ${output}.clientHeight && ${output}.scrollTop + ${output}.clientHeight >= ${output}.scrollHeight - 2`);
  assert.equal(await evaluate(`${lines}.at(-1).querySelectorAll('.fc-terminal__cursor--blink').length`), 1, "the cursor follows the last line");
  // Scrolled up to read, the reader stays put while output keeps arriving.
  await run("watch");
  await until(`${output}.scrollTop + ${output}.clientHeight >= ${output}.scrollHeight - 2`);
  await evaluate(`${output}.scrollTop = 0`);
  await until(`${lines}.at(-1).textContent.startsWith('watch rebuilt in 64ms')`);
  assert.equal(await evaluate(`${output}.scrollTop`), 0, "new output does not pull a reader back down");
  await run("clear");
  await until(`${lines}.length === 1 && ${lines}[0].textContent.trim() === '' && ${lines}[0].querySelector('.fc-terminal__cursor')`);
  const dropped = "document.querySelectorAll('#terminal .fc-terminal')[1]";
  assert.equal(await evaluate(`${dropped}.dataset.fcState`), "disconnected");
  assert.equal(await evaluate(`${dropped}.querySelector('.fc-terminal__input').disabled`), true);
  assert.equal(await evaluate(`${dropped}.querySelectorAll('.fc-terminal__cursor').length`), 0, "no cursor in a dropped session");
});

test("the webview sandboxes its frame, keeps its own history, and refuses unsafe addresses", async () => {
  await load("fixtures=false&theme=light", 1200);
  const view = "document.querySelector('#webview .fc-webview')";
  const address = `${view}.querySelector('.fc-webview__address-input')`;
  const tabs = `[...${view}.querySelectorAll('[role=tab]')]`;
  const button = name => `${view}.querySelector('.fc-webview__toolbar [aria-label=${name}]')`;
  const frame = `${view}.querySelector('iframe')`;
  assert.equal(await evaluate(`${frame}.getAttribute('sandbox')`), "allow-scripts allow-forms allow-popups");
  assert.equal(await evaluate(`${frame}.referrerPolicy`), "strict-origin-when-cross-origin");
  assert.equal(await evaluate(`${address}.value`), "https://example.com/");
  assert.equal(await evaluate(`${view}.querySelector('.fc-webview__secure').textContent`), "\u{1f512}Secure");
  assert.equal(await evaluate(`${button("Back")}.disabled`), true, "nothing behind the first page");
  await evaluate(`${tabs}[0].focus()`);
  await press("ArrowRight", "ArrowRight", 39);
  await until(`document.activeElement === ${tabs}[1] && ${tabs}[1].getAttribute('aria-selected') === 'true' && ${address}.value === 'https://example.org/'`);
  await until(`!${button("Back")}.disabled`);
  await evaluate(`${button("Back")}.click()`);
  await until(`${address}.value === 'https://example.com/' && ${button("Back")}.disabled && !${button("Forward")}.disabled`);
  await evaluate(`(() => { const i = ${address}; i.value = 'java\\nscript:alert(1)'; i.dispatchEvent(new Event('input', { bubbles: true })); })()`);
  await evaluate(`${address}.focus()`);
  await press("Enter", "Enter", 13);
  await until(`${view}.dataset.fcState === 'error' && !${view}.querySelector('.fc-webview__overlay').hidden`);
  assert.equal(await evaluate(`${frame}.getAttribute('src')`), "https://example.com/", "the refused address never reaches the frame");
  await until("document.querySelector('#webview [role=status]').textContent === 'Refused or failed to load'");
  await evaluate(`${view}.querySelector('.fc-webview__overlay button').click()`);
  await until(`${view}.dataset.fcState !== 'error' && ${view}.querySelector('.fc-webview__overlay').hidden`);
  assert.equal(await evaluate(`${address}.value`), "https://example.com/", "Retry shows the page it reloads, not the refused address");
  // Reload navigates the frame again, so a load event follows.
  await until(`${view}.dataset.fcState === 'loaded'`);
  await evaluate(`${button("Reload")}.click()`);
  await until(`${view}.dataset.fcState === 'loading'`);
  await until(`${view}.dataset.fcState === 'loaded'`);
  const plain = "document.querySelectorAll('#webview .fc-webview')[1]";
  assert.equal(await evaluate(`${plain}.querySelector('.fc-webview__secure').dataset.fcState`), "insecure");
  const refused = "document.querySelectorAll('#webview .fc-webview')[2]";
  assert.equal(await evaluate(`${refused}.querySelector('iframe').getAttribute('src')`), "about:blank", "a refused first page loads nothing, not the host page");
  assert.equal(await evaluate(`${refused}.querySelector('[role=alert]').textContent.startsWith('This page could not be loaded.')`), true);
});

test("the outermost root mirrors its scheme onto <html>, in order, and System clears it", async () => {
  await load("fixtures=false&theme=light", 1200);
  const root = "document.querySelector('[data-fc-version]')";
  await until(`document.documentElement.dataset.fcTheme === 'light' && document.documentElement.dataset.fcDesign === ${root}.dataset.fcDesign`);
  const choose = value => `(() => { const s = document.getElementById('theme'); s.value = '${value}'; s.dispatchEvent(new Event('change', { bubbles: true })); })()`;
  await evaluate(choose("dark"));
  await until("document.documentElement.dataset.fcTheme === 'dark'");
  // Back-to-back changes start overlapping mirror scripts; only the last may stand.
  await evaluate(`${choose("light")}; ${choose("dark")}; ${choose("light")}; ${choose("dark")}`);
  await until(`${root}.dataset.fcTheme === 'dark' && document.documentElement.dataset.fcTheme === 'dark'`);
  await sleep(300);
  assert.equal(await evaluate("document.documentElement.dataset.fcTheme"), "dark", "a late, older update must not win");
  await evaluate(choose("system"));
  await until(`!(${root}.hasAttribute('data-fc-theme')) && !document.documentElement.hasAttribute('data-fc-theme')`);
  assert.deepEqual(exceptions, []);
});

test("the document theme owner keeps sibling roots, restores the page's own value, and drops stale updates", async () => {
  await load("fixtures=false&theme=light", 1200);
  // The same script the roots run, driven directly with a name no root uses, so sibling
  // roots and out-of-order delivery can be produced on demand.
  const script = readFileSync(new URL("../../crates/fc-dioxus/src/core/document-theme.js", import.meta.url), "utf8");
  const run = (token, generation, value) => evaluate(`(async () => { const dioxus = { recv: async () => ${JSON.stringify([token, generation, value === undefined ? null : [["data-test-theme", value]]])} }; ${script} })()`);
  const current = () => evaluate("document.documentElement.getAttribute('data-test-theme')");
  await evaluate("document.documentElement.setAttribute('data-test-theme', 'app')");
  assert.equal(await run(9001, 1, "light"), "ok");
  assert.equal(await run(9002, 1, "light"), "ok");
  assert.equal(await current(), "light");
  await run(9001, 2);
  assert.equal(await current(), "light", "a sibling that leaves keeps the value another root still owns");
  await run(9002, 2, "dark");
  assert.equal(await run(9002, 2, "light"), "stale");
  assert.equal(await current(), "dark");
  await run(9002, 4, null);
  assert.equal(await current(), "app", "the page's own value returns when the last owner lets go");
  assert.equal(await run(9002, 3, "light"), "stale", "an update older than the release is dropped");
  await run(9001, 1, "dark");
  assert.equal(await current(), "app", "a released root is not revived");
  await evaluate("document.documentElement.removeAttribute('data-test-theme')");
  await run(9003, 1, "light");
  await run(9003, 2);
  assert.equal(await current(), null, "a page that had nothing gets nothing back");
});

test("the theme picker previews each design and drives a nested FutureCss", async () => {
  await load("fixtures=false&theme=light", 1200);
  const picker = "document.querySelector('#theme-picker .fc-theme-picker')";
  const sample = "document.querySelector('#theme-picker .fc-card').closest('[data-fc-design]')";
  const options = `[...${picker}.querySelectorAll('.fc-theme-picker__option')]`;
  assert.equal(await evaluate(`${options}.every(o => o.dataset.fcDesign && o.dataset.fcPalette)`), true, "every card wears its own design");
  assert.equal(await evaluate(`${sample}.dataset.fcDesign`), "helix");
  const target = await evaluate(`${options}.find(o => !o.querySelector('input').checked).dataset.fcDesign`);
  await evaluate(`${options}.find(o => o.dataset.fcDesign === ${JSON.stringify(target)}).querySelector('input').click()`);
  await until(`${sample}.dataset.fcDesign === ${JSON.stringify(target)} && ${picker}.dataset.fcDesignSelection === ${JSON.stringify(target)}`);
  const paired = await evaluate(`${options}.find(o => o.dataset.fcDesign === ${JSON.stringify(target)}).dataset.fcPalette`);
  assert.equal(await evaluate(`${sample}.dataset.fcPalette`), paired, "a design brings its paired palette");
  const select = label => `[...${picker}.querySelectorAll('.fc-theme-picker__field')].find(l => l.textContent.startsWith(${JSON.stringify(label)})).querySelector('select')`;
  const choose = async (label, value) => {
    await evaluate(`(() => { const s = ${select(label)}; s.value = ${JSON.stringify(value)}; s.dispatchEvent(new Event('change', { bubbles: true })); })()`);
  };
  await choose("Color scheme", "dark");
  await until(`${sample}.dataset.fcTheme === 'dark' && ${picker}.querySelector('[role=status]').textContent.endsWith('· dark')`);
  await choose("Contrast", "high");
  await until(`${sample}.dataset.fcTheme === 'high-contrast-dark' && ${options}.every(o => o.dataset.fcTheme === 'high-contrast-dark')`);
  assert.ok((await evaluate(`${picker}.querySelector('[role=status]').textContent`)).endsWith("· high contrast"));
  await choose("Color palette", "default");
  await until(`${sample}.dataset.fcPalette === 'default' && ${picker}.dataset.fcPaletteSelection === 'default'`);
});

test("the editor runs upstream's binding: marks report across the selection and removal keeps text", async () => {
  await load("fixtures=false&theme=light", 1200);
  const editor = "document.querySelector('#editor .fc-editor')";
  const content = `${editor}.querySelector('.fc-editor__content')`;
  const button = mark => `${editor}.querySelector('[data-fc-mark=${mark}]')`;
  // The binding makes the toolbar a roving set: one tab stop.
  await until(`[...${editor}.querySelectorAll('.fc-toolbar__item')].filter(b => b.tabIndex === 0).length === 1`);
  const select = async (text) => {
    await evaluate(`(() => {
      const content = ${content};
      content.focus();
      const walker = document.createTreeWalker(content, NodeFilter.SHOW_TEXT);
      for (let node = walker.nextNode(); node; node = walker.nextNode()) {
        const at = node.textContent.indexOf(${JSON.stringify(text)});
        if (at < 0) continue;
        const range = document.createRange();
        range.setStart(node, at);
        range.setEnd(node, at + ${text.length});
        const selection = getSelection();
        selection.removeAllRanges();
        selection.addRange(range);
        return;
      }
    })()`);
  };
  await select("bold");
  await until(`${button("bold")}.getAttribute('aria-pressed') === 'true' && document.querySelector('#editor [role=status]').textContent === 'Marks: Bold'`);
  await select("Some");
  await until(`${button("bold")}.getAttribute('aria-pressed') === 'false'`);
  await evaluate(`${button("italic")}.click()`);
  await until(`${content}.querySelector('em')?.textContent === 'Some' && ${button("italic")}.getAttribute('aria-pressed') === 'true'`);
  await until("document.querySelector('#editor pre').textContent.includes('<em>Some</em>')");
  await evaluate(`${button("italic")}.click()`);
  await until(`!${content}.querySelector('em') && ${content}.textContent === 'Some bold and code.'`);
  await evaluate(`${editor}.querySelector('[data-fc-block=h2]').click()`);
  await until(`${content}.querySelector('h2')?.textContent.startsWith('Some')`);
  const readonly = "document.querySelectorAll('#editor .fc-editor')[3]";
  assert.equal(await evaluate(`${readonly}.querySelector('.fc-editor__content').isContentEditable`), false);
  assert.equal(await evaluate(`${readonly}.querySelector('[data-fc-mark=bold]').disabled`), true);
});

test("an editor removed from the page releases its document listeners", async () => {
  await load("fixtures=false&theme=light", 1200);
  const listeners = async () => {
    const { result } = await send("Runtime.evaluate", { expression: "document" });
    const { listeners } = await send("DOMDebugger.getEventListeners", { objectId: result.objectId });
    return listeners.filter(l => l.type === "selectionchange").length;
  };
  const editors = "document.querySelectorAll('#editor .fc-editor').length";
  // Every mounted editor binds once its module loads.
  await until(`[...document.querySelectorAll('#editor .fc-toolbar')].every(t => [...t.querySelectorAll('button')].some(b => b.tabIndex === 0))`);
  const mounted = await listeners();
  const toggle = "[...document.querySelectorAll('#editor button')].find(b => b.textContent.includes('new document'))";
  await evaluate(`${toggle}.click()`);
  await until(`${editors} === 4`);
  await until(`(async () => true)()`);
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline && await listeners() !== mounted - 1) await sleep(80);
  assert.equal(await listeners(), mounted - 1, "the removed editor's selectionchange listener is gone");
  await evaluate(`${toggle}.click()`);
  await until(`${editors} === 5`);
  while (Date.now() < deadline + 5000 && await listeners() !== mounted) await sleep(80);
  assert.equal(await listeners(), mounted, "a remounted editor binds exactly once");
});
