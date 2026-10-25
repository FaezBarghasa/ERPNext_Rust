import sys

peacock = '''        TemplateSuite {
            slug: "stream-peacock-crest".into(),
            title: "PeacockCrest Primetime & Live Sports Network".into(),
            subtitle: "Universal studio catalog, next-day broadcast TV catchup, Premier League live streams & multi-tier AVOD/SVOD".into(),
            class_reference: "PeacockCrest Network Class".into(),
            work_type: WorkTypeClassification::SvodMediaStreaming,
            modules_used: vec!["erp-cms".into(), "erp-software".into(), "erp-trade".into()],
            key_capabilities: vec![
                "Ad-Supported AVOD Dynamic Pre/Mid-Roll SSAI Stitching".into(),
                "Next-Day Primetime Network TV Broadcast Catchup Pipeline".into(),
                "Simultaneous 60FPS Live Sports Feeds with Stadium Camera Angle".into(),
                "Editorial Binge-Watch Skip Intro & Auto-Play Next Hook".into(),
                "Universal Picture-in-Picture Mini Player for Multi-Tasking".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#0284c7".into(),
                color_secondary: "#0369a1".into(),
                color_background: "#020610".into(),
                color_surface: "#071324".into(),
                color_accent: "#38bdf8".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "0.65rem".into(),
                noise_opacity: 0.03,
            },
        },
'''

public_broadcast_tv = '''        TemplateSuite {
            slug: "tv-public-broadcast".into(),
            title: "PublicBroadcast 10-Foot Leanback Free-to-Air Grid".into(),
            subtitle: "Public broadcaster 10-foot TV UI, live regional broadcast guide, closed captions & parental PIN locks".into(),
            class_reference: "PublicBroadcast TV Class".into(),
            work_type: WorkTypeClassification::FieldLogisticsPosKiosk,
            modules_used: vec!["erp-cms".into(), "erp-software".into(), "frappe-storage".into()],
            key_capabilities: vec![
                "7-Day Electronic Program Guide (EPG) Leanback Navigation".into(),
                "D-Pad Remote Accessible Focus Rings & Voice Navigation".into(),
                "Closed Captioning (WebVTT) Subtitle Font Customization".into(),
                "Parental Control Age Rating Gate with 4-Digit Remote PIN".into(),
                "Local Regional Station Feed Auto-Geolocation Matcher".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#14b8a6".into(),
                color_secondary: "#0d9488".into(),
                color_background: "#040e0c".into(),
                color_surface: "#0a1f1b".into(),
                color_accent: "#2dd4bf".into(),
                font_heading: "'Plus Jakarta Sans', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "0.5rem".into(),
                noise_opacity: 0.03,
            },
        },
'''

deep_harpoon = '''        TemplateSuite {
            slug: "shop-deep-harpoon".into(),
            title: "DeepHarpoon Spearfishing & Sushi Bistro Arcade".into(),
            subtitle: "Aquatic day-night diving depth pressure gauges, live catch FIFO inventory, evening sushi menu price matrix & equipment shop".into(),
            class_reference: "DeepHarpoon Nautical Class".into(),
            work_type: WorkTypeClassification::DigitalGoodsCreator,
            modules_used: vec!["erp-trade".into(), "erp-inventory".into(), "erp-software".into()],
            key_capabilities: vec![
                "Day/Night Cycle Diver Oxygen & Pressure Depth Gauge".into(),
                "Spearfished Seafood Freshness Rating & Live FIFO Catch Ledger".into(),
                "Evening Sushi Bar Menu Price Multiplier & Customer Review Meter".into(),
                "Deep-Sea Harpoon & Diving Suit Equipment Upgrade Store".into(),
                "Artisanal Sea Salt & Wasabi Add-On Merchandise Checkout".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#0284c7".into(),
                color_secondary: "#0369a1".into(),
                color_background: "#030c14".into(),
                color_surface: "#071a2a".into(),
                color_accent: "#38bdf8".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "1.0rem".into(),
                noise_opacity: 0.035,
            },
        },
'''

