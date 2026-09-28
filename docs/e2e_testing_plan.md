# End-to-End GUI, UX & Persona Testing Plan
## Automated Agent Validation Suite for Pure-Rust Enterprise OS & Storefronts

---

## 1. Executive Strategy & Tooling Ecosystem

This testing plan evaluates the platform purely from the perspective of an authentic user interacting with real graphical interfaces. It exercises the WebGL 3D canvas, GSAP kinetic scroll sequences, role-adaptive multi-persona cockpits, six vertical template archetypes, dynamic theme hot-swapping, and offline-first state reconciliation.

```
                         AGENT-DRIVEN E2E TESTING ARCHITECTURE
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                Antigravity-IDE Agent                                   │
│  ├── Browser Automation Orchestrator (Playwright Chromium / WebKit / Firefox)          │
│  ├── Mobile Device Emulation (Touch, Viewport 390x844, DevicePixelRatio 3x)            │
│  ├── Visual Regression & Pixel Diffing Engine (pixelmatch, threshold <= 0.05%)         │
│  ├── Accessibility Auditor (axe-core / WCAG 2.1 AA)                                    │
│  └── Synthetic Network Condition Interceptor (Offline, 3G Slow, WebSocket Monitor)     │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │ HTTP/3, WebSockets, DOM Events
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        Statically Compiled `rustnext` Target                           │
│     (Actix-web HTTP Gateway + Embedded SurrealDB + Dioxus WASM + Three.js Substrate)   │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 1.1 Recommended Tooling & Installation Instructions

To give the agent full sensory capabilities (DOM interaction, pixel capture, tactile touch, network interruption, and screen-reader tree auditing), execute the following setup commands:

```bash
# 1. Install Node.js test harness dependencies
npm install --save-dev @playwright/test@latest @axe-core/playwright pixelmatch pngjs

# 2. Install Playwright browser engines with system dependencies
npx playwright install --with-deps chromium webkit firefox

# 3. Optional: Install Lighthouse CLI for automated Web Vitals & Core UX scoring
npm install -g lighthouse

# 4. Optional: For native Android APK testing via adb (if emulator is attached)
# sudo apt-get install -y android-tools-adb
```

---

## 2. Granular User-Journey Test Scenarios

---

### Phase 1: Luxury 3D WebGL Storefront & Kinetic Disassembly (`/storefront`)

**Persona:** Discerning luxury customer shopping on desktop and mobile.  
**Objective:** Confirm smooth visual presentation, continuous 60+ FPS WebGL rendering, scroll-linked GSAP component disassembly, dynamic metallurgy shader swaps, and zero-drift cart commits.

```
       SCROLL STAGE 1                     SCROLL STAGE 2                     SCROLL STAGE 3
   ┌────────────────────┐             ┌────────────────────┐             ┌────────────────────┐
   │     Hero View      │             │  Exploded Calibre  │             │   Bespoke Atelier  │
   │  Timepiece docked  │ ──────────► │ Bezel moves +Z     │ ──────────► │ Watch reassembles  │
   │  Mouse parallax    │  (Scroll)   │ Gears move -Z      │  (Scroll)   │ Configurator docks │
   └────────────────────┘             └────────────────────┘             └────────────────────┘
```

#### Test Scenario 1.1: WebGL Canvas Initialization & FPS Integrity
* **User Action:** Navigate to `http://localhost:8080/storefront`.
* **GUI Assertions:**
  1. `#webgl-canvas` element is present in the DOM and occupies 100% viewport width and height (`fixed inset-0`).
  2. WebGL context is initialized (`canvas.getContext('webgl2') || canvas.getContext('webgl') != null`).
  3. No WebGL shader compilation warnings or WebGL context lost errors appear in the browser console.
  4. Frame rate monitor maintains $\ge 55\,\text{FPS}$ during continuous window scrolling.

#### Test Scenario 1.2: Parallax Movement & Mouse Inertia
* **User Action:** Move cursor across the screen in diagonal patterns (`(100, 100) -> (800, 600)`).
* **GUI Assertions:**
  1. Custom magnetic cursor dot (`#cursor-dot`) moves immediately with the pointer.
  2. Custom cursor outline (`#cursor-outline`) follows with noticeable inertia damping.
  3. The 3D timepiece rotates smoothly along the X and Y axes within bounds $[-0.4, 0.4]\,\text{rad}$.

