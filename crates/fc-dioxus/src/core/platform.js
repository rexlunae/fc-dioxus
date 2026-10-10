const [operation, data, option] = await dioxus.recv();
try {
  if (operation === "copy") {
    if (!navigator.clipboard?.writeText) return "unsupported";
    await navigator.clipboard.writeText(data);
    return "ok";
  }
  const node = document.getElementById(data);
  if (!node) return "missing";
  if (operation === "edges" || operation === "page") {
    if (operation === "page") {
      const rtl = getComputedStyle(node).direction === "rtl";
      // `left` is physical; in RTL the inline-forward direction is negative.
      node.scrollBy({ left: (option === "next" ? 1 : -1) * node.clientWidth * 0.85 * (rtl ? -1 : 1) });
      return "ok";
    }
    // scrollLeft runs negative in RTL, from 0 at the start towards the end.
    const offset = Math.abs(node.scrollLeft);
    const max = node.scrollWidth - node.clientWidth;
    return `${offset <= 1} ${offset >= max - 1}`;
  }
  if (operation === "place") {
    // Upstream select.ts: under the invoker, flipping above when there is no room,
    // with both block edges always written so a stale edge cannot stretch the list.
    const invoker = document.getElementById(option);
    if (!invoker) return "missing";
    const anchor = invoker.getBoundingClientRect();
    const below = window.innerHeight - anchor.bottom;
    const above = anchor.top;
    const GAP = 2;
    const flip = below < 160 && above > below;
    const style = node.style;
    style.setProperty("inset-block-start", flip ? "auto" : `${anchor.bottom + GAP}px`);
    style.setProperty("inset-block-end", flip ? `${above - GAP}px` : "auto");
    style.setProperty("inset-inline-start", `${anchor.left}px`);
    style.setProperty("min-inline-size", `${anchor.width}px`);
    const available = (flip ? above : below) - GAP;
    style.setProperty("max-block-size", `${Math.max(80, Math.min(available, 384))}px`);
    return "ok";
  }
  if (operation === "capture") {
    // Pointer capture keeps a drag's moves coming to the handle wherever the pointer goes.
    try { node.setPointerCapture(Number(option)); return "ok"; } catch { return "rejected"; }
  }
  if (operation === "indeterminate") {
    if (!("indeterminate" in node)) return "unsupported";
    node.indeterminate = option === "on";
    return "ok";
  }
  const method = {
    modal: "showModal", close: "close", focus: "focus", scroll: "scrollIntoView",
    popover: option === "open" ? "showPopover" : "hidePopover",
  }[operation];
  if (!method || typeof node[method] !== "function") return "unsupported";
  if (operation === "scroll") node[method]({ block: option, behavior: "instant" });
  else node[method]();
  return "ok";
} catch {
  return "rejected";
}
