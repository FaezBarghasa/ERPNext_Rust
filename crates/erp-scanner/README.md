# 📱 erp-scanner (`rust-web-land`)

> **Mobile & Handheld Optical Scanner Subsystem for `rust-web-land` (`rwl`)**.  
> Camera stream decode, Zebra/Honeywell rugged handheld hardware scanner FFI, 1D/2D symbologies, and offline batch scanning buffers.

---

## 📦 Key Capabilities

- **Multi-Symbology Support**: High-speed decoding of QR Code, Code 128, DataMatrix, EAN-13, and PDF417.
- **Rugged Scanner Integration**: Native hardware intent listeners for Honeywell and Zebra Android handhelds.
- **Offline Batch Buffer**: Queues scans in local indexed storage when network connectivity is disrupted.
- **Core Engine Integration**: Direct pipeline to [`erp-scanner-core`](../erp-scanner-core) and [`erp-wms`](../erp-wms).

---

## 🛠️ Usage Example

```rust
use erp_scanner::barcode::{BarcodeDecoder, Symbology};

let decoder = BarcodeDecoder::default();
let result = decoder.decode_frame(&raw_camera_buffer, Symbology::QrCode)?;
println!("Scanned item: {}", result.payload);
```
