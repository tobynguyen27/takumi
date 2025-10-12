import { fromJsx } from "@takumi-rs/helpers/jsx";
import DocsTemplateV1 from "@takumi-rs/template/docs-template-v1";
import initWasm, { Renderer } from "@takumi-rs/wasm";
import wasmUrl from "@takumi-rs/wasm/takumi_wasm_bg.wasm?url";
import * as React from "react";
import { transform } from "sucrase";

let renderer: Renderer | undefined;

initWasm({ module_or_path: wasmUrl }).then(async () => {
  const font = await fetch("/fonts/Geist.woff2").then((r) => r.arrayBuffer());

  renderer = new Renderer();
  renderer.loadFont(new Uint8Array(font));

  self.postMessage({ type: "ready" });
});

function require(module: string) {
  if (module === "@takumi-rs/template/docs-template-v1") return DocsTemplateV1;
}

function transformCode(code: string) {
  return transform(code, {
    transforms: ["jsx", "typescript", "imports"],
    production: true,
  }).code;
}

function componentFromCode(code: string) {
  const exports = {};

  new Function("exports", "require", "React", transformCode(code))(
    exports,
    require,
    React,
  );

  if (!("default" in exports) || typeof exports.default !== "function")
    throw new Error("Default export should be a React component.");

  return exports.default as React.JSXElementConstructor<unknown>;
}

self.onmessage = async (event: MessageEvent) => {
  const { type, code } = event.data;

  if (type === "render" && renderer) {
    try {
      const component = componentFromCode(code);
      const node = await fromJsx(React.createElement(component));

      const start = performance.now();
      const dataUrl = renderer.renderAsDataUrl(node, 1200, 630, "png");
      const duration = performance.now() - start;

      self.postMessage({
        type: "render_complete",
        dataUrl,
        duration,
        node,
      });
    } catch (error) {
      self.postMessage({
        type: "render_error",
        error: error instanceof Error ? error.message : "Unknown error",
      });
    }
  }
};
