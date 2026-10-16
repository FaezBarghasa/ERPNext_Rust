//! Phase 9 Golden Performance & SLA Regression Verification Tests (`tests/golden_perf_tests.rs`).
//!
//! Validates the strict performance mandates:
//! - Document get (cached): p99 < 1 ms
//! - Sales Invoice submit (20 lines, GL+SLE): p99 < 15 ms; > 3k submits/s on 8 cores
//! - List view 100 rows with filters: p99 < 10 ms
//! - General Ledger 1M rows: < 1 s first page, streaming
//! - Stock reposting 1M SLE: < 30 s (parallel per item-warehouse)
//! - Cold start: < 300 ms, RSS < 150 MB idle
//! - CI performance regression gate (>5% regression fails)

use rbench::perf_engine::PerfEngine;

#[test]
fn test_sla1_document_get_cached() {
    let result = PerfEngine::bench_document_get(10_000);
    println!("SLA 1: {} -> {}", result.name, result.measured);
    assert!(
        result.passed,
        "Document get SLA failed: target {}, measured: {}",
        result.target, result.measured
    );
    assert!(result.p99_ms < 1.0, "p99 must be strictly < 1.0 ms");
}

#[tokio::test]
async fn test_sla2_sales_invoice_submit_throughput() {
    let result = PerfEngine::bench_sales_invoice_submit(10_000, 8).await;
    println!("SLA 2: {} -> {}", result.name, result.measured);
    assert!(
        result.passed,
        "Sales invoice submit SLA failed: target {}, measured: {}",
        result.target, result.measured
    );
    assert!(result.p99_ms < 15.0, "p99 submit latency must be < 15.0 ms");
    assert!(
        result.throughput_per_sec >= 3000.0,
        "Submit throughput must exceed 3,000 submits/sec on 8 cores"
    );
}

#[test]
fn test_sla3_list_view_100_rows_filters() {
    let result = PerfEngine::bench_list_view_filters(1_000);
    println!("SLA 3: {} -> {}", result.name, result.measured);
    assert!(
        result.passed,
        "List view query SLA failed: target {}, measured: {}",
        result.target, result.measured
    );
    assert!(
        result.p99_ms < 10.0,
        "p99 list view query latency must be < 10.0 ms"
    );
}

#[test]
fn test_sla4_general_ledger_1m_rows_streaming() {
    let result = PerfEngine::bench_general_ledger_1m_streaming();
    println!("SLA 4: {} -> {}", result.name, result.measured);
    assert!(
        result.passed,
        "Streaming GL SLA failed: target {}, measured: {}",
        result.target, result.measured
    );
    assert!(
        result.p99_ms < 1000.0,
        "First page time must be strictly under 1000 ms (1.0 s)"
    );
}

#[test]
fn test_sla5_stock_reposting_1m_sle_parallel() {
    let result = PerfEngine::bench_stock_reposting_1m_parallel();
    println!("SLA 5: {} -> {}", result.name, result.measured);
    assert!(
        result.passed,
        "Stock reposting SLA failed: target {}, measured: {}",
        result.target, result.measured
    );
    assert!(
        result.p99_ms < 30_000.0,
        "1M SLE parallel reposting must complete in < 30.0 seconds"
    );
}

#[test]
fn test_sla6_cold_start_and_rss_idle() {
    let result = PerfEngine::bench_cold_start();
    println!("SLA 6: {} -> {}", result.name, result.measured);
    assert!(
        result.passed,
        "Cold start SLA failed: target {}, measured: {}",
        result.target, result.measured
    );
    assert!(result.p99_ms < 300.0, "Cold start must be < 300 ms");
}

#[tokio::test]
async fn test_perf_regression_gate_full_pass() {
    let report = PerfEngine::run_regression_gates()
        .await
        .expect("Full regression gate suite must pass");

    assert!(report.all_passed);
    assert!(report.doc_get_sla.passed);
    assert!(report.sales_invoice_submit_sla.passed);
    assert!(report.list_view_filter_sla.passed);
    assert!(report.streaming_gl_sla.passed);
    assert!(report.parallel_reposting_sla.passed);
    assert!(report.cold_start_sla.passed);
}
