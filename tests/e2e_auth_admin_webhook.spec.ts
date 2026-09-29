import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:8080';

test.describe('Phase 8: Enterprise E2E API & Infrastructure Hardening Suite', () => {

  test('Kubernetes Health Probes & Prometheus Metrics Endpoints', async ({ request }) => {
    // 1. Verify /healthz/live
    const liveRes = await request.get(`${BASE_URL}/healthz/live`);
    expect(liveRes.ok()).toBeTruthy();
    const liveJson = await liveRes.json();
    expect(liveJson.status).toBe('alive');

    // 2. Verify /healthz/ready
    const readyRes = await request.get(`${BASE_URL}/healthz/ready`);
    expect(readyRes.ok()).toBeTruthy();
    const readyJson = await readyRes.json();
    expect(readyJson.status).toBe('ready');
    expect(readyJson.db).toBe('connected');

    // 3. Verify Prometheus /metrics exposition
    const metricsRes = await request.get(`${BASE_URL}/metrics`);
    expect(metricsRes.ok()).toBeTruthy();
    const metricsText = await metricsRes.text();
    expect(metricsText).toContain('rustnext_http_requests_total');
    expect(metricsText).toContain('rustnext_uptime_seconds');
  });

  test('PWA Web Manifest & Offline Service Worker Endpoints', async ({ request }) => {
    // 1. Verify /manifest.json
    const manifestRes = await request.get(`${BASE_URL}/manifest.json`);
    expect(manifestRes.ok()).toBeTruthy();
    const manifestJson = await manifestRes.json();
    expect(manifestJson.name).toContain('RustNext');
    expect(manifestJson.display).toBe('standalone');

    // 2. Verify /sw.js
    const swRes = await request.get(`${BASE_URL}/sw.js`);
    expect(swRes.ok()).toBeTruthy();
    const swText = await swRes.text();
    expect(swText).toContain('rustnext');
    expect(swText).toContain('addEventListener');
  });

  test('Multi-Persona SSR Desktop Shells', async ({ page }) => {
    // 1. Enterprise Desk
    await page.goto(`${BASE_URL}/desk`);
    await expect(page).toHaveTitle(/Enterprise Desk/i);

    // 2. Customer Portal
    await page.goto(`${BASE_URL}/portal`);
    await expect(page).toHaveTitle(/Customer Portal/i);

    // 3. Shop Floor Terminal
    await page.goto(`${BASE_URL}/worker`);
    await expect(page).toHaveTitle(/Shop Floor Terminal/i);

    // 4. SCADA Floor Cockpit
    await page.goto(`${BASE_URL}/factory`);
    await expect(page).toHaveTitle(/SCADA Floor Cockpit/i);

    // 5. System Fleet Admin
    await page.goto(`${BASE_URL}/admin`);
    await expect(page).toHaveTitle(/System Fleet Admin/i);
  });

  test('API v2 Ping and System Methods', async ({ request }) => {
    const pingRes = await request.get(`${BASE_URL}/api/v2/method/ping`);
    expect(pingRes.ok()).toBeTruthy();
    const pingJson = await pingRes.json();
    expect(pingJson.message).toBe('pong');
    expect(pingJson.timestamp).toBeDefined();
  });

  test('API v2 Webhook Subscriptions Management', async ({ request }) => {
    const subId = `wh_e2e_${Date.now()}`;
    const newSub = {
      id: subId,
      event: 'invoice.paid',
      target_url: 'https://example.com/webhook-receiver',
      secret: 'whsec_e2e_super_secret_test_key_2026',
      is_active: true,
      created_at: new Date().toISOString(),
    };

    // 1. Subscribe
    const subRes = await request.post(`${BASE_URL}/api/v2/webhooks/subscribe`, {
      data: newSub,
    });
    expect(subRes.ok()).toBeTruthy();

    // 2. List
    const listRes = await request.get(`${BASE_URL}/api/v2/webhooks/subscriptions`);
    expect(listRes.ok()).toBeTruthy();
    const subs = await listRes.json();
    expect(Array.isArray(subs)).toBeTruthy();

    // 3. Delete
    const delRes = await request.delete(`${BASE_URL}/api/v2/webhooks/subscribe/${subId}`);
    expect(delRes.ok()).toBeTruthy();
  });

  test('Visual CMS Slug Hardening & Page Persistence', async ({ request }) => {
    const validSlug = 'landing_page_hardened';
    const testDoc = { title: 'Hardened Landing Page', blocks: [] };

    // 1. Save draft
    const saveRes = await request.post(`${BASE_URL}/api/v2/cms/page/${validSlug}/save`, {
      data: testDoc,
    });
    expect(saveRes.ok()).toBeTruthy();

    // 2. Get draft
    const getRes = await request.get(`${BASE_URL}/api/v2/cms/page/${validSlug}`);
    expect(getRes.ok()).toBeTruthy();

    // 3. Reject malicious SQL injection slug
    const malRes = await request.get(`${BASE_URL}/api/v2/cms/page/slug;DROP TABLE user;`);
    expect(malRes.status()).toBe(400);
  });
});
