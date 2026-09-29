pub mod ai_oauth;
pub mod cache;
pub mod cli;
pub mod live;
pub mod mail_queue;
pub mod middleware;
pub mod queue;
pub mod quic_h3_stream;
pub mod rate_limit;
pub mod routes;
pub mod server;
pub mod tenant;
pub mod v2_routes;

pub use ai_oauth::{
    AiOAuthService, Claims, OAuthError, TokenRequest, TokenResponse, oauth_token_handler,
};
pub use cache::{CachedFiscalYear, CachedPricingRule, TenantMemoryCache};
pub use cli::{BenchmarkArgs, Cli, Commands, MigrateArgs, StartArgs, TenantArgs, TenantCommands};
pub use live::{live_query, live_ws_handler};
pub use mail_queue::{EmailMessage, MailQueueError, MailQueueManager};
pub use queue::{
    BackgroundJob, PriorityLevel, PriorityTaskDispatcher, QueueError, ReportDownloadJob,
    StaggeredMaintenanceScheduler,
};
pub use quic_h3_stream::{
    H3Frame, H3FrameType, H3Settings, QpackCodec, QpackField, QuicConnectionId, QuicH3Error,
    QuicH3StreamingEngine, QuicStreamType, VarInt,
};
pub use rate_limit::{RateLimitMiddleware, TokenBucketRateLimiter};
pub use routes::{create_resource, delete_resource, get_resource, list_resource};
pub use server::{configure_app, run_server, run_server_with_config};
pub use tenant::{
    AcmeGateway, ConnectionPoolManager, MicroTopologyConfig, TenantContext, TenantError, TenantId,
    TenantResolver, parse_tenant_id, provision_tenant, resolve_scoped_session,
};
pub use v2_routes::{
    LoginPayload, PingResponse, UploadFilePayload, V2ListQuery, compile_filters_to_surrealql,
    download_file_handler, h3_stream_file_handler, h3_stream_telemetry_handler, login_handler,
    logout_handler, ping_handler, quic_status_handler, upload_file_handler, v2_amend_document,
    v2_cancel_document, v2_create_document, v2_delete_document, v2_get_document, v2_list_document,
    v2_submit_document, v2_update_document,
};
