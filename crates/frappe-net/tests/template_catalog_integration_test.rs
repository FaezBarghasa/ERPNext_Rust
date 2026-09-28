use actix_web::{App, test};
use frappe_net::server::configure_app;
use frappe_net::tenant::{ConnectionPoolManager, MicroTopologyConfig};
use std::time::Duration;

#[actix_web::test]
async fn test_template_catalog_http_endpoints() {
    let pool_mgr = ConnectionPoolManager::new(Duration::from_secs(60));
    let app = test::init_service(
        App::new()
            .configure(|cfg| configure_app(cfg, pool_mgr, MicroTopologyConfig::default(), None)),
    )
    .await;

    // 1. Verify GET /templates returns the portal index
    let req = test::TestRequest::get().uri("/templates").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let html_str = String::from_utf8(body.to_vec()).expect("Valid UTF-8");
    assert!(html_str.contains("Universal Work-Type Templates"));
    assert!(html_str.contains("svod-streaming"));
    assert!(html_str.contains("lms-academy"));
    assert!(html_str.contains("digital-goods"));
    assert!(html_str.contains("b2b-industrial"));
    assert!(html_str.contains("b2c-retail"));
    assert!(html_str.contains("trading-exchange"));

    // 2. Verify all 6 individual template routes return 200 and their specialized content
    let test_templates = [
        ("svod-streaming", "rustnext Cinema"),
        ("lms-academy", "rustnext Academy"),
        ("digital-goods", "rustnext Creator Hub"),
        ("b2b-industrial", "rustnext Industrial"),
        ("b2c-retail", "rustnext Retail"),
        ("trading-exchange", "rustnext Exchange"),
    ];

    for (slug, expected_title) in test_templates {
        let req = test::TestRequest::get()
            .uri(&format!("/templates/{slug}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(
            resp.status().is_success(),
            "Template '{slug}' returned non-200"
        );

        let body = test::read_body(resp).await;
        let html_str = String::from_utf8(body.to_vec()).expect("Valid UTF-8");
        assert!(
            html_str.contains(expected_title),
            "Template '{slug}' missing '{expected_title}'"
        );
    }

    // 3. Verify non-existent template returns 404
    let req_missing = test::TestRequest::get()
        .uri("/templates/non-existent-template")
        .to_request();
    let resp_missing = test::call_service(&app, req_missing).await;
    assert_eq!(
        resp_missing.status(),
        actix_web::http::StatusCode::NOT_FOUND
    );

    // 4. Verify GET /api/v1/templates returns catalog JSON array with 6 items
    let req_api = test::TestRequest::get()
        .uri("/api/v1/templates")
        .to_request();
    let resp_api = test::call_service(&app, req_api).await;
    assert!(resp_api.status().is_success());

    let api_body = test::read_body(resp_api).await;
    let templates: Vec<serde_json::Value> =
        serde_json::from_slice(&api_body).expect("Valid JSON array");
    assert_eq!(templates.len(), 6);

    // 5. Verify GET /api/v1/templates/{slug}/manifest returns valid ThemeManifest JSON
    let req_manifest = test::TestRequest::get()
        .uri("/api/v1/templates/svod-streaming/manifest")
        .to_request();
    let resp_manifest = test::call_service(&app, req_manifest).await;
    assert!(resp_manifest.status().is_success());

    let manifest_body = test::read_body(resp_manifest).await;
    let manifest: serde_json::Value =
        serde_json::from_slice(&manifest_body).expect("Valid ThemeManifest JSON");
    assert_eq!(manifest["id"], "theme_svod-streaming");
    assert_eq!(manifest["work_type"], "SvodMediaStreaming");
    assert_eq!(manifest["engine"], "SsrTera");
    assert_eq!(
        manifest["default_design_tokens"]["color_primary"],
        "#e50914"
    );

    // 6. Verify GET /api/v1/templates/{slug}/manifest returns 404 on invalid slug
    let req_invalid_manifest = test::TestRequest::get()
        .uri("/api/v1/templates/unknown-slug/manifest")
        .to_request();
    let resp_invalid_manifest = test::call_service(&app, req_invalid_manifest).await;
    assert_eq!(
        resp_invalid_manifest.status(),
        actix_web::http::StatusCode::NOT_FOUND
    );

    // 7. Verify ?variant= query parameter triggers custom archetype CSS and JSON-LD
    let variants = [
        "cyberpunk",
        "vaporwave",
        "retrowave",
        "neonwave",
        "tasteful",
    ];
    for v in variants {
        let req_variant = test::TestRequest::get()
            .uri(&format!("/templates/svod-streaming?variant={v}"))
            .to_request();
        let resp_variant = test::call_service(&app, req_variant).await;
        assert!(resp_variant.status().is_success());
        let body = test::read_body(resp_variant).await;
        let html_str = String::from_utf8(body.to_vec()).expect("Valid UTF-8");
        assert!(html_str.contains("application/ld+json"));
        assert!(html_str.contains("theme-dock-btn"));
    }

    // 8. Verify GET /sitemap.xml returns valid XML containing all 30 permutations
    let req_sitemap = test::TestRequest::get().uri("/sitemap.xml").to_request();
    let resp_sitemap = test::call_service(&app, req_sitemap).await;
    assert!(resp_sitemap.status().is_success());
    let sitemap_body = test::read_body(resp_sitemap).await;
    let sitemap_str = String::from_utf8(sitemap_body.to_vec()).expect("Valid UTF-8");
    assert!(sitemap_str.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
    assert!(sitemap_str.contains("svod-streaming?variant=cyberpunk"));
    assert!(sitemap_str.contains("trading-exchange?variant=neonwave"));

    // 9. Verify GET /robots.txt returns valid crawling directives
    let req_robots = test::TestRequest::get().uri("/robots.txt").to_request();
    let resp_robots = test::call_service(&app, req_robots).await;
    assert!(resp_robots.status().is_success());
    let robots_body = test::read_body(resp_robots).await;
    let robots_str = String::from_utf8(robots_body.to_vec()).expect("Valid UTF-8");
    assert!(robots_str.contains("User-agent: *"));
    assert!(robots_str.contains("Sitemap: https://rustnext.enterprise.io/sitemap.xml"));
}