netflix_sub = '''        TemplateSuite {
            slug: "sub-stream-pass".into(),
            title: "StreamPass Cinema & Ultra HD Multi-Device Vault".into(),
            subtitle: "Standard with Ads vs Standard vs Premium tier matrix, spatial audio entitlement & household member sub-accounts".into(),
            class_reference: "StreamPass Membership Class".into(),
            work_type: WorkTypeClassification::DigitalGoodsCreator,
            modules_used: vec!["erp-accounting".into(), "erp-software".into(), "erp-trade".into()],
            key_capabilities: vec![
                "Tier Comparison Matrix (Standard w/ Ads vs Standard vs Premium)".into(),
                "Concurrent Stream Limiter & Download Device Manager".into(),
                "Extra Member Sub-Account Monthly Add-On Billing".into(),
                "4K HDR & Spatial Audio License Entitlement Flags".into(),
                "Instant One-Click Account Pause & Resume Retention Engine".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#e50914".into(),
                color_secondary: "#b81d24".into(),
                color_background: "#0a0607".into(),
                color_surface: "#1a0c0f".into(),
                color_accent: "#ff2a35".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "0.75rem".into(),
                noise_opacity: 0.035,
            },
        },
'''

tidal_master = '''        TemplateSuite {
            slug: "music-tidal-master".into(),
            title: "MasterTidal 24-Bit / 192kHz Audiophile Haven".into(),
            subtitle: "Bit-perfect FLAC/MQA streaming, direct-to-artist royalty payout multiplier & Dolby Atmos binaural spatial audio".into(),
            class_reference: "MasterTidal Audiophile Class".into(),
            work_type: WorkTypeClassification::SvodMediaStreaming,
            modules_used: vec!["erp-cms".into(), "erp-software".into(), "erp-accounting".into()],
            key_capabilities: vec![
                "Bit-Perfect 24-Bit / 192kHz Hi-Res FLAC Streaming Pipeline".into(),
                "Direct-to-Artist Payout Share (10% of Subscription Direct to Top Artist)".into(),
                "Dolby Atmos Binaural Spatial Headphone Rendering".into(),
                "Offline Encrypted Device Cache with Audio Quality Tier Toggle".into(),
                "Master Quality Authenticated (MQA) Hardware DAC Passthrough".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#00ffff".into(),
                color_secondary: "#00b4d8".into(),
                color_background: "#03080c".into(),
                color_surface: "#08131d".into(),
                color_accent: "#90e0ef".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "0.5rem".into(),
                noise_opacity: 0.03,
            },
        },
'''

craft_cms = '''        TemplateSuite {
            slug: "cms-craft-matrix".into(),
            title: "CraftMatrix Multi-Lingual Matrix Content Studio".into(),
            subtitle: "Tailored matrix block field editor, live entry preview pane, asset transform pipeline & multi-site localization".into(),
            class_reference: "CraftMatrix Studio Class".into(),
            work_type: WorkTypeClassification::FieldLogisticsPosKiosk,
            modules_used: vec!["erp-cms".into(), "erp-software".into(), "frappe-meta".into()],
            key_capabilities: vec![
                "Flexible Matrix Block Content Modeler with Drag Sorting".into(),
                "Split-Screen Live Side-by-Side Entry Preview Pane".into(),
                "Focal-Point Preserving Image Transform Engine".into(),
                "Multi-Site Translation & Localization Matrix".into(),
                "Granular Section, Entry & User Permission Hierarchy".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#da5a47".into(),
                color_secondary: "#b84534".into(),
                color_background: "#0f0706".into(),
                color_surface: "#1f100e".into(),
                color_accent: "#f0715f".into(),
                font_heading: "'Plus Jakarta Sans', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "0.65rem".into(),
                noise_opacity: 0.035,
            },
        },
'''

