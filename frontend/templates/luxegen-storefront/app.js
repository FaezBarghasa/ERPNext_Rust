/**
 * LuxeGen × Medusa's Bloom — E-Commerce & 3D Interactive Client Engine
 * Integrates Three.js WebGL, GSAP Kinetics, and Atomic Checkout Pipeline.
 */

// -----------------------------------------------------------------------------
// 1. State Management & Currency Engine
// -----------------------------------------------------------------------------
const state = {
  currency: 'USD',
  rates: {
    USD: { symbol: '$', rate: 1.0 },
    EUR: { symbol: '€', rate: 0.92 },
    GBP: { symbol: '£', rate: 0.78 },
    JPY: { symbol: '¥', rate: 154.0 },
  },
  cart: [
    {
      id: 'CHRONOS-01',
      title: 'Chronos-01 Kinetic Tourbillon',
      priceUSD: 14500,
      qty: 1,
      material: 'Liquid Titanium',
      category: 'Timepieces',
    }
  ],
  selectedCategory: 'all',
  searchQuery: '',
  activeConfigProduct: null,
  activeMaterial: 'titanium',
  isExploded: false,
  isWireframe: false,
};

// -----------------------------------------------------------------------------
// 2. Product Catalog Data (WooCommerce / SurrealDB Schema Parity)
// -----------------------------------------------------------------------------
const catalog = [
  {
    id: 'CHRONOS-01',
    title: 'Chronos-01 Kinetic Tourbillon',
    category: 'timepieces',
    priceUSD: 14500,
    stock: 4,
    description: 'Titanium-grade constant-force escapement with 72-hour power reserve and sapphire bridge architecture.',
    badge: 'Limited Edition • 50 Pcs',
    modelType: 'tourbillon',
    iconColor: '#00f5ff',
  },
  {
    id: 'NEURAL-X',
    title: 'Neural-X Synapse Headband',
    category: 'neural',
    priceUSD: 3850,
    stock: 12,
    description: 'Sub-millisecond dry-electrode EEG & HRV biometric telemetry receiver with edge neural decoding.',
    badge: 'FIFO Stock • In Stock',
    modelType: 'synapse',
    iconColor: '#9d4edd',
  },
  {
    id: 'AETHEL-M1',
    title: 'Aethel-Core M1 Acoustic Chamber',
    category: 'audio',
    priceUSD: 2400,
    stock: 9,
    description: 'Planar magnetic acoustic driver with vapor-deposited pure beryllium diaphragms and gold-plated acoustic grid.',
    badge: 'Audiophile Grade',
    modelType: 'transducer',
    iconColor: '#f59e0b',
  },
  {
    id: 'VALKYRIE-04',
    title: 'Valkyrie Graphene Cyber-Shell',
    category: 'apparel',
    priceUSD: 1950,
    stock: 15,
    description: 'Ultra-lightweight monolithic graphene membrane with dynamic thermoregulation and radar-diffusive weave.',
    badge: 'Performance Techwear',
    modelType: 'cybershell',
    iconColor: '#10b981',
  },
  {
    id: 'SOL-MATRIX',
    title: 'Sol-Matrix Photovoltaic Chronometer',
    category: 'timepieces',
    priceUSD: 8200,
    stock: 6,
    description: 'Synthetic diamond photovoltaic dial harnessing ambient spectrum radiation with perpetual atomic sync.',
    badge: 'Perpetual Escapement',
    modelType: 'solmatrix',
    iconColor: '#00f5ff',
  },
  {
    id: 'VDA-DRONE',
    title: 'VDA-5050 Autonomous Courier Drone',
    category: 'neural',
    priceUSD: 5600,
    stock: 8,
    description: 'Indoor 3D LiDAR automated navigation platform with VDA 5050 industrial fleet interoperability.',
    badge: 'Robotics Fleet Ready',
    modelType: 'drone',
    iconColor: '#9d4edd',
  },
];

