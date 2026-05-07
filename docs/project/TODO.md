# Wireframe-AI TODO

## Adapter Module (adapter/rust/src/main.rs)

### Completed (Code Review - 2025-05-06)
- [x] Fix command injection vulnerability in shell_exec
- [x] Fix path traversal vulnerability in file operations
- [x] Add input validation for all user-provided inputs
- [x] Add path validation helper functions
- [x] Add command allowlist for shell_exec
- [x] Refactor process_job into smaller functions
- [x] Replace magic strings with enum for tool names
- [x] Improve token estimation with proper tokenizer

### Remaining TODOs from Code Review

#### P1 - High Priority
- [ ] Add comprehensive unit tests for adapter module
  - Test tool execution functions
  - Test provider selection logic
  - Test session management
  - Test path validation
  - Test command validation
- [ ] Add integration tests for adapter module
  - Test end-to-end message flow
  - Test tool execution in different modes
  - Test selfdev compilation/restart
  - Test NATS integration

#### P2 - Standard Priority
- [x] Track actual tool execution duration
  - Implemented with tokio::time::Instant::now() and elapsed() in process_job
- [x] Implement MCP stdio integration for sandbox mode
  - Added McpStdioClient with JSON-RPC 2.0 over stdio
  - Lazy-initialized sandbox client in AdapterState
  - Maps shell_exec, file_read, file_write, file_list to MCP tools/call
- [x] Add session management strategy documentation
  - Documented correlation_id, correlation_parent, session_id design
  - Explained correlation_parent-as-session-id decision in README

#### P3 - Low Priority
- [x] Consider platform-specific shell detection
  - Added detect_platform_shell() helper with prioritized detection:
    1. WIREFRAME_AI_SHELL environment variable override
    2. COMSPEC on Windows (with PowerShell vs cmd flag detection)
    3. SHELL on Unix
    4. Platform default fallback (cmd/sh)
  - Added unit tests for shell detection
- [x] Add selfdev compilation safety checks
  - run_safety_checks() runs cargo check, cargo test, cargo clippy -D warnings
  - CompileSelf and RestartSelf run checks by default with skip override

## Other Modules

### Kernel Interface Module

#### Code Review Findings (2025-05-06)

##### P0 - Critical Security Issues
- [x] Add input validation for user_input
  - Added validate_user_input function with length validation (max 10000 chars)
  - Prevents empty input and excessively long input
  - Applied in get_user_input function
- [x] Add validation for session_id from CLI argument
  - Added validate_session_id function with length and character restrictions
  - Only allows alphanumeric, hyphens, underscores, and colons (for UUIDs)
  - Applied before publishing task
- [x] Add validation for timeout_secs from CLI argument
  - Added validate_timeout_secs function with min/max bounds (1-3600 seconds)
  - Prevents invalid timeout values
  - Applied in load_and_validate_config function
- [x] Add validation for nats_url from CLI argument
  - Added validate_nats_url function with basic URL validation
  - Must start with nats:// or contain a valid format with port
  - Applied in load_and_validate_config function

##### P1 - High Priority
- [x] Refactor main function into smaller functions
  - Extracted setup_logging function for logging initialization
  - Extracted load_and_validate_config function for config loading and validation
  - Extracted setup_nats_connection function for NATS setup
  - Extracted setup_shutdown_handler function for graceful shutdown
  - Extracted get_user_input function for user input collection
  - Extracted publish_task function for task publishing
  - Extracted wait_for_result function for result waiting
  - Main function now ~45 lines, much more maintainable
- [x] Replace magic numbers with constants
  - Added MAX_USER_INPUT_LENGTH constant (10000)
  - Added MAX_SESSION_ID_LENGTH constant (256)
  - Added MIN_TIMEOUT_SECS constant (1)
  - Added MAX_TIMEOUT_SECS constant (3600)
  - Added SPINNER_TICK_MS constant (200)
  - Added PREVIEW_LENGTH constant (60)
- [x] Remove unnecessary #[allow(dead_code)] attribute
  - Removed #[allow(dead_code)] from color constants module
  - Removed unused BLUE constant
- [x] Add schema-validation feature to Cargo.toml
  - Added [features] section with schema-validation feature
  - Eliminates compiler warning about unexpected cfg condition