minimal_motion = '''        TemplateSuite {
            slug: "portfolio-minimal-motion".into(),
            title: "MinimalMotion Micro-Interaction & UI Physics Portfolio".into(),
            subtitle: "CSS animation engineering, physics-based gesture demos, design writing journal & stark monochrome precision".into(),
            class_reference: "MinimalMotion Design Class".into(),
            work_type: WorkTypeClassification::LuxuryAtelier,
            modules_used: vec!["erp-cms".into(), "erp-software".into(), "frappe-storage".into()],
            key_capabilities: vec![
                "Physics-Based Spring Animation Interactive Lab".into(),
                "Stark Monochrome Typographic Hierarchy & Spacing Rhythm".into(),
                "Deep Long-Form Technical Design Journal with Live Demos".into(),
                "Fluid CSS Sub-Pixel Transitions with Zero Jank".into(),
                "Lightweight Offline-First Progressive Web Architecture".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#ffffff".into(),
                color_secondary: "#94a3b8".into(),
                color_background: "#080808".into(),
                color_surface: "#121212".into(),
                color_accent: "#e2e8f0".into(),
                font_heading: "'Plus Jakarta Sans', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "0.25rem".into(),
                noise_opacity: 0.02,
            },
        },
'''

schneider_energy = '''        TemplateSuite {
            slug: "ind-energy-schneider".into(),
            title: "EcoStruxure Industrial Energy & Smart Grid Command".into(),
            subtitle: "Microgrid energy telemetry, carbon accounting ledger, IoT switchgear monitoring & industrial power automation".into(),
            class_reference: "EcoStruxure Energy Grid Class".into(),
            work_type: WorkTypeClassification::HeavyIndustrialB2b,
            modules_used: vec!["erp-energy".into(), "erp-manufacturing".into(), "erp-iot".into()],
            key_capabilities: vec![
                "Real-Time Medium/Low Voltage Switchgear IoT Telemetry".into(),
                "Microgrid Solar & Battery Energy Storage Optimization".into(),
                "Scope 1, 2, 3 Carbon Emission Reporting & Net-Zero Ledger".into(),
                "Predictive Transformer Thermal Stress & Arc Flash Detection".into(),
                "Modbus / BACnet / MQTT Industrial Protocol Gateway Ingestion".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#009530".into(),
                color_secondary: "#007a27".into(),
                color_background: "#030a05".into(),
                color_surface: "#08170c".into(),
                color_accent: "#3dd168".into(),
                font_heading: "'Plus Jakarta Sans', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "0.5rem".into(),
                noise_opacity: 0.035,
            },
        },
'''

