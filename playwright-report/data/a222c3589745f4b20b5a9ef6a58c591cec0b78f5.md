# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: e2e_gui_ux.spec.ts >> Phase 2 & 3: Enterprise Template Suites & Manifest Endpoints >> Templates Portal index loads all 6 prebuilt enterprise archetypes
- Location: tests/e2e_gui_ux.spec.ts:115:7

# Error details

```
Error: expect(locator).toHaveCount(expected) failed

Locator:  locator('.template-card')
Expected: 6
Received: 10
Timeout:  5000ms

Call log:
  - Expect "toHaveCount" locator('.template-card') with timeout 5000ms
  - waiting for locator('.template-card')
    14 × locator resolved to 10 elements
       - unexpected value "10"

```

# Page snapshot

```yaml
- generic [ref=e2]:
  - banner [ref=e3]:
    - generic [ref=e4]: Milestone 10.3 • Pure-Rust Enterprise OS
    - heading "Universal Work-Type Templates" [level=1] [ref=e5]
    - paragraph [ref=e6]: Six production-grade commercial and operational website templates compiled natively in Rust with zero foreign runtimes, zero decimal drift, and sub-10ms SSR delivery.
  - generic [ref=e7]:
    - article [ref=e8]:
      - generic [ref=e9]:
        - generic [ref=e10]: Netflix / 30nama Class
        - generic [ref=e11]: SvodMediaStreaming
      - heading "SVoD Video Streaming & Entertainment" [level=3] [ref=e12]
      - paragraph [ref=e13]: Ultra-low-latency 4K HLS streaming, 1536-dim subtitle vector search & watch parties
      - generic [ref=e14]:
        - generic [ref=e15]: erp-cms
        - generic [ref=e16]: erp-software
        - generic [ref=e17]: erp-accounting
        - generic [ref=e18]: erp-support
        - generic [ref=e19]: frappe-storage
      - generic [ref=e20]:
        - generic [ref=e21]: "Aesthetic Archetypes:"
        - generic [ref=e22]:
          - link "Editorial" [ref=e23] [cursor=pointer]:
            - /url: /templates/svod-streaming?variant=awwwards
          - link "Cyberpunk" [ref=e24] [cursor=pointer]:
            - /url: /templates/svod-streaming?variant=cyberpunk
          - link "Vaporwave" [ref=e25] [cursor=pointer]:
            - /url: /templates/svod-streaming?variant=vaporwave
          - link "Retro Wave" [ref=e26] [cursor=pointer]:
            - /url: /templates/svod-streaming?variant=retrowave
          - link "Neon Wave" [ref=e27] [cursor=pointer]:
            - /url: /templates/svod-streaming?variant=neonwave
          - link "Minimal" [ref=e28] [cursor=pointer]:
            - /url: /templates/svod-streaming?variant=tasteful
      - list [ref=e29]:
        - listitem [ref=e30]:
          - generic [ref=e31]: ✓
          - text: HLS Transcoder & HTTP 206 Partial Range Streaming
        - listitem [ref=e32]:
          - generic [ref=e33]: ✓
          - text: 1536-Dimensional HNSW Subtitle Vector Search
        - listitem [ref=e34]:
          - generic [ref=e35]: ✓
          - text: Watch Party WebSockets with Synchronized Playback Locks
        - listitem [ref=e36]:
          - generic [ref=e37]: ✓
          - text: Studio Royalty Ledger Accruals (Double-Entry GL)
        - listitem [ref=e38]:
          - generic [ref=e39]: ✓
          - text: Cryptographically Signed HMAC Streaming Tokens
      - generic [ref=e40]:
        - link "Launch Live Demo →" [ref=e41] [cursor=pointer]:
          - /url: /templates/svod-streaming
        - code [ref=e42]: rbench site deploy -t svod-streaming
    - article [ref=e43]:
      - generic [ref=e44]:
        - generic [ref=e45]: Coursera / Skillshare Class
        - generic [ref=e46]: HigherEducationLMS
      - heading "Digital Learning & Professional LMS Academy" [level=3] [ref=e47]
      - paragraph [ref=e48]: Structured curriculums, ASC 606 tuition amortization & Merkle PDF/A graduation diplomas
      - generic [ref=e49]:
        - generic [ref=e50]: erp-learning
        - generic [ref=e51]: erp-software
        - generic [ref=e52]: erp-accounting
        - generic [ref=e53]: erp-cms
        - generic [ref=e54]: frappe-storage
      - generic [ref=e55]:
        - generic [ref=e56]: "Aesthetic Archetypes:"
        - generic [ref=e57]:
          - link "Editorial" [ref=e58] [cursor=pointer]:
            - /url: /templates/lms-academy?variant=awwwards
          - link "Cyberpunk" [ref=e59] [cursor=pointer]:
            - /url: /templates/lms-academy?variant=cyberpunk
          - link "Vaporwave" [ref=e60] [cursor=pointer]:
            - /url: /templates/lms-academy?variant=vaporwave
          - link "Retro Wave" [ref=e61] [cursor=pointer]:
            - /url: /templates/lms-academy?variant=retrowave
          - link "Neon Wave" [ref=e62] [cursor=pointer]:
            - /url: /templates/lms-academy?variant=neonwave
          - link "Minimal" [ref=e63] [cursor=pointer]:
            - /url: /templates/lms-academy?variant=tasteful
      - list [ref=e64]:
        - listitem [ref=e65]:
          - generic [ref=e66]: ✓
          - text: Course Graph Trees with Granular Lesson State Tracking
        - listitem [ref=e67]:
          - generic [ref=e68]: ✓
          - text: ASC 606 / IFRS 15 5-Step Revenue Amortization
        - listitem [ref=e69]:
          - generic [ref=e70]: ✓
          - text: In-Process Typst PDF/A Graduation Diploma Generator (<5ms)
        - listitem [ref=e71]:
          - generic [ref=e72]: ✓
          - text: SHA-256 Merkle Credential Root Anchoring
        - listitem [ref=e73]:
          - generic [ref=e74]: ✓
          - text: Automated Instructor Royalty Payroll Disbursements
      - generic [ref=e75]:
        - link "Launch Live Demo →" [ref=e76] [cursor=pointer]:
          - /url: /templates/lms-academy
        - code [ref=e77]: rbench site deploy -t lms-academy
    - article [ref=e78]:
      - generic [ref=e79]:
        - generic [ref=e80]: Gumroad / LemonSqueezy Class
        - generic [ref=e81]: DigitalGoodsCreator
      - heading "Digital Products & Creator Hub" [level=3] [ref=e82]
      - paragraph [ref=e83]: Software licensing, single-use signed download links, split creator payouts & EU VAT MOSS
      - generic [ref=e84]:
        - generic [ref=e85]: erp-trade
        - generic [ref=e86]: erp-software
        - generic [ref=e87]: erp-accounting
        - generic [ref=e88]: frappe-storage
        - generic [ref=e89]: frappe-meta
      - generic [ref=e90]:
        - generic [ref=e91]: "Aesthetic Archetypes:"
        - generic [ref=e92]:
          - link "Editorial" [ref=e93] [cursor=pointer]:
            - /url: /templates/digital-goods?variant=awwwards
          - link "Cyberpunk" [ref=e94] [cursor=pointer]:
            - /url: /templates/digital-goods?variant=cyberpunk
          - link "Vaporwave" [ref=e95] [cursor=pointer]:
            - /url: /templates/digital-goods?variant=vaporwave
          - link "Retro Wave" [ref=e96] [cursor=pointer]:
            - /url: /templates/digital-goods?variant=retrowave
          - link "Neon Wave" [ref=e97] [cursor=pointer]:
            - /url: /templates/digital-goods?variant=neonwave
          - link "Minimal" [ref=e98] [cursor=pointer]:
            - /url: /templates/digital-goods?variant=tasteful
      - list [ref=e99]:
        - listitem [ref=e100]:
          - generic [ref=e101]: ✓
          - text: Single-Use AES-256-GCM Encrypted Download Links
        - listitem [ref=e102]:
          - generic [ref=e103]: ✓
          - text: Node-Locked & Floating License Key Activations
        - listitem [ref=e104]:
          - generic [ref=e105]: ✓
          - text: Automated 85/15 Creator Payout Ledger Split Postings
        - listitem [ref=e106]:
          - generic [ref=e107]: ✓
          - text: Real-Time EU VAT MOSS & US State Tax Calculations
        - listitem [ref=e108]:
          - generic [ref=e109]: ✓
          - text: Tiered Partner & Affiliate Commission Tracking
      - generic [ref=e110]:
        - link "Launch Live Demo →" [ref=e111] [cursor=pointer]:
          - /url: /templates/digital-goods
        - code [ref=e112]: rbench site deploy -t digital-goods
    - article [ref=e113]:
      - generic [ref=e114]:
        - generic [ref=e115]: Grainger / Misumi Class
        - generic [ref=e116]: IndustrialWholesaleB2B
      - heading "Industrial B2B & Wholesale Matrix" [level=3] [ref=e117]
      - paragraph [ref=e118]: CAD 3D WebGL assembly viewer, tiered wholesale pricing, corporate Net terms & ZUGFeRD 2.2
      - generic [ref=e119]:
        - generic [ref=e120]: erp-trade
        - generic [ref=e121]: erp-manufacturing
        - generic [ref=e122]: erp-inventory
        - generic [ref=e123]: erp-wms
        - generic [ref=e124]: erp-accounting
      - generic [ref=e125]:
        - generic [ref=e126]: "Aesthetic Archetypes:"
        - generic [ref=e127]:
          - link "Editorial" [ref=e128] [cursor=pointer]:
            - /url: /templates/b2b-industrial?variant=awwwards
          - link "Cyberpunk" [ref=e129] [cursor=pointer]:
            - /url: /templates/b2b-industrial?variant=cyberpunk
          - link "Vaporwave" [ref=e130] [cursor=pointer]:
            - /url: /templates/b2b-industrial?variant=vaporwave
          - link "Retro Wave" [ref=e131] [cursor=pointer]:
            - /url: /templates/b2b-industrial?variant=retrowave
          - link "Neon Wave" [ref=e132] [cursor=pointer]:
            - /url: /templates/b2b-industrial?variant=neonwave
          - link "Minimal" [ref=e133] [cursor=pointer]:
            - /url: /templates/b2b-industrial?variant=tasteful
      - list [ref=e134]:
        - listitem [ref=e135]:
          - generic [ref=e136]: ✓
          - text: Interactive WebGL CAD 3D Component Exploded Viewer
        - listitem [ref=e137]:
          - generic [ref=e138]: ✓
          - text: Multi-Tier Customer Contract Wholesale Price Matrices
        - listitem [ref=e139]:
          - generic [ref=e140]: ✓
          - text: Corporate Credit Limit & OFAC Sanctions Compliance Guard
        - listitem [ref=e141]:
          - generic [ref=e142]: ✓
          - text: Capable-to-Promise (CTP) & SIMD FIFO Lot-Traceable Stock
        - listitem [ref=e143]:
          - generic [ref=e144]: ✓
          - text: Statutory ZUGFeRD 2.2 & Peppol BIS 3.0 E-Invoicing
      - generic [ref=e145]:
        - link "Launch Live Demo →" [ref=e146] [cursor=pointer]:
          - /url: /templates/b2b-industrial
        - code [ref=e147]: rbench site deploy -t b2b-industrial
    - article [ref=e148]:
      - generic [ref=e149]:
        - generic [ref=e150]: Shopify Killer / ASOS Class
        - generic [ref=e151]: HighVelocityRetail
      - heading "Consumer B2C Omnichannel Flagship" [level=3] [ref=e152]
      - paragraph [ref=e153]: 60+ FPS virtualized product grid, real-time flash stock feed, slide-over bag & atomic FIFO checkout
      - generic [ref=e154]:
        - generic [ref=e155]: erp-trade
        - generic [ref=e156]: erp-inventory
        - generic [ref=e157]: erp-wms
        - generic [ref=e158]: erp-accounting
        - generic [ref=e159]: erp-cms
      - generic [ref=e160]:
        - generic [ref=e161]: "Aesthetic Archetypes:"
        - generic [ref=e162]:
          - link "Editorial" [ref=e163] [cursor=pointer]:
            - /url: /templates/b2c-retail?variant=awwwards
          - link "Cyberpunk" [ref=e164] [cursor=pointer]:
            - /url: /templates/b2c-retail?variant=cyberpunk
          - link "Vaporwave" [ref=e165] [cursor=pointer]:
            - /url: /templates/b2c-retail?variant=vaporwave
          - link "Retro Wave" [ref=e166] [cursor=pointer]:
            - /url: /templates/b2c-retail?variant=retrowave
          - link "Neon Wave" [ref=e167] [cursor=pointer]:
            - /url: /templates/b2c-retail?variant=neonwave
          - link "Minimal" [ref=e168] [cursor=pointer]:
            - /url: /templates/b2c-retail?variant=tasteful
      - list [ref=e169]:
        - listitem [ref=e170]:
          - generic [ref=e171]: ✓
          - text: 60+ FPS Virtualized Viewport SKU Grid (<10ms TTFB)
        - listitem [ref=e172]:
          - generic [ref=e173]: ✓
          - text: SurrealDB LIVE SELECT Real-Time Flash Stock Feed
        - listitem [ref=e174]:
          - generic [ref=e175]: ✓
          - text: Atomic Checkout Engine with Balanced GL & Stock Dedup
        - listitem [ref=e176]:
          - generic [ref=e177]: ✓
          - text: Lin-Kernighan TSP Pick Wave Routing in Warehouse
        - listitem [ref=e178]:
          - generic [ref=e179]: ✓
          - text: Personalized Buying Center & Customer Loyalty Rewards
      - generic [ref=e180]:
        - link "Launch Live Demo →" [ref=e181] [cursor=pointer]:
          - /url: /templates/b2c-retail
        - code [ref=e182]: rbench site deploy -t b2c-retail
    - article [ref=e183]:
      - generic [ref=e184]:
        - generic [ref=e185]: Robinhood / TradingView Class
        - generic [ref=e186]: TradingExchange
      - heading "Financial Trading, Exchange & Brokerage Hub" [level=3] [ref=e187]
      - paragraph [ref=e188]: WebGL Canvas candlestick charts, streaming Level-2 depth ladder, sanctions KYC & multi-currency wallets
      - generic [ref=e189]:
        - generic [ref=e190]: erp-trade
        - generic [ref=e191]: erp-accounting
        - generic [ref=e192]: erp-crm
        - generic [ref=e193]: frappe-net
        - generic [ref=e194]: frappe-storage
      - generic [ref=e195]:
        - generic [ref=e196]: "Aesthetic Archetypes:"
        - generic [ref=e197]:
          - link "Editorial" [ref=e198] [cursor=pointer]:
            - /url: /templates/trading-exchange?variant=awwwards
          - link "Cyberpunk" [ref=e199] [cursor=pointer]:
            - /url: /templates/trading-exchange?variant=cyberpunk
          - link "Vaporwave" [ref=e200] [cursor=pointer]:
            - /url: /templates/trading-exchange?variant=vaporwave
          - link "Retro Wave" [ref=e201] [cursor=pointer]:
            - /url: /templates/trading-exchange?variant=retrowave
          - link "Neon Wave" [ref=e202] [cursor=pointer]:
            - /url: /templates/trading-exchange?variant=neonwave
          - link "Minimal" [ref=e203] [cursor=pointer]:
            - /url: /templates/trading-exchange?variant=tasteful
      - list [ref=e204]:
        - listitem [ref=e205]:
          - generic [ref=e206]: ✓
          - text: WebGL Canvas Candlestick & Volume Indicator Charting
        - listitem [ref=e207]:
          - generic [ref=e208]: ✓
          - text: Microsecond Tick Streaming over UDP / WebSockets
        - listitem [ref=e209]:
          - generic [ref=e210]: ✓
          - text: Streaming Level-2 Order Book Depth Ladder with Bid/Ask
        - listitem [ref=e211]:
          - generic [ref=e212]: ✓
          - text: 128-bit rust_decimal Wallet Ledgers with Zero Drift
        - listitem [ref=e213]:
          - generic [ref=e214]: ✓
          - text: Bitemporal Order Execution Records & Merkle Audit Blocks
      - generic [ref=e215]:
        - link "Launch Live Demo →" [ref=e216] [cursor=pointer]:
          - /url: /templates/trading-exchange
        - code [ref=e217]: rbench site deploy -t trading-exchange
    - article [ref=e218]:
      - generic [ref=e219]:
        - generic [ref=e220]: Linear / Vercel Class
        - generic [ref=e221]: DeveloperSaaS
      - heading "SaaS, AI & Cloud Platforms" [level=3] [ref=e222]
      - paragraph [ref=e223]: API telemetry, rate limiting analytics, SDK generators & self-serve team billing
      - generic [ref=e224]:
        - generic [ref=e225]: erp-software
        - generic [ref=e226]: erp-accounting
        - generic [ref=e227]: erp-support
        - generic [ref=e228]: frappe-net
        - generic [ref=e229]: frappe-framework
      - generic [ref=e230]:
        - generic [ref=e231]: "Aesthetic Archetypes:"
        - generic [ref=e232]:
          - link "Editorial" [ref=e233] [cursor=pointer]:
            - /url: /templates/saas-cloud?variant=awwwards
          - link "Cyberpunk" [ref=e234] [cursor=pointer]:
            - /url: /templates/saas-cloud?variant=cyberpunk
          - link "Vaporwave" [ref=e235] [cursor=pointer]:
            - /url: /templates/saas-cloud?variant=vaporwave
          - link "Retro Wave" [ref=e236] [cursor=pointer]:
            - /url: /templates/saas-cloud?variant=retrowave
          - link "Neon Wave" [ref=e237] [cursor=pointer]:
            - /url: /templates/saas-cloud?variant=neonwave
          - link "Minimal" [ref=e238] [cursor=pointer]:
            - /url: /templates/saas-cloud?variant=tasteful
      - list [ref=e239]:
        - listitem [ref=e240]:
          - generic [ref=e241]: ✓
          - text: Real-Time API Consumption Metering & QPS Gauges
        - listitem [ref=e242]:
          - generic [ref=e243]: ✓
          - text: Self-Service Team Seat Provisioning & SAML/SSO
        - listitem [ref=e244]:
          - generic [ref=e245]: ✓
          - text: Multi-Tier Subscription Invoicing (ASC 606)
        - listitem [ref=e246]:
          - generic [ref=e247]: ✓
          - text: WASM Edge Function Sandboxed Execution Logs
        - listitem [ref=e248]:
          - generic [ref=e249]: ✓
          - text: Automated Developer Portal & OpenAPI V3 Documentation
      - generic [ref=e250]:
        - link "Launch Live Demo →" [ref=e251] [cursor=pointer]:
          - /url: /templates/saas-cloud
        - code [ref=e252]: rbench site deploy -t saas-cloud
    - article [ref=e253]:
      - generic [ref=e254]:
        - generic [ref=e255]: Epic / OneMedical Class
        - generic [ref=e256]: HealthcareClinical
      - heading "Professional Healthcare & Diagnostics" [level=3] [ref=e257]
      - paragraph [ref=e258]: HIPAA-compliant EHR, real-time appointment triage, DICOM imaging links & HL7/FHIR
      - generic [ref=e259]:
        - generic [ref=e260]: erp-support
        - generic [ref=e261]: erp-crm
        - generic [ref=e262]: erp-accounting
        - generic [ref=e263]: frappe-storage
        - generic [ref=e264]: frappe-meta
      - generic [ref=e265]:
        - generic [ref=e266]: "Aesthetic Archetypes:"
        - generic [ref=e267]:
          - link "Editorial" [ref=e268] [cursor=pointer]:
            - /url: /templates/healthcare-clinical?variant=awwwards
          - link "Cyberpunk" [ref=e269] [cursor=pointer]:
            - /url: /templates/healthcare-clinical?variant=cyberpunk
          - link "Vaporwave" [ref=e270] [cursor=pointer]:
            - /url: /templates/healthcare-clinical?variant=vaporwave
          - link "Retro Wave" [ref=e271] [cursor=pointer]:
            - /url: /templates/healthcare-clinical?variant=retrowave
          - link "Neon Wave" [ref=e272] [cursor=pointer]:
            - /url: /templates/healthcare-clinical?variant=neonwave
          - link "Minimal" [ref=e273] [cursor=pointer]:
            - /url: /templates/healthcare-clinical?variant=tasteful
      - list [ref=e274]:
        - listitem [ref=e275]:
          - generic [ref=e276]: ✓
          - text: HL7 / FHIR R4 Interoperable Diagnostic Records
        - listitem [ref=e277]:
          - generic [ref=e278]: ✓
          - text: Zero-Knowledge Encrypted Clinical Note Storage
        - listitem [ref=e279]:
          - generic [ref=e280]: ✓
          - text: Doctor On-Call Real-Time Rota & Shift Matrix
        - listitem [ref=e281]:
          - generic [ref=e282]: ✓
          - text: Automated Insurance Pre-Authorization Claims
        - listitem [ref=e283]:
          - generic [ref=e284]: ✓
          - text: Biometric Patient Identification & Telehealth Portal
      - generic [ref=e285]:
        - link "Launch Live Demo →" [ref=e286] [cursor=pointer]:
          - /url: /templates/healthcare-clinical
        - code [ref=e287]: rbench site deploy -t healthcare-clinical
    - article [ref=e288]:
      - generic [ref=e289]:
        - generic [ref=e290]: Zillow / Sotheby's Realty Class
        - generic [ref=e291]: RealEstateSpatial
      - heading "Real Estate & Spatial Property Development" [level=3] [ref=e292]
      - paragraph [ref=e293]: 3D architectural walkthroughs, bitemporal lease contracts, escrow ledgers & IoT telemetry
      - generic [ref=e294]:
        - generic [ref=e295]: erp-asset
        - generic [ref=e296]: erp-accounting
        - generic [ref=e297]: erp-crm
        - generic [ref=e298]: erp-trade
        - generic [ref=e299]: frappe-storage
      - generic [ref=e300]:
        - generic [ref=e301]: "Aesthetic Archetypes:"
        - generic [ref=e302]:
          - link "Editorial" [ref=e303] [cursor=pointer]:
            - /url: /templates/real-estate-spatial?variant=awwwards
          - link "Cyberpunk" [ref=e304] [cursor=pointer]:
            - /url: /templates/real-estate-spatial?variant=cyberpunk
          - link "Vaporwave" [ref=e305] [cursor=pointer]:
            - /url: /templates/real-estate-spatial?variant=vaporwave
          - link "Retro Wave" [ref=e306] [cursor=pointer]:
            - /url: /templates/real-estate-spatial?variant=retrowave
          - link "Neon Wave" [ref=e307] [cursor=pointer]:
            - /url: /templates/real-estate-spatial?variant=neonwave
          - link "Minimal" [ref=e308] [cursor=pointer]:
            - /url: /templates/real-estate-spatial?variant=tasteful
      - list [ref=e309]:
        - listitem [ref=e310]:
          - generic [ref=e311]: ✓
          - text: Interactive 3D Spatial Walkthroughs & Floorplans
        - listitem [ref=e312]:
          - generic [ref=e313]: ✓
          - text: Bitemporal Commercial Lease Escrow Management
        - listitem [ref=e314]:
          - generic [ref=e315]: ✓
          - text: Automated CAM (Common Area Maintenance) Split Runs
        - listitem [ref=e316]:
          - generic [ref=e317]: ✓
          - text: IoT Smart Meter Telemetry & Utility Ledger Billing
        - listitem [ref=e318]:
          - generic [ref=e319]: ✓
          - text: Investor Capital Call & Dividend Yield Distribution
      - generic [ref=e320]:
        - link "Launch Live Demo →" [ref=e321] [cursor=pointer]:
          - /url: /templates/real-estate-spatial
        - code [ref=e322]: rbench site deploy -t real-estate-spatial
    - article [ref=e323]:
      - generic [ref=e324]:
        - generic [ref=e325]: Toast / Resy Class
        - generic [ref=e326]: GastronomyKitchen
      - heading "Haute Cuisine, Dining & Hospitality" [level=3] [ref=e327]
      - paragraph [ref=e328]: Real-time kitchen display system (KDS), table reservation locking, recipe yield costing & multi-outlet inventory
      - generic [ref=e329]:
        - generic [ref=e330]: erp-inventory
        - generic [ref=e331]: erp-trade
        - generic [ref=e332]: erp-accounting
        - generic [ref=e333]: desk-components
        - generic [ref=e334]: frappe-net
      - generic [ref=e335]:
        - generic [ref=e336]: "Aesthetic Archetypes:"
        - generic [ref=e337]:
          - link "Editorial" [ref=e338] [cursor=pointer]:
            - /url: /templates/gastronomy-hospitality?variant=awwwards
          - link "Cyberpunk" [ref=e339] [cursor=pointer]:
            - /url: /templates/gastronomy-hospitality?variant=cyberpunk
          - link "Vaporwave" [ref=e340] [cursor=pointer]:
            - /url: /templates/gastronomy-hospitality?variant=vaporwave
          - link "Retro Wave" [ref=e341] [cursor=pointer]:
            - /url: /templates/gastronomy-hospitality?variant=retrowave
          - link "Neon Wave" [ref=e342] [cursor=pointer]:
            - /url: /templates/gastronomy-hospitality?variant=neonwave
          - link "Minimal" [ref=e343] [cursor=pointer]:
            - /url: /templates/gastronomy-hospitality?variant=tasteful
      - list [ref=e344]:
        - listitem [ref=e345]:
          - generic [ref=e346]: ✓
          - text: Zero-Latency Kitchen Display System (KDS) WebSockets
        - listitem [ref=e347]:
          - generic [ref=e348]: ✓
          - text: Microsecond Table Reservation Distributed Locking
        - listitem [ref=e349]:
          - generic [ref=e350]: ✓
          - text: Recipe Ingredient Yield & Wastage Accruals
        - listitem [ref=e351]:
          - generic [ref=e352]: ✓
          - text: Multi-Outlet Central Commissary Stock Transfers
        - listitem [ref=e353]:
          - generic [ref=e354]: ✓
          - text: Contactless QR Order & Apple Pay / Google Pay Flow
      - generic [ref=e355]:
        - link "Launch Live Demo →" [ref=e356] [cursor=pointer]:
          - /url: /templates/gastronomy-hospitality
        - code [ref=e357]: rbench site deploy -t gastronomy-hospitality
```