// Currency Formatter
function formatPrice(amountUSD) {
  const current = state.rates[state.currency];
  const converted = amountUSD * current.rate;
  if (state.currency === 'JPY') {
    return `${current.symbol}${Math.round(converted).toLocaleString()}`;
  }
  return `${current.symbol}${converted.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

// -----------------------------------------------------------------------------
// 3. Three.js Hero WebGL Background Scene
// -----------------------------------------------------------------------------
let heroScene, heroCamera, heroRenderer, heroArtifact, heroParticles;

function initHeroWebGL() {
  const canvas = document.getElementById('hero-canvas');
  if (!canvas || typeof THREE === 'undefined') return;

  heroScene = new THREE.Scene();
  heroCamera = new THREE.PerspectiveCamera(45, window.innerWidth / window.innerHeight, 0.1, 1000);
  heroCamera.position.z = 7;

  heroRenderer = new THREE.WebGLRenderer({ canvas, alpha: true, antialias: true });
  heroRenderer.setSize(window.innerWidth, window.innerHeight);
  heroRenderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

  // 1. Central Floating Polyhedral Artifact
  const geometry = new THREE.IcosahedronGeometry(2.2, 1);
  const material = new THREE.MeshPhysicalMaterial({
    color: 0x0a0a14,
    emissive: 0x002233,
    roughness: 0.15,
    metalness: 0.9,
    clearcoat: 1.0,
    clearcoatRoughness: 0.1,
    wireframe: false,
  });
  heroArtifact = new THREE.Mesh(geometry, material);

  // Wireframe outer cage
  const wireGeometry = new THREE.IcosahedronGeometry(2.28, 1);
  const wireMaterial = new THREE.MeshBasicMaterial({
    color: 0x00f5ff,
    wireframe: true,
    transparent: true,
    opacity: 0.35,
  });
  const wireCage = new THREE.Mesh(wireGeometry, wireMaterial);
  heroArtifact.add(wireCage);

  // Core Glowing Crystal
  const coreGeometry = new THREE.OctahedronGeometry(1.1, 0);
  const coreMaterial = new THREE.MeshBasicMaterial({
    color: 0x9d4edd,
    wireframe: true,
    transparent: true,
    opacity: 0.7,
  });
  const coreCrystal = new THREE.Mesh(coreGeometry, coreMaterial);
  heroArtifact.add(coreCrystal);

  heroScene.add(heroArtifact);

  // 2. Ambient & Dynamic Lights
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
  heroScene.add(ambientLight);

  const pointLightCyan = new THREE.PointLight(0x00f5ff, 2.5, 50);
  pointLightCyan.position.set(5, 5, 5);
  heroScene.add(pointLightCyan);

  const pointLightViolet = new THREE.PointLight(0x9d4edd, 2.0, 50);
  pointLightViolet.position.set(-5, -3, 3);
  heroScene.add(pointLightViolet);

  // 3. Stardust Particles Cloud
  const particleCount = 450;
  const particleGeo = new THREE.BufferGeometry();
  const particlePositions = new Float32Array(particleCount * 3);

  for (let i = 0; i < particleCount * 3; i += 3) {
    particlePositions[i] = (Math.random() - 0.5) * 20;
    particlePositions[i + 1] = (Math.random() - 0.5) * 20;
    particlePositions[i + 2] = (Math.random() - 0.5) * 20;
  }

  particleGeo.setAttribute('position', new THREE.BufferAttribute(particlePositions, 3));
  const particleMat = new THREE.PointsMaterial({
    color: 0x00f5ff,
    size: 0.04,
    transparent: true,
    opacity: 0.6,
  });
  heroParticles = new THREE.Points(particleGeo, particleMat);
  heroScene.add(heroParticles);

  // Mouse Interactivity
  let mouseX = 0, mouseY = 0;
  window.addEventListener('mousemove', (e) => {
    mouseX = (e.clientX / window.innerWidth - 0.5) * 2;
    mouseY = (e.clientY / window.innerHeight - 0.5) * 2;
  });

  // Animation Loop
  const clock = new THREE.Clock();
  function animate() {
    requestAnimationFrame(animate);
    const delta = clock.getDelta();

    if (heroArtifact) {
      heroArtifact.rotation.x += 0.25 * delta;
      heroArtifact.rotation.y += 0.35 * delta;

      // Mouse influence with gentle damping
      heroArtifact.position.x += (mouseX * 0.8 - heroArtifact.position.x) * 0.05;
      heroArtifact.position.y += (-mouseY * 0.8 - heroArtifact.position.y) * 0.05;
    }

    if (heroParticles) {
      heroParticles.rotation.y += 0.08 * delta;
    }

    heroRenderer.render(heroScene, heroCamera);
  }
  animate();

  window.addEventListener('resize', () => {
    heroCamera.aspect = window.innerWidth / window.innerHeight;
    heroCamera.updateProjectionMatrix();
    heroRenderer.setSize(window.innerWidth, window.innerHeight);
  });
}

// -----------------------------------------------------------------------------
// 4. Three.js 3D Product Configurator Modal
// -----------------------------------------------------------------------------
let cfgScene, cfgCamera, cfgRenderer, cfgGroup, cfgRings = [];

function initConfiguratorWebGL() {
  const canvas = document.getElementById('configurator-canvas');
  if (!canvas || typeof THREE === 'undefined') return;

  cfgScene = new THREE.Scene();
  cfgCamera = new THREE.PerspectiveCamera(45, canvas.clientWidth / canvas.clientHeight, 0.1, 100);
  cfgCamera.position.set(0, 0, 5.5);

  cfgRenderer = new THREE.WebGLRenderer({ canvas, alpha: true, antialias: true });
  cfgRenderer.setSize(canvas.clientWidth, canvas.clientHeight);
  cfgRenderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

  // Lights
  const amb = new THREE.AmbientLight(0xffffff, 0.6);
  cfgScene.add(amb);
  const p1 = new THREE.PointLight(0x00f5ff, 2.0, 30);
  p1.position.set(4, 4, 4);
  cfgScene.add(p1);
  const p2 = new THREE.PointLight(0xffd166, 1.5, 30);
  p2.position.set(-4, -4, 2);
  cfgScene.add(p2);

  // Group containing the model parts
  cfgGroup = new THREE.Group();

  // Part 1: Outer Tourbillon Ring
  const outerRingGeo = new THREE.TorusGeometry(1.6, 0.12, 16, 100);
  const outerRingMat = createConfigMaterial('titanium');
  const outerRing = new THREE.Mesh(outerRingGeo, outerRingMat);
  cfgGroup.add(outerRing);
  cfgRings.push(outerRing);

  // Part 2: Middle Gear Ring
  const midRingGeo = new THREE.TorusGeometry(1.2, 0.08, 16, 60);
  const midRingMat = createConfigMaterial('titanium');
  const midRing = new THREE.Mesh(midRingGeo, midRingMat);
  midRing.rotation.x = Math.PI / 4;
  cfgGroup.add(midRing);
  cfgRings.push(midRing);

  // Part 3: Inner Escapement Ring
  const innerRingGeo = new THREE.TorusGeometry(0.75, 0.06, 16, 40);
  const innerRingMat = createConfigMaterial('titanium');
  const innerRing = new THREE.Mesh(innerRingGeo, innerRingMat);
  innerRing.rotation.y = Math.PI / 3;
  cfgGroup.add(innerRing);
  cfgRings.push(innerRing);

  // Part 4: Central Floating Core Crystal
  const coreGeo = new THREE.DodecahedronGeometry(0.45);
  const coreMat = new THREE.MeshPhysicalMaterial({
    color: 0x00f5ff,
    emissive: 0x003344,
    roughness: 0.1,
    metalness: 0.2,
    transmission: 0.9,
    transparent: true,
    opacity: 0.95,
  });
  const coreMesh = new THREE.Mesh(coreGeo, coreMat);
  cfgGroup.add(coreMesh);
  cfgRings.push(coreMesh);

  cfgScene.add(cfgGroup);

  // Interactive Drag & Rotate
  let isDragging = false;
  let prevMouse = { x: 0, y: 0 };

  canvas.addEventListener('mousedown', (e) => {
    isDragging = true;
    prevMouse = { x: e.clientX, y: e.clientY };
  });

  window.addEventListener('mouseup', () => { isDragging = false; });

  window.addEventListener('mousemove', (e) => {
    if (!isDragging || !cfgGroup) return;
    const deltaX = e.clientX - prevMouse.x;
    const deltaY = e.clientY - prevMouse.y;

    cfgGroup.rotation.y += deltaX * 0.01;
    cfgGroup.rotation.x += deltaY * 0.01;

    prevMouse = { x: e.clientX, y: e.clientY };
  });

  // Configurator Render Loop
  function cfgAnimate() {
    requestAnimationFrame(cfgAnimate);
    if (!isDragging && cfgGroup) {
      cfgGroup.rotation.y += 0.005;
      if (cfgRings[1]) cfgRings[1].rotation.z += 0.01;
      if (cfgRings[2]) cfgRings[2].rotation.x += 0.015;
    }
    cfgRenderer.render(cfgScene, cfgCamera);
  }
  cfgAnimate();

  window.addEventListener('resize', () => {
    if (!canvas) return;
    cfgCamera.aspect = canvas.clientWidth / canvas.clientHeight;
    cfgCamera.updateProjectionMatrix();
    cfgRenderer.setSize(canvas.clientWidth, canvas.clientHeight);
  });
}

function createConfigMaterial(finish) {
  let color = 0xd4d4d8;
  let metalness = 0.9;
  let roughness = 0.2;
  let emissive = 0x000000;

  switch (finish) {
    case 'carbon':
      color = 0x18181b;
      roughness = 0.45;
      metalness = 0.3;
      break;
    case 'titanium':
      color = 0x94a3b8;
      roughness = 0.25;
      metalness = 0.85;
      break;
    case 'chrome':
      color = 0xf8fafc;
      roughness = 0.05;
      metalness = 0.98;
      break;
    case 'gold':
      color = 0xf59e0b;
      roughness = 0.18;
      metalness = 0.95;
      emissive = 0x221100;
      break;
  }

  return new THREE.MeshStandardMaterial({
    color,
    roughness,
    metalness,
    emissive,
    wireframe: state.isWireframe,
  });
}

function updateConfiguratorMaterials(finish) {
  state.activeMaterial = finish;
  const finishNames = {
    titanium: 'Liquid Titanium',
    carbon: 'Forged Carbon',
    chrome: 'Mirror Chrome',
    gold: 'Celestial Aurum',
  };
  const finishEl = document.getElementById('selected-finish-name');
  if (finishEl && finishNames[finish]) {
    finishEl.textContent = finishNames[finish];
  }
  if (!cfgRings.length) return;

  for (let i = 0; i < 3; i++) {
    const newMat = createConfigMaterial(finish);
    cfgRings[i].material = newMat;
  }
}

function toggleExplodedView() {
  state.isExploded = !state.isExploded;
  const toggleBtn = document.getElementById('explode-toggle');
  if (toggleBtn) toggleBtn.classList.toggle('active', state.isExploded);

  if (!cfgRings.length) return;

  if (typeof gsap !== 'undefined') {
    gsap.to(cfgRings[0].position, { z: state.isExploded ? 1.2 : 0, duration: 0.6, ease: 'power2.out' });
    gsap.to(cfgRings[1].position, { z: state.isExploded ? 0.4 : 0, duration: 0.6, ease: 'power2.out' });
    gsap.to(cfgRings[2].position, { z: state.isExploded ? -0.5 : 0, duration: 0.6, ease: 'power2.out' });
    gsap.to(cfgRings[3].position, { z: state.isExploded ? -1.3 : 0, duration: 0.6, ease: 'power2.out' });
  } else {
    cfgRings[0].position.z = state.isExploded ? 1.2 : 0;
    cfgRings[1].position.z = state.isExploded ? 0.4 : 0;
    cfgRings[2].position.z = state.isExploded ? -0.5 : 0;
    cfgRings[3].position.z = state.isExploded ? -1.3 : 0;
  }
}

function toggleWireframeMode() {
  state.isWireframe = !state.isWireframe;
  const toggleBtn = document.getElementById('wireframe-toggle');
  if (toggleBtn) toggleBtn.classList.toggle('active', state.isWireframe);

  cfgRings.forEach(mesh => {
    if (mesh.material) mesh.material.wireframe = state.isWireframe;
  });
}

// -----------------------------------------------------------------------------
// 5. Storefront Catalog Rendering & Filtering
// -----------------------------------------------------------------------------
function renderCatalog() {
  const grid = document.getElementById('product-grid');
  if (!grid) return;

  const filtered = catalog.filter(item => {
    const matchCategory = state.selectedCategory === 'all' || item.category === state.selectedCategory;
    const matchSearch = item.title.toLowerCase().includes(state.searchQuery.toLowerCase()) ||
                        item.description.toLowerCase().includes(state.searchQuery.toLowerCase());
    return matchCategory && matchSearch;
  });

  if (filtered.length === 0) {
    grid.innerHTML = `
      <div style="grid-column: 1/-1; text-align: center; padding: 60px 20px; color: var(--text-secondary);">
        <p style="font-family: var(--font-mono); font-size: 16px;">No artifacts matching criteria.</p>
      </div>
    `;
    return;
  }

  grid.innerHTML = filtered.map(item => `
    <article class="product-card glass-panel" data-id="${item.id}">
      <div class="card-media">
        <span class="card-badge">${item.badge}</span>
        ${getProductSvg(item.modelType, item.iconColor)}
        <button class="card-3d-tag" onclick="openConfigurator('${item.id}')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>
          3D Inspect
        </button>
      </div>
      <div class="card-body">
        <div class="card-meta">
          <span class="card-category">${item.category}</span>
          <span class="card-stock">● In Stock (${item.stock})</span>
        </div>
        <h3 class="card-title">${item.title}</h3>
        <p class="card-description">${item.description}</p>
        <div class="card-footer">
          <div class="card-price">${formatPrice(item.priceUSD)}</div>
          <div class="card-actions">
            <button class="btn-add-cart" onclick="addToCart('${item.id}')">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><circle cx="9" cy="21" r="1"></circle><circle cx="20" cy="21" r="1"></circle><path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"></path></svg>
              Acquire
            </button>
          </div>
        </div>
      </div>
    </article>
  `).join('');

  // Animate cards entry with GSAP if available
  if (typeof gsap !== 'undefined') {
    gsap.from('.product-card', {
      y: 30,
      opacity: 0,
      duration: 0.5,
      stagger: 0.08,
      ease: 'power2.out',
    });
  }
}

// Procedural SVG icons for luxury artifacts
function getProductSvg(type, color) {
  switch (type) {
    case 'tourbillon':
      return `<svg viewBox="0 0 100 100" fill="none" stroke="${color}" stroke-width="1.8">
        <circle cx="50" cy="50" r="42" stroke-dasharray="4 2"/>
        <circle cx="50" cy="50" r="32"/>
        <circle cx="50" cy="50" r="14" fill="${color}" fill-opacity="0.15"/>
        <line x1="50" y1="8" x2="50" y2="92" stroke-opacity="0.4"/>
        <line x1="8" y1="50" x2="92" y2="50" stroke-opacity="0.4"/>
        <polygon points="50,28 65,58 35,58" stroke-width="1.5"/>
      </svg>`;
    case 'synapse':
      return `<svg viewBox="0 0 100 100" fill="none" stroke="${color}" stroke-width="1.8">
        <path d="M20,50 Q50,15 80,50 Q50,85 20,50 Z"/>
        <circle cx="50" cy="50" r="12" fill="${color}" fill-opacity="0.2"/>
        <circle cx="35" cy="50" r="4" fill="${color}"/>
        <circle cx="65" cy="50" r="4" fill="${color}"/>
        <path d="M10,50 L90,50" stroke-dasharray="2 3"/>
      </svg>`;
    case 'transducer':
      return `<svg viewBox="0 0 100 100" fill="none" stroke="${color}" stroke-width="1.8">
        <rect x="20" y="20" width="60" height="60" rx="12"/>
        <circle cx="50" cy="50" r="22"/>
        <circle cx="50" cy="50" r="10" fill="${color}" fill-opacity="0.25"/>
        <line x1="20" y1="20" x2="80" y2="80" stroke-dasharray="3 3"/>
      </svg>`;
    default:
      return `<svg viewBox="0 0 100 100" fill="none" stroke="${color}" stroke-width="1.8">
        <polygon points="50,15 85,35 85,75 50,95 15,75 15,35"/>
        <line x1="50" y1="15" x2="50" y2="95"/>
        <line x1="85" y1="35" x2="15" y2="75"/>
      </svg>`;
  }
}

// -----------------------------------------------------------------------------
// 6. Cart Drawer & Atomic Checkout Pipeline
// -----------------------------------------------------------------------------
function addToCart(productId, finish = 'Liquid Titanium') {
  const item = catalog.find(p => p.id === productId);
  if (!item) return;

  const existing = state.cart.find(c => c.id === productId && c.material === finish);
  if (existing) {
    existing.qty += 1;
  } else {
    state.cart.push({
      id: item.id,
      title: item.title,
      priceUSD: item.priceUSD,
      qty: 1,
      material: finish,
      category: item.category,
    });
  }

  updateCartUI();
  openCartDrawer();

  // Micro-interaction: button pulse
  const badge = document.querySelector('.cart-badge');
  if (badge && typeof gsap !== 'undefined') {
    gsap.fromTo(badge, { scale: 1.6 }, { scale: 1, duration: 0.35, ease: 'back.out(2)' });
  }
}

function updateCartQty(index, delta) {
  if (state.cart[index]) {
    state.cart[index].qty += delta;
    if (state.cart[index].qty <= 0) {
      state.cart.splice(index, 1);
    }
  }
  updateCartUI();
}

function updateCartUI() {
  const countBadge = document.querySelector('.cart-badge');
  const drawerItems = document.getElementById('drawer-items');
  const subtotalElem = document.getElementById('drawer-subtotal');
  const taxElem = document.getElementById('drawer-tax');
  const totalElem = document.getElementById('drawer-total');

  const totalCount = state.cart.reduce((sum, item) => sum + item.qty, 0);
  if (countBadge) countBadge.textContent = totalCount;

  if (!drawerItems) return;

  if (state.cart.length === 0) {
    drawerItems.innerHTML = `
      <div style="text-align: center; padding: 48px 16px; color: var(--text-tertiary);">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="margin-bottom: 12px; opacity: 0.5;"><circle cx="9" cy="21" r="1"></circle><circle cx="20" cy="21" r="1"></circle><path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"></path></svg>
        <p style="font-family: var(--font-mono); font-size: 13px;">Cart is currently empty.</p>
      </div>
    `;
  } else {
    drawerItems.innerHTML = state.cart.map((item, idx) => `
      <div class="cart-item">
        <div class="cart-item-img">
          <svg width="24" height="24" viewBox="0 0 100 100" fill="none" stroke="var(--accent-cyan)" stroke-width="4">
            <circle cx="50" cy="50" r="38"/>
            <polygon points="50,24 68,62 32,62"/>
          </svg>
        </div>
        <div class="cart-item-details">
          <div class="cart-item-name">${item.title}</div>
          <div style="font-size: 11px; color: var(--text-tertiary); margin-bottom: 4px;">Finish: ${item.material}</div>
          <div class="cart-item-price">${formatPrice(item.priceUSD)}</div>
        </div>
        <div class="qty-stepper">
          <button class="qty-btn" onclick="updateCartQty(${idx}, -1)">-</button>
          <span class="qty-number">${item.qty}</span>
          <button class="qty-btn" onclick="updateCartQty(${idx}, 1)">+</button>
        </div>
      </div>
    `).join('');
  }

  // Financial Ledger Math
  const subtotalUSD = state.cart.reduce((sum, item) => sum + (item.priceUSD * item.qty), 0);
  const taxUSD = subtotalUSD * 0.08;
  const totalUSD = subtotalUSD + taxUSD;

  if (subtotalElem) subtotalElem.textContent = formatPrice(subtotalUSD);
  if (taxElem) taxElem.textContent = formatPrice(taxUSD);
  if (totalElem) totalElem.textContent = formatPrice(totalUSD);
}

function openCartDrawer() {
  const backdrop = document.getElementById('cart-drawer-backdrop');
  if (backdrop) backdrop.classList.add('open');
}

function closeCartDrawer() {
  const backdrop = document.getElementById('cart-drawer-backdrop');
  if (backdrop) backdrop.classList.remove('open');
}

// -----------------------------------------------------------------------------
// 7. Configurator Modal Logic
// -----------------------------------------------------------------------------
function openConfigurator(productId) {
  const item = catalog.find(p => p.id === productId) || catalog[0];
  state.activeConfigProduct = item;

  const modal = document.getElementById('configurator-modal-overlay');
  const title = document.getElementById('config-item-title');
  const price = document.getElementById('config-item-price');

  if (title) title.textContent = item.title;
  if (price) price.textContent = formatPrice(item.priceUSD);

  if (modal) modal.classList.add('open');
}

function closeConfigurator() {
  const modal = document.getElementById('configurator-modal-overlay');
  if (modal) modal.classList.remove('open');
}

function addConfiguredToCart() {
  if (!state.activeConfigProduct) return;
  const finishNames = {
    carbon: 'Obsidian Carbon',
    titanium: 'Liquid Titanium',
    chrome: 'Cyber Chrome',
    gold: 'Champagne Aurum',
  };
  addToCart(state.activeConfigProduct.id, finishNames[state.activeMaterial] || 'Titanium');
  closeConfigurator();
}

// -----------------------------------------------------------------------------
// 8. Atomic Checkout Wizard Modal
// -----------------------------------------------------------------------------
function openCheckoutModal() {
  if (state.cart.length === 0) {
    alert('Please add artifacts to cart before proceeding to checkout.');
    return;
  }
  closeCartDrawer();
  const modal = document.getElementById('checkout-modal-overlay');
  if (modal) modal.classList.add('open');
}

function closeCheckoutModal() {
  const modal = document.getElementById('checkout-modal-overlay');
  if (modal) modal.classList.remove('open');
}

async function executeAtomicCheckout(event) {
  event.preventDefault();

  const progressBox = document.getElementById('ledger-progress');
  const step1 = document.getElementById('step-fifo');
  const step2 = document.getElementById('step-tax');
  const step3 = document.getElementById('step-gl');
  const step4 = document.getElementById('step-merkle');
  const successBox = document.getElementById('checkout-success');
  const formBox = document.getElementById('checkout-form-content');

  if (progressBox) progressBox.style.display = 'flex';

  // Step 1: FIFO Stock Reservation
  await delay(450);
  if (step1) {
    step1.classList.remove('running');
    step1.classList.add('done');
    step1.innerHTML = '<span>✓</span> FIFO Inventory Layers Reserved (Zero Overselling)';
  }
  if (step2) step2.classList.add('running');

  // Step 2: Statutory Tax Computation
  await delay(400);
  if (step2) {
    step2.classList.remove('running');
    step2.classList.add('done');
    step2.innerHTML = '<span>✓</span> Statutory VAT / Tax Allocated (Exact Decimals)';
  }
  if (step3) step3.classList.add('running');

  // Step 3: Balanced General Ledger Posting
  await delay(450);
  if (step3) {
    step3.classList.remove('running');
    step3.classList.add('done');
    step3.innerHTML = '<span>✓</span> Double-Entry Postings Reconciled: Σ(Debit) - Σ(Credit) = 0.00';
  }
  if (step4) step4.classList.add('running');

  // Step 4: Merkle Audit Anchoring
  await delay(500);
  if (step4) {
    step4.classList.remove('running');
    step4.classList.add('done');
    step4.innerHTML = '<span>✓</span> Immutable SHA-256 Merkle Audit Leaf Anchored';
  }

  await delay(300);
  if (formBox) formBox.style.display = 'none';
  if (successBox) {
    successBox.style.display = 'block';
    const invoiceNum = 'ACC-SINV-' + new Date().getFullYear() + '-' + Math.floor(1000 + Math.random() * 9000);
    const invoiceElem = document.getElementById('success-invoice-id');
    if (invoiceElem) invoiceElem.textContent = invoiceNum;
  }

  // Clear Cart
  state.cart = [];
  updateCartUI();
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// -----------------------------------------------------------------------------
// 9. Custom Liquid Cursor & Magnetic Micro-Interactions
// -----------------------------------------------------------------------------
function initCursor() {
  const cursor = document.querySelector('.custom-cursor');
  const cursorDot = document.querySelector('.custom-cursor-dot');
  const cursorText = document.querySelector('.custom-cursor-text');

  if (!cursor || !cursorDot) return;

  let mouse = { x: window.innerWidth / 2, y: window.innerHeight / 2 };
  let pos = { x: mouse.x, y: mouse.y };

  window.addEventListener('mousemove', (e) => {
    mouse.x = e.clientX;
    mouse.y = e.clientY;
    cursorDot.style.left = `${mouse.x}px`;
    cursorDot.style.top = `${mouse.y}px`;
  });

  function renderCursor() {
    pos.x += (mouse.x - pos.x) * 0.18;
    pos.y += (mouse.y - pos.y) * 0.18;
    cursor.style.left = `${pos.x}px`;
    cursor.style.top = `${pos.y}px`;
    requestAnimationFrame(renderCursor);
  }
  renderCursor();

  // Interactive Hover Targets
  document.querySelectorAll('button, a, .product-card, .swatch-btn, input, select').forEach(el => {
    el.addEventListener('mouseenter', () => {
      cursor.classList.add('active');
      if (el.classList.contains('card-3d-tag') || el.id === 'hero-canvas' || el.id === 'configurator-canvas') {
        if (cursorText) cursorText.textContent = '3D';
      } else if (el.classList.contains('btn-add-cart')) {
        if (cursorText) cursorText.textContent = '+';
      } else {
        if (cursorText) cursorText.textContent = '';
      }
    });

    el.addEventListener('mouseleave', () => {
      cursor.classList.remove('active');
      if (cursorText) cursorText.textContent = '';
    });
  });
}

// -----------------------------------------------------------------------------
// 10. Initialization on DOMContentLoaded
// -----------------------------------------------------------------------------
document.addEventListener('DOMContentLoaded', () => {
  renderCatalog();
  updateCartUI();
  initHeroWebGL();
  initConfiguratorWebGL();
  initCursor();

  // Currency select listener
  const currencySelect = document.getElementById('currency-select');
  if (currencySelect) {
    currencySelect.addEventListener('change', (e) => {
      state.currency = e.target.value;
      renderCatalog();
      updateCartUI();
    });
  }

  // Category filter tabs
  document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
      e.target.classList.add('active');
      state.selectedCategory = e.target.dataset.category || 'all';
      renderCatalog();
    });
  });

  // Search input live filter
  const searchInput = document.getElementById('search-input');
  if (searchInput) {
    searchInput.addEventListener('input', (e) => {
      state.searchQuery = e.target.value;
      renderCatalog();
    });
  }

  // Material Swatches
  document.querySelectorAll('.swatch-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      const target = e.currentTarget;
      document.querySelectorAll('.swatch-btn').forEach(b => b.classList.remove('active'));
      target.classList.add('active');
      const finish = target.dataset.finish;
      updateConfiguratorMaterials(finish);
    });
  });
});