eshop_extras = '''        TemplateSuite {
            slug: "shop-turkish-textile".into(),
            title: "ArtisanalLoom Slow-Craft Organic Robes & Linen".into(),
            subtitle: "Shuttle loom weave density inspector, zero-waste garment pattern layout & pre-washed Turkish organic cotton".into(),
            class_reference: "ArtisanalLoom Textile Class".into(),
            work_type: WorkTypeClassification::LuxuryAtelier,
            modules_used: vec!["erp-trade".into(), "erp-inventory".into(), "erp-accounting".into()],
            key_capabilities: vec![
                "Traditional Wooden Shuttle Loom Weave Density Visualizer".into(),
                "Zero-Waste Garment Cutting Pattern Efficiency Score".into(),
                "Pre-Washed Turkish Organic Long-Staple Cotton Guarantee".into(),
                "Small-Batch Serial Numbered Production Run Certificates".into(),
                "Plastic-Free Biodegradable Packaging Tracking".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#c2410c".into(),
                color_secondary: "#9a3412".into(),
                color_background: "#120703".into(),
                color_surface: "#241006".into(),
                color_accent: "#ea580c".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "0.5rem".into(),
                noise_opacity: 0.04,
            },
        },
        TemplateSuite {
            slug: "shop-speed-athletics".into(),
            title: "KineticFootwear Innovation Running & Apparel Flagship".into(),
            subtitle: "Carbon-fiber propulsion plate exploded 3D view, marathon pace shoe selector & Member exclusive early drop access".into(),
            class_reference: "KineticFootwear Athletics Class".into(),
            work_type: WorkTypeClassification::HighVelocityRetail,
            modules_used: vec!["erp-trade".into(), "erp-inventory".into(), "erp-wms".into()],
            key_capabilities: vec![
                "Exploded 3D WebGL Carbon-Fiber Energy Return Sole Visualizer".into(),
                "Target Marathon Split Time & Footstrike Footwear Matcher".into(),
                "Member Early Access Drops with Bot-Resistant Queue".into(),
                "High-Speed Mobile Bag Slide-Over (<10ms Open)".into(),
                "In-Store Local Hub Pickup Inventory Reservation".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#84cc16".into(),
                color_secondary: "#65a30d".into(),
                color_background: "#020402".into(),
                color_surface: "#0a1008".into(),
                color_accent: "#a3e635".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "0.25rem".into(),
                noise_opacity: 0.035,
            },
        },
        TemplateSuite {
            slug: "shop-dewy-skin".into(),
            title: "DewySkin Bare-Faced Clean Beauty & Lip Tint Boutique".into(),
            subtitle: "Dewy skin hydration slider, buildable tint shade matcher, clean formulation badges & unboxing micro-animations".into(),
            class_reference: "DewySkin Beauty Class".into(),
            work_type: WorkTypeClassification::LuxuryAtelier,
            modules_used: vec!["erp-trade".into(), "erp-inventory".into(), "erp-accounting".into()],
            key_capabilities: vec![
                "Hydration Level Dewy Finish Interactive Before/After Slider".into(),
                "Virtual Skin Undertone & Buildable Balm Shade Matcher".into(),
                "Clean Cruelty-Free & Dermatologist-Tested Safety Labels".into(),
                "Custom Bubble Pouch Sticker Set Personalization at Checkout".into(),
                "Real-Time Flash Stock Alerts for Limited Edition Balm Drops".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#f43f5e".into(),
                color_secondary: "#e11d48".into(),
                color_background: "#120609".into(),
                color_surface: "#240e14".into(),
                color_accent: "#fb7185".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "1.25rem".into(),
                noise_opacity: 0.03,
            },
        },
'''

viewer_extras = '''        TemplateSuite {
            slug: "viewer-physics-sandbox-3d".into(),
            title: "PhysicsSandbox 3D Interactive Toy Vehicle Playground".into(),
            subtitle: "Three.js / Cannon-es rigid body vehicle physics, interactive bowling pins, terrain ramps & customizable toy truck".into(),
            class_reference: "PhysicsSandbox Toy World Class".into(),
            work_type: WorkTypeClassification::LuxuryAtelier,
            modules_used: vec!["erp-cms".into(), "erp-software".into(), "frappe-storage".into()],
            key_capabilities: vec![
                "Rigid Body Vehicle Suspension & Wheel Friction Physics Simulation".into(),
                "Interactive Destructible Playground Obstacles (Bricks, Pins, Ramps)".into(),
                "Day/Night Atmospheric Dynamic Shadow & Sun Vector Direction".into(),
                "Custom Toy Truck Paint Colors & Wheel Rim Upgrades".into(),
                "Keyboard WASD / Gamepad / Mobile Virtual Joystick Controls".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#f97316".into(),
                color_secondary: "#ea580c".into(),
                color_background: "#080604".into(),
                color_surface: "#140e08".into(),
                color_accent: "#fb923c".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "1.0rem".into(),
                noise_opacity: 0.035,
            },
        },
        TemplateSuite {
            slug: "viewer-pbr-asset-showcase".into(),
            title: "PBRAssetShowcase Real-Time 3D Model Marketplace".into(),
            subtitle: "Wireframe vs normal vs roughness channel visualizer, glTF 2.0 / USDZ drag loader & HDRI environment switcher".into(),
            class_reference: "PBRAssetShowcase Model Class".into(),
            work_type: WorkTypeClassification::DigitalGoodsCreator,
            modules_used: vec!["erp-cms".into(), "erp-trade".into(), "frappe-storage".into()],
            key_capabilities: vec![
                "Shading Channel Inspector (Albedo, Normal, Roughness, Metalness)".into(),
                "Live Drag-and-Drop glTF 2.0 / USDZ / OBJ Model Upload Viewer".into(),
                "Photorealistic HDRI Studio Lighting Environment Switcher".into(),
                "Polycount, Vertex Count, and Mesh Topology Wireframe Mode".into(),
                "Commercial 3D Asset License Single-Use Download Cryptography".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#f59e0b".into(),
                color_secondary: "#d97706".into(),
                color_background: "#0a0c10".into(),
                color_surface: "#141820".into(),
                color_accent: "#fbbf24".into(),
                font_heading: "'Plus Jakarta Sans', sans-serif".into(),
                font_body: "'Plus Jakarta Sans', sans-serif".into(),
                font_mono: "'JetBrains Mono', monospace".into(),
                border_radius: "0.65rem".into(),
                noise_opacity: 0.035,
            },
        },
'''

