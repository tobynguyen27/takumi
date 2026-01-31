import {
  type AnyNode,
  type ConstructRendererOptions,
  extractResourceUrls,
  Renderer,
  type RenderOptions,
} from "@takumi-rs/core";
import { fetchResources } from "@takumi-rs/helpers";
import { type FromJsxOptions, fromJsx } from "@takumi-rs/helpers/jsx";
import type { ReactNode } from "react";

let renderer: Renderer | undefined;

declare module "react" {
  interface DOMAttributes<T> {
    tw?: string;
  }
}

type ImageResponseOptionsWithRenderer = ResponseInit &
  RenderOptions & {
    renderer: Renderer;
    signal?: AbortSignal;
    jsx?: FromJsxOptions;
  };

type ImageResponseOptionsWithoutRenderer = ResponseInit &
  RenderOptions &
  ConstructRendererOptions & {
    signal?: AbortSignal;
    jsx?: FromJsxOptions;
  };

export type ImageResponseOptions =
  | ImageResponseOptionsWithRenderer
  | ImageResponseOptionsWithoutRenderer;

const defaultOptions = {
  format: "webp",
} as const satisfies ImageResponseOptions;

async function getRenderer(options?: ImageResponseOptions) {
  if (options && "renderer" in options) {
    return options.renderer;
  }

  renderer ??= new Renderer(options);

  if (options?.fonts) {
    for (const font of options.fonts) {
      await renderer.loadFont(font);
    }
  }

  if (options?.persistentImages) {
    for (const image of options.persistentImages) {
      await renderer.putPersistentImage(image.src, image.data);
    }
  }

  return renderer;
}

function extractFetchedResources(
  node: AnyNode,
  options?: ImageResponseOptions,
) {
  if (options?.fetchedResources) {
    return options.fetchedResources;
  }

  const urls = extractResourceUrls(node);

  return fetchResources(urls);
}

function createStream(component: ReactNode, options?: ImageResponseOptions) {
  return new ReadableStream({
    async start(controller) {
      try {
        const renderer = await getRenderer(options);

        const node = await fromJsx(component, options?.jsx);
        const fetchedResources = await extractFetchedResources(node, options);

        const image = await renderer.render(
          node,
          {
            ...options,
            fetchedResources,
          },
          options?.signal,
        );

        controller.enqueue(image);
        controller.close();
      } catch (error) {
        controller.error(error);
      }
    },
  });
}

const contentTypeMapping = {
  webp: "image/webp",
  png: "image/png",
  jpeg: "image/jpeg",
  WebP: "image/webp",
  Jpeg: "image/jpeg",
  Png: "image/png",
  raw: "application/octet-stream",
};

export class ImageResponse extends Response {
  constructor(component: ReactNode, options?: ImageResponseOptions) {
    const stream = createStream(component, options);
    const headers = new Headers(options?.headers);

    if (!headers.get("content-type")) {
      headers.set(
        "content-type",
        contentTypeMapping[options?.format ?? defaultOptions.format],
      );
    }

    super(stream, {
      status: options?.status,
      statusText: options?.statusText,
      headers,
    });
  }
}

export default ImageResponse;
