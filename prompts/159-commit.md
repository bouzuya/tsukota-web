Add tracing-based logging to backend

- Initialize tracing-subscriber with env-filter in main (default: info level)
- Add startup logs (environment loaded, initialization complete, server starting)
- Add HTTP request/response logging via tower-http TraceLayer
- Add #[tracing::instrument] to all 18 use case execute methods
- Replace existing println!/eprintln! with tracing macros
- Skip sensitive fields (device_secret, session_token) from trace output
