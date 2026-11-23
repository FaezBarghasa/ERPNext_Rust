//! Interactive Look-Alike Showcase Components (`erp_cms::archetype_showcases`).
//!
//! Delivers high-fidelity, interactive, zero-slop UI components tailored to each benchmark
//! reference archetype: 4K streaming billboards, 10-foot TV leanback grids, Steam game stores,
//! subscription vaults, music players with audio waveforms, headless CMS canvases, Awwwards portfolios,
//! industrial telemetry dashboards, and 3D WebGL spatial viewports.

/// Compiles an authentic, interactive look-alike showcase section for a given template slug.
#[must_use]
pub fn render_archetype_showcase_html(slug: &str, title: &str) -> String {
    if slug.starts_with("stream-") || slug.starts_with("tv-") {
        render_streaming_showcase(slug, title)
    } else if slug.starts_with("shop-vapor")
        || slug.starts_with("shop-joker")
        || slug.starts_with("shop-occult")
        || slug.starts_with("shop-bullet")
        || slug.starts_with("shop-underworld")
        || slug.starts_with("shop-deep")
    {
        render_gaming_store_showcase(slug, title)
    } else if slug.starts_with("sub-") {
        render_subscription_showcase(slug, title)
    } else if slug.starts_with("music-") {
        render_music_player_showcase(slug, title)
    } else if slug.starts_with("cms-") {
        render_cms_builder_showcase(slug, title)
    } else if slug.starts_with("portfolio-") {
        render_portfolio_showcase(slug, title)
    } else if slug.starts_with("ind-") {
        render_industrial_showcase(slug, title)
    } else if slug.starts_with("viewer-") {
        render_3d_spatial_showcase(slug, title)
    } else if slug.starts_with("shop-") {
        render_dtc_eshop_showcase(slug, title)
    } else {
        String::new()
    }
}

fn render_streaming_showcase(slug: &str, title: &str) -> String {
    let is_tv = slug.starts_with("tv-");
    let tv_badge = if is_tv {
        r#"<span class="showcase-badge tv-badge">10-Foot Leanback Interface • D-Pad Remote Ready</span>"#
    } else {
        r#"<span class="showcase-badge">4K HLS Master Stream • Dolby Atmos 7.1</span>"#
    };

    let tv_remote_dock = if is_tv {
        r#"<div class="tv-remote-bar">
            <span class="remote-label">D-Pad Navigation:</span>
            <div class="remote-keys">
                <button class="key-btn" onclick="simulateTvKey('Left')">◀</button>
                <button class="key-btn" onclick="simulateTvKey('Up')">▲</button>
                <button class="key-btn select-key" onclick="simulateTvKey('OK')">OK</button>
                <button class="key-btn" onclick="simulateTvKey('Down')">▼</button>
                <button class="key-btn" onclick="simulateTvKey('Right')">▶</button>
            </div>
            <div class="hdmi-inputs">
                <span class="hdmi-tag active">HDMI 1 (Consoles)</span>
                <span class="hdmi-tag">HDMI 2 (eARC Audio)</span>
                <span class="hdmi-tag">AV Live Tuner</span>
            </div>
        </div>"#
    } else {
        ""
    };

    format!(
        r##"<section class="archetype-showcase-container" aria-label="Interactive Showcase">
            <div class="showcase-header">
                <div class="showcase-title-row">
                    <h3>Interactive Platform Preview</h3>
                    {tv_badge}
                </div>
                <p>Live SSR simulation of {title} with real-time controls and adaptive playback.</p>
            </div>

            <div class="streaming-billboard">
                <div class="billboard-video-viewport">
                    <div class="viewport-backdrop"></div>
                    <div class="video-overlay-content">
                        <div class="maturity-rating-strip">
                            <span class="rating-box">16+</span>
                            <span class="audio-box">5.1 SURROUND</span>
                            <span class="res-box">4K ULTRA HD</span>
                            <span class="hdr-box">DOLBY VISION</span>
                        </div>
                        <h2 class="featured-media-title">{title}</h2>
                        <p class="featured-synopsis">When anomalous transmissions penetrate the orbital satellite relay, an elite aerospace recovery squad embarks on a deep orbital investigation.</p>
                        <div class="player-action-row">
                            <button class="btn-play-hero" onclick="togglePlaySimulation(this)">
                                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>
                                <span>Play Episode 1</span>
                            </button>
                            <button class="btn-list-hero" onclick="toggleWatchlist(this)">
                                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                                <span>Add to Watchlist</span>
                            </button>
                            <button class="btn-volume-hero" onclick="toggleMute(this)" title="Toggle Audio">
                                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>
                            </button>
                        </div>
                        <div class="live-scrub-bar">
                            <div class="scrub-progress" style="width: 42%;"></div>
                            <div class="scrub-time">00:42:15 / 01:38:00</div>
                        </div>
                    </div>
                </div>

                {tv_remote_dock}

                <div class="tray-section">
                    <h4 class="tray-title">Trending in Your Region</h4>
                    <div class="tray-carousel">
                        <div class="tray-card active" tabindex="0">
                            <div class="tray-thumb thumb-1"></div>
                            <div class="tray-meta">
                                <span class="match-score">98% Match</span>
                                <span class="badge-tag">Top 10</span>
                                <p class="card-name">The Obsidian Horizon</p>
                            </div>
                        </div>
                        <div class="tray-card" tabindex="0">
                            <div class="tray-thumb thumb-2"></div>
                            <div class="tray-meta">
                                <span class="match-score">95% Match</span>
                                <span class="badge-tag">New Season</span>
                                <p class="card-name">Quantum Protocol: Tokyo</p>
                            </div>
                        </div>
                        <div class="tray-card" tabindex="0">
                            <div class="tray-thumb thumb-3"></div>
                            <div class="tray-meta">
                                <span class="match-score">92% Match</span>
                                <span class="badge-tag">Award Winner</span>
                                <p class="card-name">Subsurface Trench 7</p>
                            </div>
                        </div>
                        <div class="tray-card" tabindex="0">
                            <div class="tray-thumb thumb-4"></div>
                            <div class="tray-meta">
                                <span class="match-score">96% Match</span>
                                <span class="badge-tag">Dolby Atmos</span>
                                <p class="card-name">Stuttgart Motors: Atelier</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>"##,
        title = title,
        tv_badge = tv_badge,
        tv_remote_dock = tv_remote_dock
    )
}