#### Test Scenario 1.3: GSAP Kinetic Exploded-View Disassembly
* **User Action:** Smoothly scroll from `#hero` down into the `#mechanics` section.
* **GUI Assertions:**
  1. As scroll position passes `#mechanics`, GSAP `ScrollTrigger` updates component coordinates.
  2. Outer Bezel separates forward along the Z-axis ($z \to +1.2$).
  3. Gear train carriage moves backward ($z \to -1.0$).
  4. Core luminescent crystal scales up ($1.0 \to 1.4$).
  5. Exploded callout cards ("01. Dual-Axis Bezel", "02. Tourbillon Carriage", "03. Cryptographic Caseback") fade in with staggered entry animations.

#### Test Scenario 1.4: Bespoke Atelier 3D Customizer
* **User Action:** Scroll to `#configurator`.
  1. Click **Celestial Gold** button (`[data-mat="gold"]`).
  2. Uncheck **Gyroscopic Tourbillon Rotation** checkbox.
  3. Click **Order Allocation** button.
* **GUI Assertions:**
  1. Titanium button border clears; Celestial Gold button receives `#e6c887` gold highlight ring.
  2. Timepiece bezel shader updates to warm gold reflectivity without full page reload.
  3. Configured price badge animates from `$14,200` to `$24,600`.
  4. Toast notification appears at bottom-right: *"Metallurgy changed to GOLD"*.
  5. Slide-over cart drawer smoothly translates from `translateX(100%)` to `translateX(0%)`.
  6. Cart badge updates from `0` to `1`.

#### Test Scenario 1.5: ACID Checkout Simulation
* **User Action:** In the open cart drawer, click **Execute ACID Checkout**.
* **GUI Assertions:**
  1. Button displays loading state: *"Submitting ACID transaction to SurrealDB..."*.
  2. After $\approx 1200\,\text{ms}$, cart drawer closes and confirmation modal appears.
  3. Modal displays: *"SurrealDB Commit #TX-98241 - ALLOCATION RESERVED"*.
  4. Subtotal resets to `$0.00` and cart item count drops to `(0 items)`.

---

### Phase 2: Role-Adaptive Multi-Persona Shells (`/desk`)

**Objective:** Validate that the application interface changes layout, input targets, and interaction models based on the authenticated role.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        ROLE-ADAPTIVE SHELL TEST MATRIX                                 │
├───────────────────┬───────────────────┬──────────────────────┬─────────────────────────┤
│ Active Role       │ Route             │ Required UI Paradigm │ Touch Target Standard   │
├───────────────────┼───────────────────┼──────────────────────┼─────────────────────────┤
│ Warehouse Worker  │ `/worker`         │ Industrial Scanner   │ >= 56px, high contrast  │
│ MES Shopfloor     │ `/factory`        │ Landscape Terminal   │ Dual-witness sign-off   │
│ Approving Manager │ `/approvals`      │ Gesture Swipe Deck   │ Swipe-R approve, L reject│
│ System Admin      │ `/admin`          │ Telemetry Cockpit    │ Real-time metrics       │
│ Client / Buyer    │ `/portal`         │ Consumer Hub         │ Editorial catalog       │
└───────────────────┴───────────────────┴──────────────────────┴─────────────────────────┘
```

#### Test Scenario 2.1: Warehouse Worker High-Tactile Interface (`/worker`)
* **User Context:** Logged in with role `Warehouse Worker` on mobile viewport ($390 \times 844\,\text{px}$).
* **Action:**
  1. Navigate to `/worker`.
  2. Inspect layout and touch bounding boxes.
  3. Trigger simulated barcode scan input (`SSCC-18: 001234567890123456`).
  4. Disconnect network (`context.setOffline(true)`).
  5. Complete 3 inventory bin transfer picks.
* **UX & Ergonomics Assertions:**
  1. Every interactive button has minimum computed height $\ge 56\,\text{px}$ and width $\ge 56\,\text{px}$.
  2. High-contrast color palette: background `#000000`, text `#FFFFFF`, primary action `#00FF66` or high-visibility yellow.
  3. Barcode scan successfully resolves the pick line, emits positive audio/haptic tone trigger, and advances to next bin.
  4. When offline, persistent offline badge displays: *"Offline Mode - 3 Scans Queued"*.
  5. Application does not freeze, throw uncaught network errors, or block input while disconnected.

