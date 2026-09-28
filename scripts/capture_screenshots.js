const { chromium } = require('@playwright/test');
const path = require('path');
const fs = require('fs');

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:8080';
const OUT_DIR = path.resolve(__dirname, '../docs/screenshots');

async function capture() {
  if (!fs.existsSync(OUT_DIR)) {
    fs.mkdirSync(OUT_DIR, { recursive: true });
  }

  const browser = await chromium.launch();

  // 1. Desktop Browser Context (1440x900)
  const desktopContext = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 2,
  });
  const page = await desktopContext.newPage();

  console.log('Capturing 01_storefront_desktop.png...');
  await page.goto(`${BASE_URL}/storefront`, { waitUntil: 'networkidle' });
  await page.waitForTimeout(1000);
  await page.screenshot({ path: path.join(OUT_DIR, '01_storefront_desktop.png'), fullPage: false });

  console.log('Capturing 02_storefront_configurator.png...');
  const openConfigBtn = page.getByRole('button', { name: /Launch 3D Configurator/i });
  if (await openConfigBtn.isVisible()) {
    await openConfigBtn.click();
    await page.waitForTimeout(800);
    // Switch to Aurum
    const goldSwatch = page.locator('button.swatch-btn[data-finish="gold"]');
    if (await goldSwatch.isVisible()) await goldSwatch.click();
    await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(OUT_DIR, '02_storefront_configurator.png') });
  }

  console.log('Capturing 03_storefront_cart_vault.png...');
  const addConfigBtn = page.getByRole('button', { name: /Add Configured Artifact to Vault/i });
  if (await addConfigBtn.isVisible()) {
    await addConfigBtn.click();
    await page.waitForTimeout(600);
    await page.screenshot({ path: path.join(OUT_DIR, '03_storefront_cart_vault.png') });
  }

  console.log('Capturing 04_storefront_acid_checkout.png...');
  const checkoutBtn = page.locator('button.checkout-btn');
  if (await checkoutBtn.isVisible()) {
    await checkoutBtn.click();
    await page.waitForTimeout(600);
    await page.screenshot({ path: path.join(OUT_DIR, '04_storefront_acid_checkout.png') });
  }

  // 2. Templates Portal & 6 Template Archetypes
  console.log('Capturing 06_templates_portal.png...');
  await page.goto(`${BASE_URL}/templates`, { waitUntil: 'networkidle' });
  await page.waitForTimeout(600);
  await page.screenshot({ path: path.join(OUT_DIR, '06_templates_portal.png'), fullPage: false });

  const templates = [
    { slug: 'svod-streaming', file: '07_template_svod_streaming.png' },
    { slug: 'lms-academy', file: '08_template_lms_academy.png' },
    { slug: 'digital-goods', file: '09_template_digital_goods.png' },
    { slug: 'b2b-industrial', file: '10_template_b2b_industrial.png' },
    { slug: 'b2c-retail', file: '11_template_b2c_retail.png' },
    { slug: 'trading-exchange', file: '12_template_trading_exchange.png' },
  ];

  for (const t of templates) {
    console.log(`Capturing ${t.file}...`);
    await page.goto(`${BASE_URL}/templates/${t.slug}`, { waitUntil: 'networkidle' });
    await page.waitForTimeout(600);
    await page.screenshot({ path: path.join(OUT_DIR, t.file), fullPage: false });
  }

  await desktopContext.close();

  // 3. Mobile Browser Context (iPhone 14: 390x844)
  console.log('Capturing 05_storefront_mobile.png...');
  const mobileContext = await browser.newContext({
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: 3,
    isMobile: true,
    hasTouch: true,
  });
  const mobilePage = await mobileContext.newPage();
  await mobilePage.goto(`${BASE_URL}/storefront`, { waitUntil: 'networkidle' });
  await mobilePage.waitForTimeout(1000);
  await mobilePage.screenshot({ path: path.join(OUT_DIR, '05_storefront_mobile.png'), fullPage: false });
  await mobileContext.close();

  await browser.close();
  console.log('All screenshots captured successfully in docs/screenshots/!');
}

capture().catch(err => {
  console.error('Error capturing screenshots:', err);
  process.exit(1);
});
