# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: e2e_auth_admin_webhook.spec.ts >> Phase 8: Enterprise E2E API & Infrastructure Hardening Suite >> Kubernetes Health Probes & Prometheus Metrics Endpoints
- Location: tests/e2e_auth_admin_webhook.spec.ts:7:7

# Error details

```
Error: expect(received).toBeTruthy()

Received: false
```

# Test source

```ts
  1   | import { test, expect } from '@playwright/test';
  2   | 
  3   | const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:8080';
  4   | 
  5   | test.describe('Phase 8: Enterprise E2E API & Infrastructure Hardening Suite', () => {
  6   | 
  7   |   test('Kubernetes Health Probes & Prometheus Metrics Endpoints', async ({ request }) => {
  8   |     // 1. Verify /healthz/live
  9   |     const liveRes = await request.get(`${BASE_URL}/healthz/live`);
  10  |     expect(liveRes.ok()).toBeTruthy();
  11  |     const liveJson = await liveRes.json();
  12  |     expect(liveJson.status).toBe('alive');
  13  | 
  14  |     // 2. Verify /healthz/ready
  15  |     const readyRes = await request.get(`${BASE_URL}/healthz/ready`);
> 16  |     expect(readyRes.ok()).toBeTruthy();
      |                           ^ Error: expect(received).toBeTruthy()
  17  |     const readyJson = await readyRes.json();
  18  |     expect(readyJson.status).toBe('ready');
  19  |     expect(readyJson.db).toBe('connected');
  20  | 
  21  |     // 3. Verify Prometheus /metrics exposition
  22  |     const metricsRes = await request.get(`${BASE_URL}/metrics`);
  23  |     expect(metricsRes.ok()).toBeTruthy();
  24  |     const metricsText = await metricsRes.text();
  25  |     expect(metricsText).toContain('rustnext_http_requests_total');
  26  |     expect(metricsText).toContain('rustnext_uptime_seconds');
  27  |   });
  28  | 
  29  |   test('PWA Web Manifest & Offline Service Worker Endpoints', async ({ request }) => {
  30  |     // 1. Verify /manifest.json
  31  |     const manifestRes = await request.get(`${BASE_URL}/manifest.json`);
  32  |     expect(manifestRes.ok()).toBeTruthy();
  33  |     const manifestJson = await manifestRes.json();
  34  |     expect(manifestJson.name).toContain('RustNext');
  35  |     expect(manifestJson.display).toBe('standalone');
  36  | 
  37  |     // 2. Verify /sw.js
  38  |     const swRes = await request.get(`${BASE_URL}/sw.js`);
  39  |     expect(swRes.ok()).toBeTruthy();
  40  |     const swText = await swRes.text();
  41  |     expect(swText).toContain('rustnext');
  42  |     expect(swText).toContain('addEventListener');
  43  |   });
  44  | 
  45  |   test('Multi-Persona SSR Desktop Shells', async ({ page }) => {
  46  |     // 1. Enterprise Desk
  47  |     await page.goto(`${BASE_URL}/desk`);
  48  |     await expect(page).toHaveTitle(/Enterprise Desk/i);
  49  | 
  50  |     // 2. Customer Portal
  51  |     await page.goto(`${BASE_URL}/portal`);
  52  |     await expect(page).toHaveTitle(/Customer Portal/i);
  53  | 
  54  |     // 3. Shop Floor Terminal
  55  |     await page.goto(`${BASE_URL}/worker`);
  56  |     await expect(page).toHaveTitle(/Shop Floor Terminal/i);
  57  | 
  58  |     // 4. SCADA Floor Cockpit
  59  |     await page.goto(`${BASE_URL}/factory`);
  60  |     await expect(page).toHaveTitle(/SCADA Floor Cockpit/i);
  61  | 
  62  |     // 5. System Fleet Admin
  63  |     await page.goto(`${BASE_URL}/admin`);
  64  |     await expect(page).toHaveTitle(/System Fleet Admin/i);
  65  |   });
  66  | 
  67  |   test('API v2 Ping and System Methods', async ({ request }) => {
  68  |     const pingRes = await request.get(`${BASE_URL}/api/v2/method/ping`);
  69  |     expect(pingRes.ok()).toBeTruthy();
  70  |     const pingJson = await pingRes.json();
  71  |     expect(pingJson.message).toBe('pong');
  72  |     expect(pingJson.timestamp).toBeDefined();
  73  |   });
  74  | 
  75  |   test('API v2 Webhook Subscriptions Management', async ({ request }) => {
  76  |     const subId = `wh_e2e_${Date.now()}`;
  77  |     const newSub = {
  78  |       id: subId,
  79  |       event: 'invoice.paid',
  80  |       target_url: 'https://example.com/webhook-receiver',
  81  |       secret: 'whsec_e2e_super_secret_test_key_2026',
  82  |       is_active: true,
  83  |       created_at: new Date().toISOString(),
  84  |     };
  85  | 
  86  |     // 1. Subscribe
  87  |     const subRes = await request.post(`${BASE_URL}/api/v2/webhooks/subscribe`, {
  88  |       data: newSub,
  89  |     });
  90  |     expect(subRes.ok()).toBeTruthy();
  91  | 
  92  |     // 2. List
  93  |     const listRes = await request.get(`${BASE_URL}/api/v2/webhooks/subscriptions`);
  94  |     expect(listRes.ok()).toBeTruthy();
  95  |     const subs = await listRes.json();
  96  |     expect(Array.isArray(subs)).toBeTruthy();
  97  | 
  98  |     // 3. Delete
  99  |     const delRes = await request.delete(`${BASE_URL}/api/v2/webhooks/subscribe/${subId}`);
  100 |     expect(delRes.ok()).toBeTruthy();
  101 |   });
  102 | 
  103 |   test('Visual CMS Slug Hardening & Page Persistence', async ({ request }) => {
  104 |     const validSlug = 'landing_page_hardened';
  105 |     const testDoc = { title: 'Hardened Landing Page', blocks: [] };
  106 | 
  107 |     // 1. Save draft
  108 |     const saveRes = await request.post(`${BASE_URL}/api/v2/cms/page/${validSlug}/save`, {
  109 |       data: testDoc,
  110 |     });
  111 |     expect(saveRes.ok()).toBeTruthy();
  112 | 
  113 |     // 2. Get draft
  114 |     const getRes = await request.get(`${BASE_URL}/api/v2/cms/page/${validSlug}`);
  115 |     expect(getRes.ok()).toBeTruthy();
  116 | 
```