fn render_gaming_store_showcase(slug: &str, title: &str) -> String {
    let (game_tags, price, discount) = if slug.contains("joker") {
        ("Roguelike Deckbuilder • Poker Mechanics • Hypnotic Synth", "$14.99", "-10%")
    } else if slug.contains("occult") {
        ("Colony Sim • Action Roguelike • Cult Management • Dark Humor", "$24.99", "-25%")
    } else if slug.contains("bullet") {
        ("Bullet Hell • Auto-Shooter • Pixel Graphics • Endless Survival", "$4.99", "-20%")
    } else if slug.contains("underworld") {
        ("Mythic Action • Hack and Slash • Hand-Drawn Art • Roguelite", "$29.99", "-15%")
    } else if slug.contains("deep") {
        ("Aquatic Adventure • Sushi Bistro • Deep Diving • Pixel RPG", "$19.99", "-20%")
    } else {
        ("PC Master Race • Cloud Saves • Workshop Mods • Achievements", "$39.99", "-60%")
    };

    format!(
        r##"<section class="archetype-showcase-container" aria-label="Gaming Store Showcase">
            <div class="showcase-header">
                <div class="showcase-title-row">
                    <h3>Steam & Digital Goods Interactive Storefront</h3>
                    <span class="showcase-badge vapor-badge">Verified Playable • Zero-Decimal Ledger</span>
                </div>
                <p>Authentic PC store display inspired by Valve Steam and critically acclaimed independent title pages.</p>
            </div>

            <div class="gaming-store-layout">
                <div class="store-media-col">
                    <div class="store-main-viewport">
                        <div class="gameplay-canvas" id="gameplay-screen">
                            <div class="gameplay-hud-overlay">
                                <span class="hud-fps">FPS: 144</span>
                                <span class="hud-res">3840×2160 V-Sync</span>
                            </div>
                            <div class="gameplay-hero-badge">IN-GAME CAPTURE</div>
                        </div>
                    </div>
                    <div class="store-thumb-row">
                        <button class="thumb-btn active" onclick="swapGameThumb(0)">Thumb 1</button>
                        <button class="thumb-btn" onclick="swapGameThumb(1)">Thumb 2</button>
                        <button class="thumb-btn" onclick="swapGameThumb(2)">Thumb 3</button>
                        <button class="thumb-btn" onclick="swapGameThumb(3)">Gameplay Video</button>
                    </div>
                </div>

                <div class="store-info-col">
                    <div class="store-header-info">
                        <h2 class="game-display-title">{title}</h2>
                        <div class="reviews-meter">
                            <span class="review-label">All Reviews:</span>
                            <span class="review-value-positive">Overwhelmingly Positive (98% of 124,530)</span>
                        </div>
                        <div class="release-date-row">
                            <span class="info-label">Release Date:</span>
                            <span class="info-val">October 24, 2026</span>
                        </div>
                        <div class="tags-cluster">
                            <span class="tag-pill">{game_tags}</span>
                        </div>
                    </div>

                    <div class="buy-module">
                        <div class="buy-module-header">
                            <h4>Buy {title}</h4>
                            <div class="platform-icons">
                                <span title="Windows PC">🪟</span>
                                <span title="Steam Deck Verified">🎮</span>
                                <span title="Linux Native">🐧</span>
                            </div>
                        </div>
                        <div class="buy-price-row">
                            <span class="discount-pill">{discount}</span>
                            <span class="orig-price">$49.99</span>
                            <span class="final-price">{price}</span>
                            <button class="btn-buy-game" onclick="triggerStorePurchase(this, '{title}')">Add to Cart</button>
                            <button class="btn-wishlist" onclick="toggleWishlistBtn(this)">★ Wishlist</button>
                        </div>
                    </div>

                    <div class="system-specs-mini">
                        <span class="spec-head">Minimum Specs:</span>
                        <p class="spec-body">64-bit Quad-Core CPU, 8 GB RAM, Vulkan 1.3 / DirectX 12, 12 GB High-Speed NVMe Storage.</p>
                    </div>
                </div>
            </div>
        </section>"##,
        title = title,
        game_tags = game_tags,
        price = price,
        discount = discount
    )
}

