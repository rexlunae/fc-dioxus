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
  if (operation === "point") {
    // Upstream menu.ts placeAt: a menu at a viewport point, its start edge at x by
    // default, clamped inside the viewport and moved above flipY when it would cross
    // the block end.
    const [x, y, flipY, edge] = option.split(",");
    const style = node.style;
    style.position = "fixed";
    style.insetBlockStart = `${Number(y)}px`;
    style.insetInlineEnd = "auto";
    style.insetBlockEnd = "auto";
    style.margin = "0";
    const rtl = getComputedStyle(node).direction === "rtl";
    const rect = node.getBoundingClientRect();
    const viewWidth = document.documentElement.clientWidth;
    const viewHeight = document.documentElement.clientHeight;
    const side = edge || (rtl ? "right" : "left");
    const wanted = side === "left" ? Number(x) : Number(x) - rect.width;
    const left = Math.min(Math.max(0, wanted), Math.max(0, viewWidth - rect.width));
    style.insetInlineStart = `${rtl ? viewWidth - left - rect.width : left}px`;
    if (Number(y) + rect.height > viewHeight) {
      const above = (flipY === "" ? Number(y) : Number(flipY)) - rect.height;
      style.insetBlockStart = `${above >= 0 ? above : Math.max(0, viewHeight - rect.height)}px`;
    }
    return "ok";
  }
  if (operation === "below") {
    // A menu button's menu: under the trigger's start edge, flipping above it.
    const rect = node.getBoundingClientRect();
    const rtl = getComputedStyle(node).direction === "rtl";
    return `${rtl ? rect.right : rect.left},${rect.bottom + 4},${rect.top - 4},${rtl ? "right" : "left"}`;
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
  if (operation === "reload") {
    // Reassigning src reloads any frame, cross-origin included, where
    // contentWindow.location.reload() is out of reach. Upstream's webview.ts does the same.
    if (!(node instanceof HTMLIFrameElement)) return "unsupported";
    node.src = node.src;
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