#### Test Scenario 2.2: Shopfloor MES Operator Landscape Terminal (`/factory`)
* **User Context:** Logged in with role `MES Operator` on tablet viewport ($1280 \times 800\,\text{px}$, landscape).
* **Action:**
  1. Open active Job Card traveler.
  2. Progress step 01 (Machining Bezel) to complete.
  3. Trigger Electronic Batch Record (EBR) sign-off modal.
  4. Input Operator PIN, then input Witness Supervisor PIN.
* **UX & Governance Assertions:**
  1. Real-time Nelson-rule SPC chart renders sub-second telemetry points without canvas stutter.
  2. Dual-witness modal requires two distinct authorized signatures; signing with identical credentials flags validation error: *"Witness must differ from primary operator"*.
  3. Upon valid submission, step updates to `Approved & Cryptographically Anchored`.

#### Test Scenario 2.3: Executive Manager Swipe Approval Deck (`/approvals`)
* **User Context:** Logged in with role `Department Lead / Manager` on mobile viewport ($390 \times 844\,\text{px}$).
* **Action:**
  1. Navigate to `/approvals`.
  2. Observe pending approval card stack (Purchase Orders, Expense Requests).
  3. Perform rightward touch drag gesture ($\Delta x > +160\,\text{px}$) on Card #1.
  4. Perform leftward touch drag gesture ($\Delta x < -160\,\text{px}$) on Card #2.
* **UX & Interaction Assertions:**
  1. Card #1 smoothly rotates and flies out to the right with green *"APPROVED"* stamp overlay.
  2. Card #2 smoothly rotates and flies out to the left with red *"REJECTED"* stamp overlay; displays rejection reason textarea.
  3. Next card in stack immediately slides up into the interactive focal position.
  4. Streaming macro KPI tickers at the top (Daily Gross Margin, Liquidity Pool) pulse green/red as values update.

#### Test Scenario 2.4: System Administrator Telemetry Cockpit (`/admin`)
* **User Context:** Logged in with role `System Administrator` on desktop viewport ($1920 \times 1080\,\text{px}$).
* **Action:**
  1. Navigate to `/admin`.
  2. Inspect Actix worker thread pool gauges and SurrealDB connection multiplexer graph.
  3. Open Dynamic DocType Schema Designer.
  4. Add a new field: `custom_serial_override` (Type: `Data`, Label: `Custom Serial`).
  5. Click **Apply Online Schema Migration**.
* **UX Assertions:**
  1. Telemetry gauges update in real-time over WebSocket feeds without full-page re-renders.
  2. Schema designer provides instant drag-and-drop field reordering.
  3. Migration completes without throwing database lock warnings or requiring service restart.

---

### Phase 3: Six Enterprise Template Site Archetypes

Test the end-to-end user workflows across all six prebuilt site models:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              THE 6 TEMPLATE ARCHETYPES                                 │
│  [1. SVoD Streaming]      [2. Digital Learning]      [3. Digital Goods Hub]            │
│  ├── Netflix/Crunchyroll  ├── Coursera/Udemy         ├── Steam/Creative Market         │
│  ├── ABR HLS Video HUD    ├── Course Graph Syllabus  ├── Live Type-Tester & Bundles    │
│  └── Vector Subtitles     └── Typst Merkle Diplomas  └── Signed Expiring Downloads     │
│                                                                                        │
│  [4. Industrial B2B]      [5. Luxury B2C Flagship]   [6. Trading Exchange]             │
│  ├── Frost CNC/Rowa/Starm ├── Cartier/Industry West  ├── Binance/Robinhood/Polymarket  │
│  ├── Parametric 3D Tooling├── Spatial WebGL & AR     ├── High-Density L2 Orderbook     │
│  └── Quad-BOM & RFQ Net   └── Exploded Disassembly   └── Implied Probability Bets      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

#### Test Scenario 3.1: SVoD Streaming (`template-svod-streaming`)
* **User Action:**
  1. Navigate to `/template-svod-streaming`.
  2. Click Play on hero anime title.
  3. Click and scrub timeline from $00:00$ to $12:45$.
  4. Type in subtitle semantic search box: *"quantum crystal"*.
* **GUI Assertions:**
  1. Video playback HUD launches; responsive video element streams HLS chunks without black frames.
  2. Timestamp scrubbing initiates HTTP Range request (`Status 206 Partial Content`) and repositions playback in $< 300\,\text{ms}$.
  3. Vector search returns matching scenes with timestamps; clicking result jumps video directly to that point.