fn render_subscription_showcase(_slug: &str, _title: &str) -> String {
    r##"<section class="archetype-showcase-container" aria-label="Subscription Showcase">
            <div class="showcase-header">
                <div class="showcase-title-row">
                    <h3>Multi-Tier Membership & Digital Pass Vault</h3>
                    <span class="showcase-badge sub-badge">Auto-Renewal • Instant Entitlement Unlock</span>
                </div>
                <p>Enterprise subscription matrix inspired by PlayStation Plus, Game Pass, and Adobe Creative Cloud.</p>
            </div>

            <div class="sub-billing-toggle-row">
                <span class="billing-opt active" id="btn-monthly" onclick="setBillingCycle('monthly')">Monthly Billing</span>
                <span class="billing-opt" id="btn-annual" onclick="setBillingCycle('annual')">Annual Billing (Save 20%)</span>
            </div>

            <div class="sub-cards-grid">
                <div class="sub-card">
                    <span class="tier-name">Essential Pass</span>
                    <div class="tier-price"><span class="price-val" data-m="$9.99" data-a="$7.99">$9.99</span><span class="price-freq">/ month</span></div>
                    <p class="tier-desc">Core access, online multiplayer matchmaking, and 100 GB encrypted cloud backup storage.</p>
                    <ul class="tier-perks">
                        <li>✓ 3 Monthly Curated Free Titles</li>
                        <li>✓ Cloud Save Encryption & Sync</li>
                        <li>✓ Standard HD Streaming Delivery</li>
                        <li class="disabled">✕ Day-One AAA Studio Releases</li>
                        <li class="disabled">✕ Cloud Virtual Machine Streaming</li>
                    </ul>
                    <button class="btn-sub-tier" onclick="subscribeTier('Essential', this)">Choose Essential</button>
                </div>

                <div class="sub-card tier-featured">
                    <div class="featured-ribbon">MOST POPULAR</div>
                    <span class="tier-name">Extra Vault</span>
                    <div class="tier-price"><span class="price-val" data-m="$14.99" data-a="$11.99">$14.99</span><span class="price-freq">/ month</span></div>
                    <p class="tier-desc">Access to full catalogue of 400+ titles, lossless streaming, and priority server queues.</p>
                    <ul class="tier-perks">
                        <li>✓ All Essential Tier Benefits</li>
                        <li>✓ 400+ Downloadable Library Catalogue</li>
                        <li>✓ 4K HDR High-Bitrate Ingestion</li>
                        <li>✓ 2 Simultaneous Active Devices</li>
                        <li class="disabled">✕ Retro Classics Cloud Emulation</li>
                    </ul>
                    <button class="btn-sub-tier btn-primary-tier" onclick="subscribeTier('Extra', this)">Choose Extra Vault</button>
                </div>

                <div class="sub-card">
                    <span class="tier-name">Deluxe Studio Pass</span>
                    <div class="tier-price"><span class="price-val" data-m="$19.99" data-a="$15.99">$19.99</span><span class="price-freq">/ month</span></div>
                    <p class="tier-desc">The complete enterprise entertainment pass including cloud streaming and timed game trials.</p>
                    <ul class="tier-perks">
                        <li>✓ Everything in Extra & Essential</li>
                        <li>✓ Cloud Game Streaming (Zero-Install)</li>
                        <li>✓ Time-Limited Pre-Release Trials</li>
                        <li>✓ 5 Household Member Sub-Accounts</li>
                        <li>✓ 24-Bit / 192kHz Audiophile FLAC Tier</li>
                    </ul>
                    <button class="btn-sub-tier" onclick="subscribeTier('Deluxe', this)">Choose Deluxe Pass</button>
                </div>
            </div>
        </section>"##.to_string()
}

