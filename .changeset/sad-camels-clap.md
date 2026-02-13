---
"@takumi-rs/wasm": minor
---

**`WasmBuffer` class for zero-copy rendering**

Before:

```tsx
const buffer = renderer.render();
```

After (with `using` keyword):

```tsx
using buffer = renderer.render().asUint8Array();
// buffer is automatically disposed when it goes out of scope
```

After (manual freeing):

```tsx
const buffer = renderer.render();
const bytes = buffer.asUint8Array();

buffer.free();

// If you forget to free the buffer, it will be leaked.
// Do NOT attempt to read from bytes after freeing to avoid use-after-free exploit.
```