# Test source

```ts
  25  |     await expect(cursor).toBeAttached();
  26  |     await expect(cursorDot).toBeAttached();
  27  |   });
  28  | 
  29  |   test('3D Material customizer updates shader and finish reactively', async ({ page }) => {
  30  |     await page.goto(`${BASE_URL}/storefront`);
  31  | 
  32  |     // Open 3D Configurator
  33  |     const openConfigBtn = page.getByRole('button', { name: /Launch 3D Configurator/i });
  34  |     await openConfigBtn.click();
  35  | 
  36  |     // Verify Configurator Modal is displayed
  37  |     const modalOverlay = page.locator('#configurator-modal-overlay');
  38  |     await expect(modalOverlay).toHaveClass(/open/);
  39  | 
  40  |     // Verify initial base price
  41  |     const priceLabel = page.locator('#config-item-price');
  42  |     await expect(priceLabel).toHaveText('$14,500.00');
  43  | 
  44  |     // Click on Aurum / Gold metallurgy swatch
  45  |     const goldButton = page.locator('button.swatch-btn[data-finish="gold"]');
  46  |     await goldButton.click();
  47  | 
  48  |     // Verify selected finish text updates
  49  |     const finishLabel = page.locator('#selected-finish-name');
  50  |     await expect(finishLabel).toHaveText(/Aurum/i);
  51  | 
  52  |     // Click on Carbon swatch
  53  |     const carbonButton = page.locator('button.swatch-btn[data-finish="carbon"]');
  54  |     await carbonButton.click();
  55  |     await expect(finishLabel).toHaveText(/Carbon/i);
  56  | 
  57  |     // Add configured item to vault cart
  58  |     const addToVaultBtn = page.getByRole('button', { name: /Add Configured Artifact to Vault/i });
  59  |     await addToVaultBtn.click();
  60  | 
  61  |     // Verify Cart Drawer opens with item
  62  |     const drawerBackdrop = page.locator('#cart-drawer-backdrop');
  63  |     await expect(drawerBackdrop).toHaveClass(/open/);
  64  |     const cartBadge = page.locator('.cart-badge');
  65  |     await expect(cartBadge).toBeVisible();
  66  |   });
  67  | 
  68  |   test('Interactive slide-over cart drawer and ACID checkout execution', async ({ page }) => {
  69  |     await page.goto(`${BASE_URL}/storefront`);
  70  | 
  71  |     // Open Cart Drawer directly from nav
  72  |     const cartBtn = page.locator('.cart-btn');
  73  |     await cartBtn.click();
  74  | 
  75  |     // Cart drawer should open
  76  |     const drawerBackdrop = page.locator('#cart-drawer-backdrop');
  77  |     await expect(drawerBackdrop).toHaveClass(/open/);
  78  | 
  79  |     // Proceed to Atomic Checkout
  80  |     const checkoutBtn = page.locator('button.checkout-btn');
  81  |     await checkoutBtn.click();
  82  | 
  83  |     // Checkout modal must appear
  84  |     const checkoutOverlay = page.locator('#checkout-modal-overlay');
  85  |     await expect(checkoutOverlay).toHaveClass(/open/);
  86  | 
  87  |     // Submit ACID checkout form
  88  |     const submitBtn = checkoutOverlay.locator('button[type="submit"]');
  89  |     await submitBtn.click();
  90  | 
  91  |     // Verification screen must appear with invoice ID
  92  |     const successBox = page.locator('#checkout-success');
  93  |     await expect(successBox).toBeVisible({ timeout: 10000 });
  94  |     const invoiceId = page.locator('#success-invoice-id');
  95  |     await expect(invoiceId).toContainText(/ACC-SINV-|INV-COMMERCE-/i);
  96  |   });
  97  | 
  98  |   test('Accessibility (a11y) audit meets WCAG 2.1 AA', async ({ page }) => {
  99  |     await page.goto(`${BASE_URL}/storefront`);
  100 | 
  101 |     // Run axe accessibility analysis
  102 |     const accessibilityScanResults = await new AxeBuilder({ page })
  103 |       .withTags(['wcag2a', 'wcag2aa'])
  104 |       .analyze();
  105 | 
  106 |     // Assert zero critical violations
  107 |     const criticalViolations = accessibilityScanResults.violations.filter(v => v.impact === 'critical');
  108 |     expect(criticalViolations).toEqual([]);
  109 |   });
  110 | 
  111 | });
  112 | 
  113 | test.describe('Phase 2 & 3: Enterprise Template Suites & Manifest Endpoints', () => {
  114 | 
  115 |   test('Templates Portal index loads all 6 prebuilt enterprise archetypes', async ({ page }) => {
  116 |     await page.goto(`${BASE_URL}/templates`);
  117 | 
  118 |     // Verify Portal Title and Heading
  119 |     await expect(page).toHaveTitle(/Universal Enterprise Template Suites/i);
  120 |     const heading = page.locator('h1');
  121 |     await expect(heading).toContainText('Universal Work-Type Templates');
  122 | 
  123 |     // Verify all 6 template suite cards exist
  124 |     const cards = page.locator('.template-card');
> 125 |     await expect(cards).toHaveCount(6);
      |                         ^ Error: expect(locator).toHaveCount(expected) failed
  126 | 
  127 |     // Verify specific slugs exist in links
  128 |     await expect(page.locator('a[href="/templates/svod-streaming"]')).toBeVisible();
  129 |     await expect(page.locator('a[href="/templates/lms-academy"]')).toBeVisible();
  130 |     await expect(page.locator('a[href="/templates/digital-goods"]')).toBeVisible();
  131 |     await expect(page.locator('a[href="/templates/b2b-industrial"]')).toBeVisible();
  132 |     await expect(page.locator('a[href="/templates/b2c-retail"]')).toBeVisible();
  133 |     await expect(page.locator('a[href="/templates/trading-exchange"]')).toBeVisible();
  134 |   });
  135 | 
  136 |   const templateSuites = [
  137 |     { slug: 'svod-streaming', titlePattern: /Cinema/i, contentSelector: '.hero-title', expectedContent: 'Chronos Horizon' },
  138 |     { slug: 'lms-academy', titlePattern: /Academy/i, contentSelector: 'h1', expectedContent: 'Lock-Free Shared Resources' },
  139 |     { slug: 'digital-goods', titlePattern: /Creator Hub/i, contentSelector: 'h1', expectedContent: 'Developer Tools' },
  140 |     { slug: 'b2b-industrial', titlePattern: /Industrial/i, contentSelector: '.top-bar', expectedContent: 'Siemens AG Procurement' },
  141 |     { slug: 'b2c-retail', titlePattern: /Retail/i, contentSelector: '.brand', expectedContent: 'Retail' },
  142 |     { slug: 'trading-exchange', titlePattern: /Exchange/i, contentSelector: '.ticker', expectedContent: 'BTC-USD' },
  143 |   ];
  144 | 
  145 |   for (const { slug, titlePattern, contentSelector, expectedContent } of templateSuites) {
  146 |     test(`Template Suite [${slug}] renders with design tokens & live HUD`, async ({ page }) => {
  147 |       await page.goto(`${BASE_URL}/templates/${slug}`);
  148 | 
  149 |       // Verify page title
  150 |       await expect(page).toHaveTitle(titlePattern);
  151 | 
  152 |       // Verify content element
  153 |       const contentEl = page.locator(contentSelector);
  154 |       await expect(contentEl).toContainText(new RegExp(expectedContent, 'i'));
  155 | 
  156 |       // Check JSON manifest endpoint
  157 |       const manifestRes = await page.request.get(`${BASE_URL}/api/v1/templates/${slug}/manifest`);
  158 |       expect(manifestRes.status()).toBe(200);
  159 |       const manifest = await manifestRes.json();
  160 |       expect(manifest.id).toBe(`theme_${slug}`);
  161 |       expect(manifest.default_design_tokens).toBeDefined();
  162 |     });
  163 |   }
  164 | 
  165 | });
  166 | 
  167 | test.describe('Phase 4 & 5: Health, APIs & Offline Resilience Quality Gates', () => {
  168 | 
  169 |   test('System health check returns valid JSON status', async ({ request }) => {
  170 |     const res = await request.get(`${BASE_URL}/health`);
  171 |     expect(res.status()).toBe(200);
  172 |     const body = await res.json();
  173 |     expect(body.status).toBe('healthy');
  174 |     expect(body.version).toBeDefined();
  175 |   });
  176 | 
  177 |   test('Templates API returns all 6 registered suites', async ({ request }) => {
  178 |     const res = await request.get(`${BASE_URL}/api/v1/templates`);
  179 |     expect(res.status()).toBe(200);
  180 |     const templates = await res.json();
  181 |     expect(Array.isArray(templates)).toBe(true);
  182 |     expect(templates.length).toBe(6);
  183 |   });
  184 | 
  185 |   test('Storefront products API returns complete luxury catalog', async ({ request }) => {
  186 |     const res = await request.get(`${BASE_URL}/api/v1/storefront/products`);
  187 |     expect(res.status()).toBe(200);
  188 |     const products = await res.json();
  189 |     expect(Array.isArray(products)).toBe(true);
  190 |     expect(products.length).toBeGreaterThanOrEqual(6);
  191 |     expect(products[0]).toHaveProperty('id');
  192 |     expect(products[0]).toHaveProperty('price_usd');
  193 |   });
  194 | 
  195 |   test('Atomic Checkout API executes ACID commit successfully', async ({ request }) => {
  196 |     const res = await request.post(`${BASE_URL}/api/v1/storefront/checkout`, {
  197 |       data: {
  198 |         checkout_id: 'CHK-TEST-001',
  199 |         customer_id: 'CUST-001',
  200 |         items: [
  201 |           {
  202 |             item_code: 'CHRONOS-01',
  203 |             description: 'Chronos-01 Kinetic Tourbillon',
  204 |             qty: '1',
  205 |             unit_price: '14500.00',
  206 |             available_stock: '10',
  207 |           },
  208 |         ],
  209 |         tax_rate_percent: '8.0',
  210 |         receivable_account: '1100 - Accounts Receivable',
  211 |         revenue_account: '4100 - Luxury Product Sales',
  212 |         tax_account: '2200 - Sales Tax Payable',
  213 |       },
  214 |     });
  215 | 
  216 |     expect(res.status()).toBe(200);
  217 |     const result = await res.json();
  218 |     expect(result.is_success).toBe(true);
  219 |     expect(result.invoice_id).toContain('INV-COMMERCE-');
  220 |     expect(result.grand_total).toBeDefined();
  221 |     expect(result.gl_postings.length).toBeGreaterThan(0);
  222 |   });
  223 | 
  224 | });
  225 | 
```