fn render_music_player_showcase(_slug: &str, title: &str) -> String {
    format!(
        r##"<section class="archetype-showcase-container" aria-label="Music Streaming Showcase">
            <div class="showcase-header">
                <div class="showcase-title-row">
                    <h3>Desktop Hi-Fi Music Streaming Studio</h3>
                    <span class="showcase-badge music-badge">Lossless FLAC • WebAudio DSP Normalization</span>
                </div>
                <p>Interactive web music player inspired by Spotify Web, SoundCloud, and Tidal Hi-Fi.</p>
            </div>

            <div class="music-player-frame">
                <div class="music-sidebar">
                    <div class="sidebar-item active">♫ Discover Radar</div>
                    <div class="sidebar-item">♥ Liked Tracks (2,410)</div>
                    <div class="sidebar-item">📁 Heavy Rotation</div>
                    <div class="sidebar-item">📻 Curated Live Radio</div>
                    <div class="sidebar-item">⚙ DSP & EQ Studio</div>
                </div>

                <div class="music-content-pane">
                    <div class="album-hero-banner">
                        <div class="album-vinyl-cover"></div>
                        <div class="album-details">
                            <span class="playlist-tag">EDITORIAL MASTER PLAYLIST</span>
                            <h2 class="album-heading">{title}</h2>
                            <p class="album-meta-text">Curated by Sound Engineers • 48 Songs, 3 hr 14 min • 14,892,100 Monthly Listeners</p>
                            <div class="album-btn-row">
                                <button class="btn-music-play" onclick="toggleMusicPlayback(this)">▶ Play Master</button>
                                <button class="btn-music-fav" onclick="toggleFavorite(this)">♥ Save</button>
                                <div class="audio-wave-anim" id="wave-bars">
                                    <span></span><span></span><span></span><span></span><span></span><span></span><span></span><span></span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="tracklist-table">
                        <div class="track-row header-row">
                            <span class="col-num">#</span>
                            <span class="col-title">Title</span>
                            <span class="col-album">Album</span>
                            <span class="col-time">Duration</span>
                        </div>
                        <div class="track-row active-track" onclick="selectTrack(1, 'Midnight Resonance', '3:45')">
                            <span class="col-num">1</span>
                            <span class="col-title"><strong>Midnight Resonance (FLAC 24-bit)</strong><br><small>Acoustic Continuum</small></span>
                            <span class="col-album">Spatial Horizons</span>
                            <span class="col-time">03:45</span>
                        </div>
                        <div class="track-row" onclick="selectTrack(2, 'Sub-Bass Velocity', '4:12')">
                            <span class="col-num">2</span>
                            <span class="col-title"><strong>Sub-Bass Velocity</strong><br><small>K-Beat Matrix</small></span>
                            <span class="col-album">Tokyo Frequency</span>
                            <span class="col-time">04:12</span>
                        </div>
                        <div class="track-row" onclick="selectTrack(3, 'Ethereal Cloud Drift', '2:58')">
                            <span class="col-num">3</span>
                            <span class="col-title"><strong>Ethereal Cloud Drift</strong><br><small>Ambient Sound Collective</small></span>
                            <span class="col-album">Calm Mind</span>
                            <span class="col-time">02:58</span>
                        </div>
                    </div>
                </div>
            </div>

            <div class="sticky-music-dock">
                <div class="now-playing-info">
                    <div class="now-thumb"></div>
                    <div class="now-text">
                        <strong id="now-title">Midnight Resonance</strong>
                        <small id="now-artist">Acoustic Continuum • 24-Bit / 192kHz</small>
                    </div>
                </div>
                <div class="now-controls">
                    <div class="control-buttons">
                        <button class="ctrl-btn" onclick="prevTrack()">⏮</button>
                        <button class="ctrl-btn play-pause-btn" id="master-play-btn" onclick="toggleMusicPlayback(this)">▶</button>
                        <button class="ctrl-btn" onclick="nextTrack()">⏭</button>
                    </div>
                    <div class="now-scrubber">
                        <span class="time-elapsed" id="time-curr">01:14</span>
                        <div class="scrub-line"><div class="scrub-fill" style="width: 32%;"></div></div>
                        <span class="time-total" id="time-dur">03:45</span>
                    </div>
                </div>
                <div class="volume-dock">
                    <span>🔊</span>
                    <input type="range" class="vol-slider" min="0" max="100" value="85" onchange="adjustVolume(this.value)">
                </div>
            </div>
        </section>"##,
        title = title
    )
}

