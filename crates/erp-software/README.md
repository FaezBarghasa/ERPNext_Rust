# 💻 erp-software (`rust-web-land`)

> **Software, Subscription & IT Asset Management Engine for `rust-web-land` (`rwl`)**.  
> License tracking, seat allocation, subscription renewal triggers, digital asset management, and software entitlement enforcement.

---

## 📦 Key Capabilities

- **Subscription Lifecycle**: Auto-calculation of recurring revenue (MRR/ARR), renewal schedules, and prorated billing.
- **Seat & License Allocation**: Enforce concurrent user seat bounds across cloud and on-premise deployments.
- **Asset Integration**: Direct links to hardware infrastructure managed in [`erp-asset`](../erp-asset).

---

## 🛠️ Usage Example

```rust
use erp_software::license::{LicenseKey, SoftwarePlan};

let plan = SoftwarePlan::new("ENTERPRISE_PRO", 50 /* seats */);
let license = LicenseKey::generate(&plan, "TENANT_1001");
```
