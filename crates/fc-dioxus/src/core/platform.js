const [operation, data, option] = await dioxus.recv();
try {
  if (operation === "copy") {
    if (!navigator.clipboard?.writeText) return "unsupported";
    await navigator.clipboard.writeText(data);
    return "ok";
  }
  const node = document.getElementById(data);
  if (!node) return "missing";
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
