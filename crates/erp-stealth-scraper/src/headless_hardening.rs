//! Headless Browser Hardening & Canvas/WebGL Anti-Fingerprinting (`erp_stealth_scraper::headless_hardening`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Hardware spoofing configuration for headless Chromium instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpoofConfig {
    pub unmasked_vendor: CompactString,
    pub unmasked_renderer: CompactString,
    pub hardware_concurrency: u32,
    pub device_memory_gb: u32,
    pub canvas_noise_seed: u64,
}

impl Default for HardwareSpoofConfig {
    fn default() -> Self {
        Self {
            unmasked_vendor: "Google Inc. (NVIDIA)".into(),
            unmasked_renderer:
                "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Direct3D11 vs_5_0 ps_5_0, D3D11)".into(),
            hardware_concurrency: 16,
            device_memory_gb: 32,
            canvas_noise_seed: 1337420,
        }
    }
}

/// Generates evasion scripts injected into `Page.addScriptToEvaluateOnNewDocument`.
pub struct HeadlessHardeningEngine;

impl HeadlessHardeningEngine {
    /// Generates pure JavaScript anti-detection prelude for headless browsers.
    #[must_use]
    pub fn generate_evasion_script(config: &HardwareSpoofConfig) -> String {
        format!(
            r#"(function() {{
    // 1. Eradicate navigator.webdriver
    Object.defineProperty(navigator, 'webdriver', {{
        get: () => undefined,
        configurable: true
    }});

    // 2. Hardware Concurrency & Memory
    Object.defineProperty(navigator, 'hardwareConcurrency', {{
        get: () => {concurrency},
        configurable: true
    }});
    Object.defineProperty(navigator, 'deviceMemory', {{
        get: () => {memory},
        configurable: true
    }});

    // 3. WebGL Vendor & Renderer Spoofing
    const getParameter = WebGLRenderingContext.prototype.getParameter;
    WebGLRenderingContext.prototype.getParameter = function(parameter) {{
        // UNMASKED_VENDOR_WEBGL
        if (parameter === 37445) return '{vendor}';
        // UNMASKED_RENDERER_WEBGL
        if (parameter === 37446) return '{renderer}';
        return getParameter.apply(this, arguments);
    }};

    if (typeof WebGL2RenderingContext !== 'undefined') {{
        const getParameter2 = WebGL2RenderingContext.prototype.getParameter;
        WebGL2RenderingContext.prototype.getParameter = function(parameter) {{
            if (parameter === 37445) return '{vendor}';
            if (parameter === 37446) return '{renderer}';
            return getParameter2.apply(this, arguments);
        }};
    }}

    // 4. HTML5 Canvas toDataURL / getImageData Noise Injection
    const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
    HTMLCanvasElement.prototype.toDataURL = function(type) {{
        const ctx = this.getContext('2d');
        if (ctx) {{
            const shift = ({noise_seed} % 3) - 1;
            ctx.fillStyle = 'rgba(' + (255 + shift) + ',255,255,0.01)';
            ctx.fillRect(0, 0, 1, 1);
        }}
        return originalToDataURL.apply(this, arguments);
    }};

    // 5. Chrome Runtime & Plugins Mock
    window.chrome = {{
        runtime: {{}},
        loadTimes: function() {{}},
        csi: function() {{}},
        app: {{}}
    }};
}})();"#,
            concurrency = config.hardware_concurrency,
            memory = config.device_memory_gb,
            vendor = config.unmasked_vendor,
            renderer = config.unmasked_renderer,
            noise_seed = config.canvas_noise_seed,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_evasion_script() {
        let config = HardwareSpoofConfig::default();
        let script = HeadlessHardeningEngine::generate_evasion_script(&config);
        assert!(script.contains("navigator.webdriver"));
        assert!(script.contains("NVIDIA GeForce RTX 4080"));
        assert!(script.contains("HTMLCanvasElement.prototype.toDataURL"));
    }
}