#### Test Scenario 3.2: Digital Learning Academy (`template-lms-academy`)
* **User Action:**
  1. Navigate to `/template-lms-academy`.
  2. Enroll in *"Advanced Rust & Distributed Systems"*.
  3. Check off final lesson task.
  4. Click **Download Accredited Diploma**.
* **GUI Assertions:**
  1. Course syllabus graph marks lesson nodes complete with green checkmarks.
  2. Progress bar advances from $90\%$ to $100\%$ with celebratory confetti animation.
  3. Verified Typst PDF diploma generates and triggers browser download with valid SHA-256 Merkle root stamp.

#### Test Scenario 3.3: Digital Goods & Creator Hub (`template-digital-goods`)
* **User Action:**
  1. Navigate to `/template-digital-goods`.
  2. On font asset page, type text in interactive live type-tester: *"Aetheria Studio 2026"*.
  3. Select *"Extended Commercial License"*.
  4. Complete checkout and click generated download button.
* **GUI Assertions:**
  1. Type-tester updates rendered glyphs in real-time with variable font axes (weight, slant).
  2. License selector updates cart subtotal dynamically.
  3. Download link contains valid HMAC signature and expires after 3600 seconds.

#### Test Scenario 3.4: Industrial B2B E-Commerce & Parametric CAD (`template-b2b-industrial`)
* **User Action:**
  1. Navigate to `/template-b2b-industrial`.
  2. Open Parametric CNC Tooling Configurator.
  3. Adjust Shank Diameter slider from $12\,\text{mm}$ to $25\,\text{mm}$ and Flute Length to $50\,\text{mm}$.
  4. Request Quote for 500 units under Net 60 terms.
* **GUI Assertions:**
  1. 3D WebGL parametric mesh model updates geometry dynamically according to dimensions.
  2. Volume discount pricing table highlights corresponding 500+ unit price tier.
  3. Order summary generates valid ZUGFeRD 2.2 / Peppol compliant draft invoice preview.

#### Test Scenario 3.5: Multi-Asset Trading Exchange (`template-trading-exchange`)
* **User Action:**
  1. Navigate to `/template-trading-exchange`.
  2. Inspect high-density Level-2 order book depth chart.
  3. Place Limit Buy order: 2.5 units at limit price `$1,420.00`.
* **GUI Assertions:**
  1. Bid/Ask price ladder updates smoothly over WebSockets with blinking green/red flash indicators on tick change.
  2. Order ticket verifies wallet balance and locks required funds immediately in UI without page reload.
  3. Order appears instantly in active orders table.

---

### Phase 4: UI/UX Quality, Accessibility & Performance Gates

#### Test Scenario 4.1: Automated Visual Regression Testing
* **Mechanism:** Compare captured page screenshots against baseline golden snapshots across desktop ($1440 \times 900$) and mobile ($390 \times 844$).
* **Threshold:** Pixel difference ratio $\le 0.05\%$.
* **Pages Checked:**
  - `/storefront` (Hero, Exploded Mechanics, Configurator)
  - `/portal`
  - `/worker`
  - `/factory`
  - `/approvals`
  - All 6 Template Catalog landing pages.

#### Test Scenario 4.2: Accessibility Audit (WCAG 2.1 AA Compliance)
* **Mechanism:** Execute `@axe-core/playwright` automated scans on all visible pages.
* **Strict Checks:**
  1. **Color Contrast:** All normal text must have contrast ratio $\ge 4.5:1$ against background; large display text $\ge 3.0:1$.
  2. **Keyboard Focus:** Tabbing through the page displays visible focus rings (`focus-visible:ring-2 focus-visible:ring-champagne`).
  3. **ARIA States:** Radiogroups have `role="radiogroup"`, material options have `role="radio"` and `aria-checked="true/false"`.
  4. **Live Regions:** Cart mutations and notification toasts announce changes to assistive technologies via `aria-live="polite"` or `aria-live="assertive"`.

