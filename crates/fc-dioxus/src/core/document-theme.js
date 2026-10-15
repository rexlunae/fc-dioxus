// One owner for the theme attributes on <html>, shared by every outermost root in the
// document. `<html>` is a single element, so a root cannot tell from the attribute alone
// whether a sibling or the application wrote the same value; this registry records who
// owns each name, and what the document had before any root took it.
const [token, generation, pairs] = await dioxus.recv();
const html = document.documentElement;
const state = (globalThis.__fcDocumentTheme ??= { roots: new Map(), original: new Map(), clock: 0 });
const previous = state.roots.get(token);
// Each update runs as its own script and they can finish out of order. A late, older
// update must not undo a newer one, nor revive a root that has already released.
if (previous && previous.generation >= generation) return "stale";
const valueOf = (entry, name) => entry.pairs?.find(([n]) => n === name)?.[1] ?? null;
const owned = name => [...state.roots.values()].some(entry => valueOf(entry, name) !== null);
const names = new Set([...(previous?.pairs ?? []), ...(pairs ?? [])].map(([name]) => name));
for (const name of names) {
  if (!state.original.has(name) && !owned(name) && pairs?.some(([n, v]) => n === name && v !== null)) {
    state.original.set(name, html.getAttribute(name));
  }
}
// A released root keeps its generation, with no pairs, so the guard above still holds.
state.roots.set(token, { generation, pairs, at: ++state.clock });
for (const name of names) {
  // The root that wrote most recently wins; when the last owner leaves, the document
  // gets back whatever it had before, including nothing.
  let winner = null;
  for (const entry of state.roots.values()) {
    const value = valueOf(entry, name);
    if (value !== null && (!winner || entry.at > winner.at)) winner = { at: entry.at, value };
  }
  let value;
  if (winner) value = winner.value;
  else if (state.original.has(name)) { value = state.original.get(name); state.original.delete(name); }
  else continue;
  if (value === null) html.removeAttribute(name);
  else html.setAttribute(name, value);
}
return "ok";
