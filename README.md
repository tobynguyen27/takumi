<img src="./assets/images/takumi.svg" alt="Takumi" width="64" />

# Takumi

Takumi is a **image rendering engine** written in Rust and **provide bindings for Node.js, WebAssembly**. Suitable for high-throughput image rendering workloads like Open Graph images, Twitter images, etc.

For newcomers, check out the [Getting Started](https://takumi.kane.tw/docs/getting-started) documentation for installation and usage.

## Why build a satori alternative?

- All in one. No need to output SVG then have [resvg-js](https://github.com/thx/resvg-js) rendering it again to output PNG.
- Minimal binary size targets to run everywhere. Node.js, web, embedded in Rust, pre-built http server.
- Takes your existing JSX components and drops them in. It should just work.
- RTL support.
- Variable fonts support.
- WOFF2 font format support. Trims your bundle size.
- PNG, JPEG, WebP, AVIF output support.
- Host Takumi as a standalone service for easier load balancing and scaling (coming soon).
- WebP, APNG animation rendering support.

## Showcase

- Takumi's Open Graph image is generated with Takumi [(source)](./example/twitter-images/components/og-image.tsx).

  ![Takumi OG Image](./example/twitter-images/output/og-image.png)

- [shiki-image](https://github.com/pi0/shiki-image) Convert code snippets into images.

  ![Shiki Image Example](https://github.com/pi0/shiki-image/blob/7ec449c53c20ce5d3260b76cb4bf2cb32e3938e3/test/.snapshot/image.webp)

- [Takumi Playground](https://takumi-playground.kapadiya.net/) In‑browser playground for designing and rendering Open Graph style images.

## License

Licensed under the terms in the [LICENSE](LICENSE) file.
