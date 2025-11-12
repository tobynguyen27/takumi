import { describe, expect, test } from "bun:test";
import init from "@takumi-rs/wasm";
import ImageResponse from "../src/backends/wasm";

await init({
  module_or_path: fetch(
    import.meta.resolve("@takumi-rs/wasm/takumi_wasm_bg.wasm"),
  ),
});

describe("ImageResponse", () => {
  test("should not crash", async () => {
    const response = new ImageResponse(<div tw="bg-black w-4 h-4" />);

    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("image/webp");

    expect(await response.arrayBuffer()).toBeDefined();
  });

  test("should set content-type", async () => {
    const response = new ImageResponse(<div tw="bg-black w-4 h-4" />, {
      width: 100,
      height: 100,
      format: "png",
    });

    expect(response.headers.get("content-type")).toBe("image/png");
    expect(await response.arrayBuffer()).toBeDefined();
  });
});
