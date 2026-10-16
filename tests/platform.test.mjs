import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { test } from "node:test";
import { checkout } from "../tools/sync-upstream.mjs";

const { FutureCssSelect } = await import(pathToFileURL(join(checkout(), "src/js/select.ts")));
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
const bridge = new AsyncFunction("dioxus", "document", "window", readFileSync(new URL("../crates/fc-dioxus/src/core/platform.js", import.meta.url), "utf8"));

test("select placement matches upstream and keeps flipped lists inside the viewport", async () => {
  const previous = Object.getOwnPropertyDescriptor(globalThis, "window");
  const viewport = { innerHeight: 800 };
  Object.defineProperty(globalThis, "window", { configurable: true, value: viewport });
  try {
    for (const anchor of [
      { top: 730, bottom: 760, left: 200, width: 120 },
      { top: 100, bottom: 130, left: 20, width: 200 },
    ]) {
      const actual = new Map();
      const expected = new Map();
      const invoker = { getBoundingClientRect: () => anchor };
      const list = { style: { setProperty: (key, value) => actual.set(key, value) } };
      const doc = { getElementById: id => id === "list" ? list : invoker };
      assert.equal(await bridge({ recv: async () => ["place", "list", "trigger"] }, doc, viewport), "ok");
      const binding = Object.assign(Object.create(FutureCssSelect.prototype), { invoker, list: { style: { setProperty: (key, value) => expected.set(key, value) } } });
      binding.position();
      assert.deepEqual(actual, expected, "the host bridge and pinned upstream agree");
      if (actual.get("inset-block-start") === "auto") {
        const bottom = viewport.innerHeight - parseFloat(actual.get("inset-block-end"));
        assert.equal(bottom, anchor.top - 2);
        assert.ok(bottom - 190 >= 0, "the popup's first row stays onscreen");
      } else {
        assert.equal(parseFloat(actual.get("inset-block-start")), anchor.bottom + 2);
        assert.equal(actual.get("inset-block-end"), "auto");
      }
    }
  } finally {
    if (previous) Object.defineProperty(globalThis, "window", previous);
    else Reflect.deleteProperty(globalThis, "window");
  }
});