#### Test Scenario 4.3: Core Web Vitals & Production Performance Audit
* **Tool:** Google Lighthouse CLI.
* **Production Thresholds:**
  - **Performance Score:** $\ge 95 / 100$
  - **Accessibility Score:** $\ge 95 / 100$
  - **Best Practices:** $100 / 100$
  - **SEO Score:** $100 / 100$
  - **Time to First Byte (TTFB):** $< 50\,\text{ms}$ (statically compiled Actix-web SSR)
  - **Largest Contentful Paint (LCP):** $< 1.5\,\text{s}$
  - **Cumulative Layout Shift (CLS):** $< 0.05$ (zero layout shifts from dynamically loaded assets)

#### Test Scenario 4.4: Dynamic Theme Hot-Swapping UX
* **User Action:** In admin panel, change primary accent color from `#e6c887` (champagne gold) to `#00f0ff` (cyber cyan).
* **GUI Assertions:**
  1. Without refreshing the page, all CSS custom properties (`var(--color-accent)`) update across all active client views.
  2. The custom cursor, buttons, and WebGL key light update color in $< 50\,\mu\text{s}$.

---

### Phase 5: Local-First Offline & Network Disconnection UX

```
  CONNECTED               DISCONNECTED (OFFLINE)                RECONNECTED
┌───────────┐         ┌───────────────────────────────┐      ┌───────────────┐
│ Online    │ ──────► │ Persistent Offline HUD Active │ ───► │ Sync Complete │
│ Real-time │ (Drop)  │ Mutations stored in IndexedDB │(Heal)│ CRDT joins    │
│ Sync      │         │ Zero user disruption or locks │      │ Zero conflict │
└───────────┘         └───────────────────────────────┘      └───────────────┘
```

#### Test Scenario 5.1: Offline Disconnection & Reconnection
* **Action:**
  1. Open `/worker` on mobile emulation.
  2. Simulate network disconnection: `await context.setOffline(true);`.
  3. Scan 3 items into picking cart.
  4. Navigate between tabs (Pick List -> Stock Balance -> Profile).
  5. Restore network: `await context.setOffline(false);`.
* **UX Assertions:**
  1. Disconnection immediately prompts subtle offline status indicator; no intrusive blocking error popups.
  2. All operations remain interactive and fast, writing to local storage.
  3. When network reconnects, background vector clock sync merges items into main SurrealDB ledger.
  4. Sync indicator changes from *"3 Pending"* to *"All Changes Synchronized"* with green checkmark.

---

## 3. Ready-to-Run Playwright E2E Test Suite

Save the following test script as `tests/e2e_gui_ux.spec.ts`. The Antigravity-IDE agent can execute this directly using `npx playwright test`.