fn render_cms_builder_showcase(_slug: &str, title: &str) -> String {
    format!(
        r##"<section class="archetype-showcase-container" aria-label="CMS Builder Showcase">
            <div class="showcase-header">
                <div class="showcase-title-row">
                    <h3>Headless CMS Visual Canvas & Content Model Studio</h3>
                    <span class="showcase-badge cms-badge">GraphQL / REST Headless API • Live SSR Sync</span>
                </div>
                <p>Modern visual publishing workspace inspired by Strapi, Storyblok, Blume, and Craft CMS.</p>
            </div>

            <div class="cms-studio-layout">
                <div class="cms-tree-panel">
                    <span class="panel-tag">Component Tree</span>
                    <ul class="tree-nodes">
                        <li class="tree-node active">📄 Page: Landing / Home</li>
                        <li class="tree-node child-node">◈ HeroSection (Header Block)</li>
                        <li class="tree-node child-node">◈ FeatureGrid (Bento 4-Column)</li>
                        <li class="tree-node child-node">◈ PricingMatrix (3-Tier)</li>
                        <li class="tree-node child-node">◈ GlobalFooter (CTA Strip)</li>
                        <li class="tree-node">📄 Section: Pricing FAQ</li>
                        <li class="tree-node">📄 Collection: Case Studies</li>
                    </ul>
                </div>

                <div class="cms-canvas-panel">
                    <div class="canvas-viewport-bar">
                        <span class="viewport-label">Live Preview Canvas:</span>
                        <div class="device-toggles">
                            <button class="dev-btn active" onclick="setCanvasDevice('desktop')">Desktop (1440px)</button>
                            <button class="dev-btn" onclick="setCanvasDevice('tablet')">Tablet (768px)</button>
                            <button class="dev-btn" onclick="setCanvasDevice('mobile')">Mobile (375px)</button>
                        </div>
                        <button class="btn-publish-live" onclick="publishCmsDraft(this)">Publish Live Changes</button>
                    </div>

                    <div class="canvas-artboard" id="cms-artboard">
                        <div class="mock-hero-block" contenteditable="true" spellcheck="false">
                            <h2>Crafted Without Compromise</h2>
                            <p>Direct live in-place typography editing with real-time JSON sync and zero build delays.</p>
                        </div>
                        <div class="mock-grid-block">
                            <div class="mock-card">Feature Card 1</div>
                            <div class="mock-card">Feature Card 2</div>
                            <div class="mock-card">Feature Card 3</div>
                        </div>
                    </div>
                </div>

                <div class="cms-inspector-panel">
                    <span class="panel-tag">Block Properties</span>
                    <div class="prop-group">
                        <label>Component Title</label>
                        <input type="text" class="prop-input" value="{title}">
                    </div>
                    <div class="prop-group">
                        <label>Padding Top / Bottom</label>
                        <select class="prop-input"><option>8rem (Editorial Large)</option><option>5rem (Standard)</option></select>
                    </div>
                    <div class="prop-group">
                        <label>Background Treatment</label>
                        <select class="prop-input"><option>Obsidian Dark Mesh</option><option>Pure Stark White</option></select>
                    </div>
                    <div class="prop-group">
                        <label>API Schema Response</label>
                        <pre class="json-preview">{{ "status": 200, "slug": "{title}", "ssr": true }}</pre>
                    </div>
                </div>
            </div>
        </section>"##,
        title = title
    )
}

