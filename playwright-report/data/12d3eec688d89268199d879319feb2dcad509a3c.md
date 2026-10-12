# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: e2e_gui_ux.spec.ts >> Phase 4 & 5: Health, APIs & Offline Resilience Quality Gates >> Templates API returns all 6 registered suites
- Location: tests/e2e_gui_ux.spec.ts:177:7

# Error details

```
Error: expect(received).toBe(expected) // Object.is equality

Expected: 6
Received: 10
```

# Test source

```ts
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
  125 |     await expect(cards).toHaveCount(6);
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
> 182 |     expect(templates.length).toBe(6);
      |                              ^ Error: expect(received).toBe(expected) // Object.is equality
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