import { createServer } from "node:http";
import { readFileSync, existsSync } from "node:fs";
import { join, resolve, extname, sep } from "node:path";
import { pathToFileURL } from "node:url";
import { root } from "./sync-upstream.mjs";

export async function serve(port = 0) {
  const publicDir = join(root, "target/dx/fc-demo/debug/web/public");
  if (!existsSync(join(publicDir, "index.html"))) throw new Error("Build the web demo first: cargo xtask demo --renderer web --build-only");
  const mime = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".wasm": "application/wasm", ".css": "text/css", ".json": "application/json" };
  const server = createServer((request, response) => {
    try {
      const url = new URL(request.url, "http://localhost");
      const file = resolve(publicDir, "." + decodeURIComponent(url.pathname === "/" ? "/index.html" : url.pathname));
      if (!file.startsWith(publicDir + sep)) { response.writeHead(403).end(); return; }
      const content = readFileSync(file);
      response.writeHead(200, { "Content-Type": mime[extname(file)] ?? "application/octet-stream" });
      response.end(content);
    } catch { if (!response.headersSent) response.writeHead(404); response.end(); }
  });
  await new Promise((resolve, reject) => { server.once("error", reject); server.listen(port, "127.0.0.1", resolve); });
  return { origin: `http://127.0.0.1:${server.address().port}`, close: () => new Promise(resolve => server.close(resolve)) };
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const server = await serve(Number(process.env.PORT ?? 8087));
  console.log(`Demo: ${server.origin}`);
}