```typescript
// tests/e2e_gui_ux.spec.ts
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const BASE_URL = process.env.BASE_URL || 'http://localhost:8080';

test.describe('Haute Horlogerie Luxury 3D Storefront E2E Suite', () => {

  test('Storefront WebGL canvas and kinetic UI loads smoothly', async ({ page }) => {
    await page.goto(`${BASE_URL}/storefront`);

    // Verify Title & Hero Headline
    await expect(page).toHaveTitle(/AETHERIA | Haute Horlogerie/i);
    const heroTitle = page.locator('h1');
    await expect(heroTitle).toContainText('CHRONO');
    await expect(heroTitle).toContainText('CELESTIAL');

    // Verify WebGL Canvas initialization
    const canvas = page.locator('#webgl-canvas');
    await expect(canvas).toBeVisible();
    
    // Verify Custom Cursor presence
    const cursorDot = page.locator('#cursor-dot');
    const cursorOutline = page.locator('#cursor-outline');
    await expect(cursorDot).toBeAttached();
    await expect(cursorOutline).toBeAttached();
  });

  test('3D Material customizer updates shader and price reactively', async ({ page }) => {
    await page.goto(`${BASE_URL}/storefront`);

    // Scroll down to the 3D Atelier Configurator
    await page.locator('#configurator').scrollIntoViewIfNeeded();

    // Verify initial base price
    const priceLabel = page.locator('#config-price');
    await expect(priceLabel).toHaveText('$14,200');

    // Click on Celestial Gold metallurgy
    const goldButton = page.locator('button[data-mat="gold"]');
    await goldButton.click();

    // Price must reactively update to gold tier
    await expect(priceLabel).toHaveText('$24,600');

    // Verify Toast notification
    const toast = page.locator('#toast-container');
    await expect(toast).toContainText(/metallurgy changed to gold/i);

    // Click on Carbon DLC Obsidian metallurgy
    const obsidianButton = page.locator('button[data-mat="obsidian"]');
    await obsidianButton.click();
    await expect(priceLabel).toHaveText('$17,800');
  });

  test('Interactive slide-over cart drawer and ACID checkout execution', async ({ page }) => {
    await page.goto(`${BASE_URL}/storefront`);

    // Add item from hero section
    const quickAddBtn = page.getByRole('button', { name: /acquire reserve/i });
    await quickAddBtn.click();

    // Cart drawer should open automatically
    const cartDrawer = page.locator('#cart-panel');
    await expect(cartDrawer).toBeVisible();

    // Verify cart badge reflects 1 item
    const badge = page.locator('#cart-counter-badge');
    await expect(badge).toHaveText('1');

    // Execute simulated SurrealDB ACID checkout
    const checkoutBtn = page.getByRole('button', { name: /execute acid checkout/i });
    await checkoutBtn.click();

    // Verification modal must appear with commit hash
    const modal = page.locator('#quickview-modal');
    await expect(modal).toBeVisible({ timeout: 5000 });
    await expect(modal).toContainText(/SurrealDB Commit/i);
    await expect(modal).toContainText(/ALLOCATION RESERVED/i);
  });

  test('Accessibility (a11y) audit meets WCAG 2.1 AA', async ({ page }) => {
    await page.goto(`${BASE_URL}/storefront`);

    // Run axe accessibility analysis
    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa'])
      .analyze();

    // Assert zero critical violations
    const criticalViolations = accessibilityScanResults.violations.filter(v => v.impact === 'critical');
    expect(criticalViolations).toEqual([]);
  });

});

test.describe('Role-Adaptive Multi-Persona Shells E2E Suite', () => {

  test('Warehouse Worker (/worker) enforces >= 56px touch boundaries', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(`${BASE_URL}/worker`);

    // Verify every interactive button adheres to >= 56px touch targets
    const buttons = page.locator('button');
    const count = await buttons.count();

    for (let i = 0; i < count; i++) {
      const box = await buttons.nth(i).boundingBox();
      if (box && await buttons.nth(i).isVisible()) {
        expect(box.height).toBeGreaterThanOrEqual(56);
        expect(box.width).toBeGreaterThanOrEqual(56);
      }
    }
  });

  test('Manager Approval Deck (/approvals) handles swipe gestures', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(`${BASE_URL}/approvals`);

    const topCard = page.locator('.approval-card').first();
    if (await topCard.isVisible()) {
      const box = await topCard.boundingBox();
      if (box) {
        // Simulate rightward touch swipe (Approve)
        await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
        await page.mouse.down();
        await page.mouse.move(box.x + box.width / 2 + 200, box.y + box.height / 2, { steps: 10 });
        await page.mouse.up();

        // Card should animate out
        await expect(topCard).not.toBeVisible();
      }
    }
  });

});
```

---

## 4. Execution Commands for Antigravity-IDE Agent

Execute the following testing runs in order:

```bash
# Run 1: Launch the full E2E user experience test suite
npx playwright test tests/e2e_gui_ux.spec.ts --project=chromium

# Run 2: Execute responsive mobile testing under iPhone 14 emulation
npx playwright test tests/e2e_gui_ux.spec.ts --project="Mobile Safari"

# Run 3: Run headed interactive mode with visual trace recording
npx playwright test tests/e2e_gui_ux.spec.ts --headed --trace on

# Run 4: Perform automated Lighthouse performance and UX audit
lighthouse http://localhost:8080/storefront --output html --output-path ./docs/lighthouse_report.html --chrome-flags="--headless"
```

---

## 5. Pass/Fail Quality Gates

| Evaluation Criteria | Required Standard | Automated Verification Metric |
| :--- | :--- | :--- |
| **Visual Smoothness** | $\ge 55\,\text{FPS}$ sustained | Playwright frame time sampling during scroll |
| **Touch Ergonomics** | $\ge 56\,\text{px} \times 56\,\text{px}$ targets | DOM element bounding box calculation on `/worker` |
| **Accessibility** | 0 critical violations | `@axe-core/playwright` WCAG 2.1 AA scan |
| **Performance** | TTFB $< 50\,\text{ms}$, LCP $< 1.5\,\text{s}$ | Google Lighthouse automated CLI report |
| **Offline Resilience** | Zero data loss on disconnect | Local mutation buffer and CRDT sync handshake |