fn render_portfolio_showcase(_slug: &str, title: &str) -> String {
    format!(
        r##"<section class="archetype-showcase-container" aria-label="Portfolio Showcase">
            <div class="showcase-header">
                <div class="showcase-title-row">
                    <h3>Awwwards Creative Portfolio & Motion Engineering</h3>
                    <span class="showcase-badge portfolio-badge">Site of the Day • 60 FPS Canvas Physics</span>
                </div>
                <p>Award-grade creative studio experience inspired by Bruno Simon, Anton & Irene, Adham Dannaway, and Cassie Codes.</p>
            </div>

            <div class="portfolio-interactive-frame">
                <div class="physics-canvas-stage" id="physics-stage" onmousemove="trackMouseParticles(event)">
                    <div class="canvas-particles" id="particle-emitter"></div>
                    <div class="stage-central-copy">
                        <span class="hero-kicker">DESIGN ENGINEER & SYSTEMS ARCHITECT</span>
                        <h2 class="stage-name">{title}</h2>
                        <p class="stage-bio">Bridging mathematical physics simulations, pure Rust backends, and award-grade micro-interactions.</p>
                        <div class="discipline-toggle">
                            <button class="disc-btn active" id="btn-designer" onclick="toggleDiscipline('designer')">Designer Perspective</button>
                            <button class="disc-btn" id="btn-engineer" onclick="toggleDiscipline('engineer')">Systems Engineer Perspective</button>
                        </div>
                    </div>
                </div>

                <div class="portfolio-works-grid">
                    <div class="work-item" onclick="openWorkModal('Kinetic Autonomous Vehicle')">
                        <div class="work-thumb work-1"></div>
                        <div class="work-caption">
                            <h4>Kinetic Autonomous Vehicle</h4>
                            <p>Real-Time Telemetry & Rapier3D Collision Simulation</p>
                        </div>
                    </div>
                    <div class="work-item" onclick="openWorkModal('High-Density Trading Terminal')">
                        <div class="work-thumb work-2"></div>
                        <div class="work-caption">
                            <h4>High-Density Trading Terminal</h4>
                            <p>Microsecond WebSocket Streaming & WebGL Orderbook</p>
                        </div>
                    </div>
                    <div class="work-item" onclick="openWorkModal('Spatial Room Configurator')">
                        <div class="work-thumb work-3"></div>
                        <div class="work-caption">
                            <h4>Spatial Room Configurator</h4>
                            <p>WebXR Augmented Reality & PBR Texture Shaders</p>
                        </div>
                    </div>
                </div>
            </div>
        </section>"##,
        title = title
    )
}

