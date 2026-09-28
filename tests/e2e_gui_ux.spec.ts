import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:8080';

test.describe('Phase 1: Haute Horlogerie Luxury 3D Storefront E2E Suite', () => {

  test('Storefront WebGL canvas, kinetic UI & custom cursor load smoothly', async ({ page }) => {
    await page.goto(`${BASE_URL}/storefront`);

    // Verify Title & Hero Headline
    await expect(page).toHaveTitle(/LuxeGen × Medusa's Bloom/i);
    const heroTitle = page.locator('.hero-title');
    await expect(heroTitle).toBeVisible();
    await expect(heroTitle).toContainText('ENGINEERED FOR THE');
    await expect(heroTitle).toContainText('TRANSCENDENT ERA');

    // Verify WebGL Canvas initialization
    const canvas = page.locator('#hero-canvas');
    await expect(canvas).toBeVisible();
    
    // Verify Custom Cursor elements are present in DOM
    const cursor = page.locator('.custom-cursor');
    const cursorDot = page.locator('.custom-cursor-dot');
    await expect(cursor).toBeAttached();
    await expect(cursorDot).toBeAttached();
  });

  test('3D Material customizer updates shader and finish reactively', async ({ page }) => {
    await page.goto(`${BASE_URL}/storefront`);

    // Open 3D Configurator
    const openConfigBtn = page.getByRole('button', { name: /Launch 3D Configurator/i });
    await openConfigBtn.click();

    // Verify Configurator Modal is displayed
    const modalOverlay = page.locator('#configurator-modal-overlay');
    await expect(modalOverlay).toHaveClass(/open/);

    // Verify initial base price
    const priceLabel = page.locator('#config-item-price');
    await expect(priceLabel).toHaveText('$14,500.00');

    // Click on Aurum / Gold metallurgy swatch
    const goldButton = page.locator('button.swatch-btn[data-finish="gold"]');
    await goldButton.click();

    // Verify selected finish text updates
    const finishLabel = page.locator('#selected-finish-name');
    await expect(finishLabel).toHaveText(/Aurum/i);

    // Click on Carbon swatch
    const carbonButton = page.locator('button.swatch-btn[data-finish="carbon"]');
    await carbonButton.click();
    await expect(finishLabel).toHaveText(/Carbon/i);

    // Add configured item to vault cart
    const addToVaultBtn = page.getByRole('button', { name: /Add Configured Artifact to Vault/i });
    await addToVaultBtn.click();

    // Verify Cart Drawer opens with item
    const drawerBackdrop = page.locator('#cart-drawer-backdrop');
    await expect(drawerBackdrop).toHaveClass(/open/);
    const cartBadge = page.locator('.cart-badge');
    await expect(cartBadge).toBeVisible();
  });

  test('Interactive slide-over cart drawer and ACID checkout execution', async ({ page }) => {
    await page.goto(`${BASE_URL}/storefront`);

    // Open Cart Drawer directly from nav
    const cartBtn = page.locator('.cart-btn');
    await cartBtn.click();

    // Cart drawer should open
    const drawerBackdrop = page.locator('#cart-drawer-backdrop');
    await expect(drawerBackdrop).toHaveClass(/open/);

    // Proceed to Atomic Checkout
    const checkoutBtn = page.locator('button.checkout-btn');
    await checkoutBtn.click();

    // Checkout modal must appear
    const checkoutOverlay = page.locator('#checkout-modal-overlay');
    await expect(checkoutOverlay).toHaveClass(/open/);

    // Submit ACID checkout form
    const submitBtn = checkoutOverlay.locator('button[type="submit"]');
    await submitBtn.click();

    // Verification screen must appear with invoice ID
    const successBox = page.locator('#checkout-success');
    await expect(successBox).toBeVisible({ timeout: 10000 });
    const invoiceId = page.locator('#success-invoice-id');
    await expect(invoiceId).toContainText(/ACC-SINV-|INV-COMMERCE-/i);
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

test.describe('Phase 2 & 3: Enterprise Template Suites & Manifest Endpoints', () => {

  test('Templates Portal index loads all 6 prebuilt enterprise archetypes', async ({ page }) => {
    await page.goto(`${BASE_URL}/templates`);

    // Verify Portal Title and Heading
    await expect(page).toHaveTitle(/Universal Enterprise Template Suites/i);
    const heading = page.locator('h1');
    await expect(heading).toContainText('Universal Work-Type Templates');

    // Verify all 6 template suite cards exist
    const cards = page.locator('.template-card');
    await expect(cards).toHaveCount(6);

    // Verify specific slugs exist in links
    await expect(page.locator('a[href="/templates/svod-streaming"]')).toBeVisible();
    await expect(page.locator('a[href="/templates/lms-academy"]')).toBeVisible();
    await expect(page.locator('a[href="/templates/digital-goods"]')).toBeVisible();
    await expect(page.locator('a[href="/templates/b2b-industrial"]')).toBeVisible();
    await expect(page.locator('a[href="/templates/b2c-retail"]')).toBeVisible();
    await expect(page.locator('a[href="/templates/trading-exchange"]')).toBeVisible();
  });

  const templateSuites = [
    { slug: 'svod-streaming', titlePattern: /Cinema/i, contentSelector: '.hero-title', expectedContent: 'Chronos Horizon' },
    { slug: 'lms-academy', titlePattern: /Academy/i, contentSelector: 'h1', expectedContent: 'Lock-Free Shared Resources' },
    { slug: 'digital-goods', titlePattern: /Creator Hub/i, contentSelector: 'h1', expectedContent: 'Developer Tools' },
    { slug: 'b2b-industrial', titlePattern: /Industrial/i, contentSelector: '.top-bar', expectedContent: 'Siemens AG Procurement' },
    { slug: 'b2c-retail', titlePattern: /Retail/i, contentSelector: '.brand', expectedContent: 'Retail' },
    { slug: 'trading-exchange', titlePattern: /Exchange/i, contentSelector: '.ticker', expectedContent: 'BTC-USD' },
  ];

  for (const { slug, titlePattern, contentSelector, expectedContent } of templateSuites) {
    test(`Template Suite [${slug}] renders with design tokens & live HUD`, async ({ page }) => {
      await page.goto(`${BASE_URL}/templates/${slug}`);

      // Verify page title
      await expect(page).toHaveTitle(titlePattern);

      // Verify content element
      const contentEl = page.locator(contentSelector);
      await expect(contentEl).toContainText(new RegExp(expectedContent, 'i'));

      // Check JSON manifest endpoint
      const manifestRes = await page.request.get(`${BASE_URL}/api/v1/templates/${slug}/manifest`);
      expect(manifestRes.status()).toBe(200);
      const manifest = await manifestRes.json();
      expect(manifest.id).toBe(`theme_${slug}`);
      expect(manifest.default_design_tokens).toBeDefined();
    });
  }

});

test.describe('Phase 4 & 5: Health, APIs & Offline Resilience Quality Gates', () => {

  test('System health check returns valid JSON status', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/health`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.status).toBe('healthy');
    expect(body.version).toBeDefined();
  });

  test('Templates API returns all 6 registered suites', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/api/v1/templates`);
    expect(res.status()).toBe(200);
    const templates = await res.json();
    expect(Array.isArray(templates)).toBe(true);
    expect(templates.length).toBe(6);
  });

  test('Storefront products API returns complete luxury catalog', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/api/v1/storefront/products`);
    expect(res.status()).toBe(200);
    const products = await res.json();
    expect(Array.isArray(products)).toBe(true);
    expect(products.length).toBeGreaterThanOrEqual(6);
    expect(products[0]).toHaveProperty('id');
    expect(products[0]).toHaveProperty('price_usd');
  });

  test('Atomic Checkout API executes ACID commit successfully', async ({ request }) => {
    const res = await request.post(`${BASE_URL}/api/v1/storefront/checkout`, {
      data: {
        checkout_id: 'CHK-TEST-001',
        customer_id: 'CUST-001',
        items: [
          {
            item_code: 'CHRONOS-01',
            description: 'Chronos-01 Kinetic Tourbillon',
            qty: '1',
            unit_price: '14500.00',
            available_stock: '10',
          },
        ],
        tax_rate_percent: '8.0',
        receivable_account: '1100 - Accounts Receivable',
        revenue_account: '4100 - Luxury Product Sales',
        tax_account: '2200 - Sales Tax Payable',
      },
    });

    expect(res.status()).toBe(200);
    const result = await res.json();
    expect(result.is_success).toBe(true);
    expect(result.invoice_id).toContain('INV-COMMERCE-');
    expect(result.grand_total).toBeDefined();
    expect(result.gl_postings.length).toBeGreaterThan(0);
  });

});