##### P2 - Standard Priority
- [x] Add unit tests for validation functions
  - Added test_validate_user_input_valid, test_validate_user_input_empty
  - Added test_validate_user_input_too_long
  - Added test_validate_session_id_valid, test_validate_session_id_empty
  - Added test_validate_session_id_invalid_chars, test_validate_session_id_too_long
  - Added test_validate_timeout_secs_valid, test_validate_timeout_secs_too_low
  - Added test_validate_timeout_secs_too_high
  - Added test_validate_nats_url_valid, test_validate_nats_url_empty
  - Added test_validate_nats_url_invalid
  - All 13 unit tests passing

### Context Module

#### Code Review Findings (2025-05-06)

##### P0 - Critical Security Issues
- [x] Fix FTS5 query injection risk in search_memory (lines 180-264)
  - Implemented strict allowlist validation (alphanumeric, spaces, hyphens only)
  - Removed apostrophes to prevent FTS5 phrase injection
  - Added validate_fts5_query function with comprehensive validation
- [x] Add input validation for session_id
  - Added validate_session_id function with length and character restrictions
  - Only allows alphanumeric, hyphens, and underscores
  - Applied to all database operations using session_id
- [x] Add input validation for user_input
  - Added validate_user_input function with length validation
  - Sanitizes input to remove control characters
  - Applied in process_task_submitted function
- [x] Add database path validation
  - Added validate_database_path function with canonical path resolution
  - Supports WIREFRAME_AI_ALLOWED_DB_DIR environment variable for path restrictions
  - Applied in main function before database initialization

##### P1 - High Priority
- [x] Refactor main function (lines 308-547)
  - Extracted setup_nats_connection function for NATS setup
  - Extracted handle_task_complete function for task.complete handling
  - Extracted process_task_submitted function for task.submitted processing
  - Extracted filter_safe_env_vars function for environment variable filtering
  - Main function now ~70 lines, much more maintainable
- [x] Add unit tests for search_memory function
  - Added test_validate_fts5_query_valid
  - Added test_validate_fts5_query_invalid
  - Added test_validate_fts5_query_too_long
  - Added test_validate_fts5_query_removes_dangerous_chars
- [x] Add unit tests for build_fts5_query function
  - Added test_build_fts5_query_construction
  - Added test_build_fts5_query_empty
- [x] Add unit tests for environment variable filtering
  - Added test_filter_safe_env_vars_filters_secrets
  - Added test_filter_safe_env_vars_prefix_suffix_matching
  - Improved filtering to use suffix/prefix matching instead of substring matching

##### P2 - Standard Priority
- [x] Replace magic numbers with constants
  - Added MAX_QUERY_LENGTH constant (1000)
  - Added MAX_SESSION_ID_LENGTH constant (256)
  - Added MAX_USER_INPUT_LENGTH constant (10000)
  - Added FTS5_RELEVANCE_SCORE_MAX constant (100.0)
- [x] Improve environment variable filtering
  - Changed from substring matching to suffix/prefix matching
  - Reduces false positives (e.g., "API_ENDPOINT" no longer filtered)
  - Added exact secret matching for common secret names
- [x] Extract common database initialization logic
  - Created DATABASE_SCHEMA constant with SQL schema
  - Shared between main.rs and context_integration_test.rs
  - Eliminates code duplication

##### Additional Improvements
- Added sanitize_string function to remove control characters from all inputs
- Added comprehensive unit tests for all validation functions
- Added test_validate_session_id_valid, test_validate_session_id_invalid, test_validate_session_id_too_long
- Added test_validate_user_input_valid, test_validate_user_input_too_long
- Added test_sanitize_string_removes_control_chars, test_sanitize_string_preserves_whitespace

##### Existing TODOs
- [x] Add integration tests for FTS5 search
- [x] Add tests for memory persistence

### Orchestrator Module

#### Code Review Findings (2025-05-06)

##### P0 - Critical Security Issues
- [x] Fix undefined CONCURRENCY_N reference (line 309)
  - Changed to use the `concurrency` variable instead of undefined constant
- [x] Add input validation for session_id
  - Added validate_session_id function with length and character restrictions
  - Only allows alphanumeric, hyphens, and underscores
  - Applied to all task processing
- [x] Add input validation for correlation_id
  - Added validate_correlation_id function with length and character restrictions
  - Only allows alphanumeric, hyphens, and underscores
  - Applied to all task processing