fn render_industrial_showcase(_slug: &str, title: &str) -> String {
    format!(
        r##"<section class="archetype-showcase-container" aria-label="Industrial Showcase">
            <div class="showcase-header">
                <div class="showcase-title-row">
                    <h3>Precision Industrial Machinery & SCADA Command</h3>
                    <span class="showcase-badge ind-badge">IEC 61131-3 • ISO 9001 / 14001 Compliant</span>
                </div>
                <p>Heavy engineering platform inspired by Hyundai Heavy Industries, Trumpf Laser Photonics, and KUKA Robotics.</p>
            </div>

            <div class="industrial-command-layout">
                <div class="telemetry-meters-col">
                    <div class="meter-card">
                        <span class="meter-label">SPINDLE VELOCITY</span>
                        <span class="meter-value" id="val-rpm">18,450 <small>RPM</small></span>
                        <div class="meter-bar"><div class="meter-fill" style="width: 82%;"></div></div>
                    </div>
                    <div class="meter-card">
                        <span class="meter-label">LASER EMISSION POWER</span>
                        <span class="meter-value" id="val-kw">19.8 <small>kW</small></span>
                        <div class="meter-bar"><div class="meter-fill" style="width: 95%;"></div></div>
                    </div>
                    <div class="meter-card">
                        <span class="meter-label">CHAMBER VACUUM</span>
                        <span class="meter-value">1.2 × 10⁻⁵ <small>mbar</small></span>
                        <div class="meter-bar"><div class="meter-fill" style="width: 98%;"></div></div>
                    </div>
                    <div class="meter-card">
                        <span class="meter-label">CORE TEMPERATURE</span>
                        <span class="meter-value">48.2 <small>°C (Nominal)</small></span>
                        <div class="meter-bar"><div class="meter-fill" style="width: 45%;"></div></div>
                    </div>
                </div>

                <div class="cad-blueprint-col">
                    <div class="cad-viewport">
                        <div class="cad-grid-lines"></div>
                        <div class="cad-spec-label">CAD DWG REF #TRX-9942 // SHEET METAL CNC</div>
                        <div class="cad-axis-overlay">
                            <span>X: +142.50mm</span>
                            <span>Y: -89.20mm</span>
                            <span>Z: +12.00mm</span>
                        </div>
                    </div>
                    <div class="rfq-actions-bar">
                        <button class="btn-rfq-instant" onclick="triggerRfqModal('{title}')">Request Technical Specification & CAD STEP Download</button>
                        <span class="iso-seal">ISO 9001:2026 CERTIFIED</span>
                    </div>
                </div>
            </div>
        </section>"##,
        title = title
    )
}

fn render_dtc_eshop_showcase(_slug: &str, title: &str) -> String {
    format!(
        r##"<section class="archetype-showcase-container" aria-label="DTC eShop Showcase">
            <div class="showcase-header">
                <div class="showcase-title-row">
                    <h3>Direct-to-Consumer Flagship Storefront</h3>
                    <span class="showcase-badge eshop-badge">Sub-10ms Bag Slide-Over • Single-Source Harvest</span>
                </div>
                <p>Awwwards-grade DTC pantry and lifestyle store inspired by Graza, Caraway, Snif, Diaspora Co, and Wild One.</p>
            </div>

            <div class="dtc-product-layout">
                <div class="dtc-gallery-col">
                    <div class="dtc-hero-image">
                        <div class="badge-freshness">OCTOBER 2026 HARVEST</div>
                    </div>
                    <div class="dtc-thumbs">
                        <div class="d-thumb active">Front Bottle</div>
                        <div class="d-thumb">Squeeze Action</div>
                        <div class="d-thumb">Sensory Oil Drizzle</div>
                        <div class="d-thumb">Packaging Tin</div>
                    </div>
                </div>

                <div class="dtc-buy-col">
                    <span class="dtc-brand-kicker">SMALL-BATCH ARTISANAL HARVEST</span>
                    <h2 class="dtc-prod-title">{title}</h2>
                    <div class="dtc-stars-row">
                        <span class="stars">★★★★★</span>
                        <span class="review-count">4.9 / 5 (3,410 Reviews)</span>
                    </div>
                    <p class="dtc-tagline">100% Spanish Picual single-source harvest in an easy-squeeze bottle designed for high-heat searing and finishing drizzles.</p>
                    
                    <div class="variant-picker-section">
                        <span class="var-title">Select Squeeze Format:</span>
                        <div class="format-options">
                            <button class="format-card active" onclick="selectFormat(this, 'Duo Set', '$37.00')">
                                <strong>Duo Set (Drizzle + Sizzle)</strong>
                                <span>$37.00 • Save 10%</span>
                            </button>
                            <button class="format-card" onclick="selectFormat(this, 'Single Drizzle', '$20.00')">
                                <strong>Single Drizzle Bottle</strong>
                                <span>$20.00 (500ml)</span>
                            </button>
                        </div>
                    </div>

                    <div class="dtc-purchase-row">
                        <div class="qty-counter">
                            <button onclick="decrementQty()">-</button>
                            <span id="dtc-qty">1</span>
                            <button onclick="incrementQty()">+</button>
                        </div>
                        <button class="btn-dtc-add" onclick="addToBag('{title}')">Add to Bag • <span id="dtc-price-disp">$37.00</span></button>
                    </div>

                    <div class="shipping-guarantee-pill">
                        <span>🚚 Free 2-Day Carbon-Neutral Shipping on orders over $35</span>
                    </div>
                </div>
            </div>
        </section>"##,
        title = title
    )
}

