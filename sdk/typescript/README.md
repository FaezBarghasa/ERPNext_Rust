# `@rustnext/sdk`

Official typed TypeScript client library for interacting with the RustNext ERP & Frappe-Rust enterprise backend.

## Installation

```bash
pnpm add @rustnext/sdk
# or
npm install @rustnext/sdk
```

## Quick Start

```typescript
import { RustNextClient } from "@rustnext/sdk";

const client = new RustNextClient({
  baseUrl: "https://erp.example.com",
  apiKey: "rn_sec_...",
  site: "default.localhost"
});

// Fetch documents with typed filters
const items = await client.listDocuments("Item", {
  is_stock_item: 1
}, 50);

console.log(`Found ${items.length} items`);
```