- [x] Add validation for concurrency value from config
  - Added validate_concurrency function with min/max bounds (1-10)
  - Applied at startup to prevent invalid configuration

##### P1 - High Priority
- [x] Refactor main function into smaller functions
  - Extracted setup_nats_connection function for NATS setup
  - Extracted setup_shutdown_handler function for graceful shutdown
  - Extracted fan_out_jobs function for job dispatch
  - Extracted fan_in_results function for result collection
  - Extracted synthesize_task_complete function for result synthesis
  - Main function now ~100 lines, much more maintainable
- [x] Add unit tests for core orchestration logic
  - Added test_validate_session_id_valid, test_validate_session_id_invalid_chars
  - Added test_validate_session_id_too_long, test_validate_session_id_empty
  - Added test_validate_correlation_id_valid, test_validate_correlation_id_invalid_chars
  - Added test_validate_correlation_id_too_long, test_validate_correlation_id_empty
  - Added test_validate_user_input_valid, test_validate_user_input_too_long
  - Added test_validate_concurrency_valid, test_validate_concurrency_too_low
  - Added test_validate_concurrency_too_high
- [x] Replace magic numbers with constants
  - Added MAX_SESSION_ID_LENGTH constant (256)
  - Added MAX_CORRELATION_ID_LENGTH constant (256)
  - Added MAX_USER_INPUT_LENGTH constant (10000)
  - Added MIN_CONCURRENCY constant (1)
  - Added MAX_CONCURRENCY constant (10)
- [x] Fix resource leak in result subscription handling
  - Added proper error handling for result subscription
  - Improved error logging for subscription failures

##### P2 - Standard Priority
- [x] Add length validation for user_input
  - Added validate_user_input function with length validation
  - Applied to all task processing
- [x] Improve error handling for critical failures
  - Added error handling for result subscription failures
  - Improved error logging throughout the module
- [x] Fix magic number in integration test (620 seconds)
  - Changed timeout from 620 seconds to 15 seconds for faster testing
  - Fixed integration test compilation errors (PathBuf, UsageMetrics, panic macro)

##### Existing TODOs
- [x] Add integration tests for fan-out/fan-in
- [x] Add tests for correlation tracking
- [x] Add tests for timeout handling

## Configuration
- [x] Document environment variables in README
  - WIREFRAME_AI_NATS_URL
  - WIREFRAME_AI_EXECUTION_MODE
  - WIREFRAME_AI_SELFDEV
  - WIREFRAME_AI_SOURCE_ROOT
  - WIREFRAME_AI_ALLOWED_BASE_DIR
  - OPENAI_API_KEY
  - DEEPSEEK_API_KEY
  - OPENCODE_GO_API_KEY

## Documentation
- [x] Update AGENTS.md with adapter security best practices (covered in code review and unit tests)
- [x] Add security documentation for selfdev feature (covered in README security sections)
- [x] Document command allowlist expansion process

## Milestone 2: Provider Ecosystem (In Progress)

### Completed
- [x] Add cost-tracking data structures to provider-core (`UsageCost`, `ProviderCostTracker`, `CostTracker`)
- [x] Implement Anthropic Claude provider (`providers/anthropic/`)
  - Messages API with non-streaming support
  - Tool use support
  - Cost tracking per model
  - 8 unit tests passing
- [x] Implement Google Gemini provider (`providers/google/`)
  - generateContent API with system instruction support
  - Function declaration tools
  - Cost tracking per model
  - 7 unit tests passing
- [x] Implement Cohere provider (`providers/cohere/`)
  - Chat API with preamble/chat_history format
  - Tool use support
  - Cost tracking per model
  - 7 unit tests passing
- [x] Implement Ollama local model provider (`providers/ollama/`)
  - OpenAI-compatible API (default port 11434)
  - Tool use support
  - Health check custom method
  - Zero cost tracking (local inference)
  - 7 unit tests passing
- [x] Register all 4 new providers in workspace `Cargo.toml`

### Remaining
- [ ] Provider discovery/registration system
- [ ] Provider marketplace infrastructure
- [ ] Provider testing framework (integration tests)
- [ ] Capability negotiation between providers
- [ ] Streaming support for Anthropic, Google, Cohere, Ollama
- [ ] Provider fallback and routing logic
- [ ] Unified provider configuration schema
