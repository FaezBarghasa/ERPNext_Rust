# ⚡ erp-scanner-core

> **Zero-Allocation `#![no_std]` Edge Scanner Engine for `rust-web-land` (`rwl`)**

`erp-scanner-core` provides bare-metal and microcontroller runtime support for barcode, QR code, and industrial symbology classification, GS1-128 Application Identifier parsing, and hardware laser wedge inter-keystroke temporal interception.

---

## ⚡ Core Capabilities

- **Zero Heap / `#![no_std]` Architecture**:
  - Operates entirely with static bounded buffers (`heapless::Vec` and `heapless::String`).
  - Safe for bare-metal ARM Cortex-M microcontrollers, RTIC v2, Embassy, and low-power IoT scanners.
- **GS1-128 Application Identifier (AI) Parser (`gs1.rs`)**:
  - Deterministic parsing of variable and fixed-length GS1-128 element strings.
  - Supports GTIN (AI 01), Serial Number (AI 21), Batch/Lot Number (AI 10), Expiration Date (AI 17), Production Date (AI 11), Net Weight (AI 310x), etc.
- **Symbology Classification Engine (`symbology.rs`)**:
  - Instant classification of QR Code, Micro QR, Data Matrix, Aztec, GS1-128, EAN-13, EAN-8, Code 128, Code 39, Code 93, ITF-14, and UPC-A/E.
- **Embedded Laser Wedge Interceptor (`wedge.rs`)**:
  - Inter-keystroke microsecond temporal burst discriminator detecting hardware laser wedge scanners from manual human typing.

---

## 📦 Workspace Architecture

```
crates/erp-scanner-core/
├── Cargo.toml
└── src/
    ├── gs1.rs          # Zero-allocation GS1-128 Application Identifier parser
    ├── lib.rs          # Public no_std exports & SymbologyCore enum
    ├── symbology.rs    # Fast format & symbology classification
    └── wedge.rs        # Microcontroller laser wedge burst detector
```

---

Part of the **[rust-web-land](file:///home/jrad/RustroverProjects/ERPNext_workspace/README.md)** ecosystem.