fn render_3d_spatial_showcase(_slug: &str, title: &str) -> String {
    format!(
        r##"<section class="archetype-showcase-container" aria-label="3D Spatial Showcase">
            <div class="showcase-header">
                <div class="showcase-title-row">
                    <h3>Interactive 3D WebGL Spatial Product Viewport</h3>
                    <span class="showcase-badge spatial-badge">PBR Materials • Real-Time Light Ray Orbit</span>
                </div>
                <p>Physically based spatial visualizer inspired by Porsche Car Configurator, IKEA Kreativ, and Nike 3D.</p>
            </div>

            <div class="spatial-viewport-layout">
                <div class="webgl-canvas-box" id="webgl-stage" onmousedown="startOrbit(event)" onmousemove="doOrbit(event)" onmouseup="endOrbit()">
                    <div class="webgl-hud-tags">
                        <span>CAMERA: ROTATION 360°</span>
                        <span>SHADING: PHYSICALLY BASED PBR</span>
                        <span id="wireframe-status">WIREFRAME: OFF</span>
                    </div>

                    <div class="interactive-3d-object" id="spatial-model-3d">
                        <div class="model-car-silhouette"></div>
                    </div>

                    <div class="camera-angle-dock">
                        <button class="cam-btn active" onclick="setCameraAngle('front')">Front 3/4</button>
                        <button class="cam-btn" onclick="setCameraAngle('side')">Profile</button>
                        <button class="cam-btn" onclick="setCameraAngle('interior')">Interior Cockpit</button>
                        <button class="cam-btn" onclick="setCameraAngle('aero')">Aerodynamics</button>
                    </div>
                </div>

                <div class="spatial-controls-box">
                    <h4>PBR Material & Finish Customizer</h4>
                    <p class="ctrl-sub">Select procedural paint flake clearcoats, forged alloys, and interior leather tones.</p>
                    
                    <div class="swatch-group">
                        <label>Exterior Paint Tone:</label>
                        <div class="swatches-row">
                            <span class="swatch sw-red active" onclick="set3dColor('#dc2626', 'Carmine Red', this)"></span>
                            <span class="swatch sw-blue" onclick="set3dColor('#2563eb', 'Shark Blue Metallic', this)"></span>
                            <span class="swatch sw-yellow" onclick="set3dColor('#eab308', 'Racing Yellow', this)"></span>
                            <span class="swatch sw-black" onclick="set3dColor('#09090b', 'Jet Black Metallic', this)"></span>
                            <span class="swatch sw-silver" onclick="set3dColor('#e2e8f0', 'GT Silver Flake', this)"></span>
                        </div>
                        <span class="selected-color-name" id="color-name-disp">Carmine Red Metallic</span>
                    </div>

                    <div class="swatch-group">
                        <label>Wheel & Brake Caliper:</label>
                        <div class="swatches-row">
                            <span class="swatch sw-carbon active" onclick="set3dWheel('Forged Carbon', this)"></span>
                            <span class="swatch sw-gold" onclick="set3dWheel('Neodyme Gold', this)"></span>
                            <span class="swatch sw-silver" onclick="set3dWheel('Brushed Titanium', this)"></span>
                        </div>
                    </div>

                    <div class="view-toggles-row">
                        <button class="toggle-btn" onclick="toggleWireframe()">Toggle Wireframe Mesh</button>
                        <button class="toggle-btn" onclick="toggleStudioLighting()">Switch Studio HDRI</button>
                    </div>

                    <div class="export-spec-box">
                        <button class="btn-export-gltf" onclick="exportGltfSpec('{title}')">Export 3D glTF 2.0 Asset Bundle</button>
                    </div>
                </div>
            </div>
        </section>"##,
        title = title
    )
}
