# `@rust-web-land/sdk` (`@rwl/sdk`)

Official typed TypeScript client SDK for interacting with the `rust-web-land` (`rwl`) backend, DocType models, and v2 APIs.

## Installation

```bash
pnpm add @rust-web-land/sdk
# or
npm install @rust-web-land/sdk
```

## Quick Start

```typescript
import { RustWebLandClient } from "@rust-web-land/sdk";

const client = new RustWebLandClient({
  baseUrl: "https://app.yourdomain.com",
  apiKey: "rwl_sec_...",
  site: "default.localhost"
});

// Fetch documents with typed filters
const items = await client.listDocuments("Item", {
  is_stock_item: 1
}, 50);

console.log(`Found ${items.length} items`);
```