with open("crates/erp-cms/src/reference_templates.rs") as f:
    text = f.read()

# Insert peacock before GROUP 2
text = text.replace(
    "        // GROUP 2: SMART TV",
    peacock + "\n        // GROUP 2: SMART TV"
)

# Insert public_broadcast_tv before GROUP 3
text = text.replace(
    "        // GROUP 3: DIGITAL GOODS",
    public_broadcast_tv + "\n        // GROUP 3: DIGITAL GOODS"
)

# Insert deep_harpoon before GROUP 4
text = text.replace(
    "        // GROUP 4: DIGITAL SUBSCRIPTION",
    deep_harpoon + "\n        // GROUP 4: DIGITAL SUBSCRIPTION"
)

# Insert netflix_sub before GROUP 5
text = text.replace(
    "        // GROUP 5: MUSIC STREAMING",
    netflix_sub + "\n        // GROUP 5: MUSIC STREAMING"
)

# Insert tidal_master before GROUP 6
text = text.replace(
    "        // GROUP 6: MODERN CMS",
    tidal_master + "\n        // GROUP 6: MODERN CMS"
)

# Insert craft_cms before GROUP 7
text = text.replace(
    "        // GROUP 7: INTERACTIVE PORTFOLIO",
    craft_cms + "\n        // GROUP 7: INTERACTIVE PORTFOLIO"
)

# Insert minimal_motion before GROUP 8
text = text.replace(
    "        // GROUP 8: INDUSTRIAL BRAND",
    minimal_motion + "\n        // GROUP 8: INDUSTRIAL BRAND"
)

# Insert schneider_energy before GROUP 9
text = text.replace(
    "        // GROUP 9: DIRECT-TO-CONSUMER",
    schneider_energy + "\n        // GROUP 9: DIRECT-TO-CONSUMER"
)

# Insert eshop_extras before GROUP 10
text = text.replace(
    "        // GROUP 10: 3D OBJECT",
    eshop_extras + "\n        // GROUP 10: 3D OBJECT"
)

# Insert viewer_extras before closing ]
text = text.rstrip()
if text.endswith("]"):
    text = text[:-1] + viewer_extras + "    ]\n}"
elif text.endswith("}\n}") or text.endswith("}\n    ]\n}"):
    pos = text.rfind("    ]")
    text = text[:pos] + viewer_extras + text[pos:]

with open("crates/erp-cms/src/reference_templates.rs", "w") as f:
    f.write(text)

print("Updated reference_templates.rs successfully!")
