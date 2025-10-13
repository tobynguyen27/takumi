import { fromJsx } from "@takumi-rs/helpers/jsx";
import DocsTemplateV1 from "@takumi-rs/template/docs-template-v1";
import initWasm, { Renderer } from "@takumi-rs/wasm";
import wasmUrl from "@takumi-rs/wasm/takumi_wasm_bg.wasm?url";
import * as React from "react";
import { transform } from "sucrase";
import * as z from "zod/mini";
import { type messageSchema, optionsSchema } from "./schema";

function postMessage(message: z.input<typeof messageSchema>) {
  return self.postMessage(message);
}

const exportsSchema = z.object({
  default: z.function(),
  options: optionsSchema,
});

let renderer: Renderer | undefined;

(async () => {
  const [_, normalFont] = await Promise.all([
    initWasm({ module_or_path: wasmUrl }),
    fetch("/fonts/Geist.woff2").then((r) => r.arrayBuffer()),
  ]);

  renderer = new Renderer();
  renderer.loadFont(new Uint8Array(normalFont));

  postMessage({ type: "ready" });
})();

function require(module: string) {
  if (module === "@takumi-rs/template/docs-template-v1") return DocsTemplateV1;
}

function transformCode(code: string) {
  return transform(code, {
    transforms: ["jsx", "typescript", "imports"],
    production: true,
  }).code;
}

function evaluateCodeExports(code: string) {
  const exports = {};

  new Function("exports", "require", "React", transformCode(code))(
    exports,
    require,
    React,
  );

  return exportsSchema.parse(exports);
}

self.onmessage = async (event: MessageEvent) => {
  const { type, code } = event.data;

  if (type === "render" && renderer) {
    try {
      const { default: component, options } = evaluateCodeExports(code);
      const node = await fromJsx(
        React.createElement(component as React.JSXElementConstructor<unknown>),
      );

      const start = performance.now();
      const dataUrl = renderer.renderAsDataUrl(
        node,
        options.width,
        options.height,
        options.format,
        options.quality,
      );
      const duration = performance.now() - start;

      postMessage({
        type: "render-result",
        result: {
          status: "success",
          dataUrl,
          duration,
          node,
          options,
        },
      });
    } catch (error) {
      postMessage({
        type: "render-result",
        result: {
          status: "error",
          message: error instanceof Error ? error.message : "Unknown error",
        },
      });
    }
  }
};
