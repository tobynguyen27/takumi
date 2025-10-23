import { Renderer } from "@takumi-rs/core";
import { fromJsx } from "@takumi-rs/helpers/jsx";
import { write } from "bun";
import { html } from "satori-html";

const renderer = new Renderer();

const markup = html`<div style="color: black;">hello, world</div>`;
const node = await fromJsx(markup);

const png = await renderer.render(node, {
  width: 600,
  height: 400,
});

await write("./output.png", png.buffer);
