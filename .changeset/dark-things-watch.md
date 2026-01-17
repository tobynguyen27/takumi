---
"@takumi-rs/core": minor
---

**BREAKING: Externalize image fetching**

To allow more control over fetching and match the WASM version, `@takumi-rs/core` no longer runs `fetch` for you. `@takumi-rs/image-response` will not be affected by this change.

Before:

```tsx
const renderer = new Renderer();
const node = await fromJsx(<img src="https://example.com/image.png" />);
const image = await renderer.render(node);
```

After:

```tsx
import { extractResourceUrls } from "@takumi-rs/core";
import { fetchResources } from "@takumi-rs/helpers";

const renderer = new Renderer();
const node = await fromJsx(<img src="https://example.com/image.png" />);

const urls = extractResourceUrls(node);
const fetchedResources = await fetchResources(urls);

const image = await renderer.render(node, {
  fetchedResources,
});
```
