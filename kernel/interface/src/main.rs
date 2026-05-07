//! wireframe-ai-interface — CLI frontend
//! Reads user input, publishes to NATS, and displays the result.
//! This is the entry point humans interact with. Swappable with a
//! web server or VS Code extension — only the transport changes.

use agentic_sdk::announce_online;
use agentic_sdk::envelope::Envelope;
use agentic_sdk::message_types::{TaskComplete, TaskSubmitted};
use clap::Parser;
use futures::StreamExt;
use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info};
use tracing_subscriber::fmt::format::FmtSpan;
use wireframe_config::WireframeConfig;

// ── Constants ───────────────────────────────────────────────────────────────

const MAX_USER_INPUT_LENGTH: usize = 10000;
const MAX_SESSION_ID_LENGTH: usize = 256;
const MIN_TIMEOUT_SECS: u64 = 1;
const MAX_TIMEOUT_SECS: u64 = 3600;
const SPINNER_TICK_MS: u64 = 200;
const PREVIEW_LENGTH: usize = 60;

// ── Validation functions ─────────────────────────────────────────────────────

fn validate_user_input(input: &str) -> Result<(), String> {
    if input.is_empty() {
        return Err("user_input cannot be empty".to_string());
    }
    if input.len() > MAX_USER_INPUT_LENGTH {
        return Err(format!(
            "user_input exceeds maximum length of {} characters",
            MAX_USER_INPUT_LENGTH
        ));
    }
    Ok(())
}

fn validate_session_id(session_id: &str) -> Result<(), String> {
    if session_id.is_empty() {
        return Err("session_id cannot be empty".to_string());
    }
    if session_id.len() > MAX_SESSION_ID_LENGTH {
        return Err(format!(
            "session_id exceeds maximum length of {} characters",
            MAX_SESSION_ID_LENGTH
        ));
    }
    // Allow alphanumeric, hyphens, underscores, and UUIDs
    let valid_chars = session_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':');
    if !valid_chars {
        return Err("session_id contains invalid characters (only alphanumeric, hyphens, underscores, and colons allowed)".to_string());
    }
    Ok(())
}

fn validate_timeout_secs(timeout: u64) -> Result<(), String> {
    if timeout < MIN_TIMEOUT_SECS {
        return Err(format!(
            "timeout_secs must be at least {} seconds",
            MIN_TIMEOUT_SECS
        ));
    }
    if timeout > MAX_TIMEOUT_SECS {
        return Err(format!(
            "timeout_secs cannot exceed {} seconds",
            MAX_TIMEOUT_SECS
        ));
    }
    Ok(())
}

fn validate_nats_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("nats_url cannot be empty".to_string());
    }
    // Basic URL validation - must start with nats:// or contain a valid format
    if !url.starts_with("nats://") && !url.contains(':') {
        return Err("nats_url must be a valid NATS URL (e.g., nats://localhost:4222)".to_string());
    }
    Ok(())
}

// ── Terminal helpers ─────────────────────────────────────────────────────────
// Use ANSI when supported (Windows Terminal / modern consoles), fall back to
// plain text on legacy consoles.

fn use_color() -> bool {
    // Windows Terminal, VS Code terminal, and real terminals all support ANSI
    if cfg!(windows) {
        std::env::var("WT_SESSION").is_ok()
            || std::env::var("TERM_PROGRAM").is_ok()
            || std::env::var("ANSICON").is_ok()
    } else {
        true
    }
}

fn style(text: impl Into<String>, code: &str) -> String {
    let s: String = text.into();
    if use_color() {
        format!("\x1b[{}m{}\x1b[0m", code, s)
    } else {
        s
    }
}

mod c {
    pub const BOLD: &str = "1";
    pub const DIM: &str = "2";
    pub const GREEN: &str = "32";
    pub const CYAN: &str = "36";
    pub const YELLOW: &str = "33";
    pub const RED: &str = "31";
    pub const MAGENTA: &str = "35";
}

// ── CLI args ─────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(author, version, about = "Wireframe AI - distributed agent system", long_about = None)]
struct Args {
    /// Direct task string (if not provided, opens interactive input)
    task: Option<String>,

    /// NATS server URL (overrides config)
    #[arg(long)]
    nats_url: Option<String>,

    /// Session ID (auto-generated if omitted)
    #[arg(long)]
    session_id: Option<String>,

    /// Timeout in seconds (overrides config)
    #[arg(long)]
    timeout_secs: Option<u64>,

    /// Suppress the welcome banner
    #[arg(long)]
    quiet: bool,
}

// ── Interactive input ────────────────────────────────────────────────────────

/// Read multiline input from stdin with a visual prompt.
async fn read_input() -> String {
    // Show prompt
    print!(
        "\n  {} {}\n\n",
        style(">>>", c::CYAN),
        style("Enter your task (Ctrl+Z then Enter to submit):", c::BOLD),
    );
    std::io::stdout().flush().ok();

    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();
    let mut input = String::new();

    while let Some(line) = lines.next_line().await.unwrap_or(None) {
        input.push_str(&line);
        input.push('\n');
    }

    input.trim().to_string()
}

// ── Waiting spinner ──────────────────────────────────────────────────────────

struct Spinner {
    chars: &'static [&'static str],
    idx: usize,
    last_len: usize,
}

impl Spinner {
    fn new() -> Self {
        Self {
            chars: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            idx: 0,
            last_len: 0,
        }
    }

    fn tick(&mut self, msg: &str) {
        let clear = "\r".to_string() + &" ".repeat(self.last_len) + "\r";
        print!("{}", clear);
        let prefix = if use_color() {
            format!("\x1b[36m{}\x1b[0m", self.chars[self.idx])
        } else {
            ">".to_string()
        };
        let line = format!("  {} {}", prefix, msg);
        print!("{}", line);
        std::io::stdout().flush().ok();
        self.last_len = line.len();
        self.idx = (self.idx + 1) % self.chars.len();
    }

    fn done(&mut self) {
        let clear = "\r".to_string() + &" ".repeat(self.last_len) + "\r";
        print!("{}", clear);
        std::io::stdout().flush().ok();
    }
}

// ── Result formatting ────────────────────────────────────────────────────────

fn format_result(complete: &TaskComplete, elapsed: f64) {
    // Section: result
    println!();
    println!("  {}", style("═".repeat(58), c::DIM));
    println!("  {} {}", style("✓", c::GREEN), style("Result", c::BOLD));
    println!("  {}", style("═".repeat(58), c::DIM));
    println!();

    // Indent the result text
    for line in complete.result.lines() {
        println!("    {}", line);
    }

    // Side effects
    if !complete.side_effects.is_empty() {
        println!();
        println!("  {}", style("── Side effects ──", c::DIM));
        for side in &complete.side_effects {
            let icon = match side.kind.as_str() {
                "file_written" => "📄",
                "command_run" => "⚡",
                "file_read" => "📖",
                _ => " •",
            };
            let path = side
                .path
                .as_ref()
                .map(|p| format!(" ({})", p.display()))
                .unwrap_or_default();
            println!(
                "    {} {}  {}{}",
                style(icon, c::DIM),
                style(&side.kind, c::CYAN),
                side.description,
                path
            );
        }
    }

    // Warnings
    if !complete.warnings.is_empty() {
        println!();
        println!("  {}", style("── Warnings ──", c::YELLOW));
        for w in &complete.warnings {
            println!("    {} {}", style("!", c::YELLOW), w);
        }
    }

    // Footer
    println!();
    println!(
        "  {} {}  {}",
        style("┈", c::DIM),
        style(format!("completed in {:.1}s", elapsed), c::DIM),
        style("┈", c::DIM),
    );
    println!();
}

// ── Helper functions ─────────────────────────────────────────────────────────

fn setup_logging() {
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_span_events(FmtSpan::CLOSE)
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(tracing::Level::WARN.into())
                    .from_env_lossy(),
            )
            .init();
    }
}

fn load_and_validate_config(
    args: &Args,
) -> Result<(WireframeConfig, String, u64), Box<dyn std::error::Error>> {
    let config = WireframeConfig::from_env()?;
    let nats_url = args
        .nats_url
        .as_ref()
        .unwrap_or(&config.nats_url().to_string())
        .clone();
    let timeout_secs = args
        .timeout_secs
        .unwrap_or(config.interface.default_timeout_secs);

    if let Err(e) = validate_nats_url(&nats_url) {
        eprintln!("{} Error: {}", style("✖", c::RED), e);
        std::process::exit(1);
    }
    if let Err(e) = validate_timeout_secs(timeout_secs) {
        eprintln!("{} Error: {}", style("✖", c::RED), e);
        std::process::exit(1);
    }

    Ok((config, nats_url, timeout_secs))
}

async fn setup_nats_connection(
    nats_url: &str,
    quiet: bool,
) -> Result<async_nats::Client, Box<dyn std::error::Error>> {
    if !quiet {
        eprintln!(
            "  {} Connecting to NATS at {} …",
            style("│", c::DIM),
            &nats_url
        );
    }
    let client = async_nats::connect(nats_url).await.map_err(|e| {
        eprintln!("{} Failed to connect to NATS: {}", style("✖", c::RED), e);
        e
    })?;

    announce_online(
        &client,
        "wireframe-ai-interface",
        "0.1.0",
        &[],
        &["task.submitted"],
    )
    .await
    .map_err(|e| {
        eprintln!("{} Failed to announce online: {}", style("✖", c::RED), e);
        e
    })?;

    if !quiet {
        eprintln!("  {} Connected", style("│", c::DIM));
    }

    Ok(client)
}

fn setup_shutdown_handler(client: async_nats::Client) {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        let _ = agentic_sdk::announce_offline(&client, "wireframe-ai-interface", "0.1.0").await;
        std::process::exit(0);
    });
}

async fn get_user_input(args: &Args) -> String {
    let user_input = match args.task.as_ref() {
        Some(t) => t.clone(),
        None => read_input().await,
    };

    if let Err(e) = validate_user_input(&user_input) {
        eprintln!("{} Error: {}", style("✖", c::RED), e);
        std::process::exit(1);
    }

    user_input
}

async fn publish_task(
    client: &async_nats::Client,
    user_input: String,
    session_id: String,
    quiet: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Err(e) = validate_session_id(&session_id) {
        eprintln!("{} Error: {}", style("✖", c::RED), e);
        std::process::exit(1);
    }

    let submitted = TaskSubmitted {
        session_id: session_id.clone(),
        user_input: user_input.clone(),
        submitted_at: chrono::Utc::now().timestamp(),
    };

    let envelope = Envelope::new("task.submitted", submitted, Some(session_id.clone()));
    let correlation_id = envelope.correlation_id.clone();

    if !quiet {
        let preview = if user_input.len() > PREVIEW_LENGTH {
            format!("{} …", &user_input[..PREVIEW_LENGTH])
        } else {
            user_input.clone()
        };
        println!(
            "  {} {}",
            style("│", c::DIM),
            style(
                format!("Submitted: \"{}\"", preview.replace('\n', " ")),
                c::DIM
            ),
        );
    }

    let payload = serde_json::to_string(&envelope)?;

    #[cfg(feature = "schema-validation")]
    {
        if let Err(e) = agentic_sdk::validate_envelope_payload("task.submitted", &envelope.payload)
        {
            eprintln!(
                "{} Schema validation failed for task.submitted: {}",
                style("✖", c::RED),
                e
            );
            std::process::exit(1);
        }
    }

    client.publish("task.submitted", payload.into()).await?;
    info!(
        correlation_id = %correlation_id,
        session_id = %session_id,
        "Published task to task.submitted"
    );

    Ok(correlation_id)
}

async fn wait_for_result(
    client: &async_nats::Client,
    correlation_id: String,
    timeout_secs: u64,
    quiet: bool,
    start: tokio::time::Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut subscription = client.subscribe("task.complete").await?;
    info!("Subscribed to task.complete, waiting for response...");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);

    let mut spinner = Spinner::new();
    let mut spinner_tick = tokio::time::Instant::now();

    loop {
        if !quiet && spinner_tick.elapsed() >= Duration::from_millis(SPINNER_TICK_MS) {
            spinner.tick("waiting for response …");
            spinner_tick = tokio::time::Instant::now();
        }

        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            spinner.done();
            eprintln!(
                "{} Error: timed out after {}s",
                style("✖", c::RED),
                timeout_secs
            );
            std::process::exit(1);
        }

        let msg = match timeout(remaining, subscription.next()).await {
            Ok(Some(msg)) => msg,
            Ok(None) => {
                spinner.done();
                eprintln!(
                    "{} Error: subscription ended without response",
                    style("✖", c::RED)
                );
                std::process::exit(1);
            }
            Err(_) => {
                spinner.done();
                eprintln!(
                    "{} Error: timed out after {}s",
                    style("✖", c::RED),
                    timeout_secs
                );
                std::process::exit(1);
            }
        };

        let env: Envelope<TaskComplete> = match serde_json::from_slice(&msg.payload) {
            Ok(e) => e,
            Err(e) => {
                error!(error = ?e, "failed to parse task.complete");
                continue;
            }
        };

        debug!(
            received_correlation = %env.correlation_id,
            expected_correlation = %correlation_id,
            "Received message on task.complete"
        );

        if env.correlation_id == correlation_id {
            spinner.done();
            let elapsed = start.elapsed().as_secs_f64();
            info!(correlation = %correlation_id, elapsed_s = %elapsed, "task.complete received");
            format_result(&env.payload, elapsed);
            return Ok(());
        } else {
            debug!("ignoring unrelated message (correlation mismatch)");
        }
    }
}

// ── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();

    let args = Args::parse();

    let (config, nats_url, timeout_secs) = load_and_validate_config(&args)?;

    // ── Banner ───────────────────────────────────────────────────────────
    if !args.quiet && config.interface.show_banner {
        println!();
        println!(
            "  {} {}",
            style("◆", c::MAGENTA),
            style(" Wireframe AI ", c::BOLD),
        );
        println!(
            "  {} {}",
            style("│", c::DIM),
            style("distributed agent system", c::DIM),
        );
        println!();
    }

    // ── Connect ──────────────────────────────────────────────────────────
    let start = tokio::time::Instant::now();
    let client = setup_nats_connection(&nats_url, args.quiet).await?;

    setup_shutdown_handler(client.clone());

    // ── Get user input ──────────────────────────────────────────────────
    let user_input = get_user_input(&args).await;

    // ── Publish ──────────────────────────────────────────────────────────
    let session_id = args
        .session_id
        .unwrap_or_else(|| format!("session_{}", uuid::Uuid::new_v4()));

    let correlation_id = publish_task(&client, user_input, session_id, args.quiet).await?;

    // ── Wait for result ─────────────────────────────────────────────────
    wait_for_result(&client, correlation_id, timeout_secs, args.quiet, start).await?;

    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_user_input_valid() {
        assert!(validate_user_input("Hello world").is_ok());
        assert!(validate_user_input("a".repeat(100).as_str()).is_ok());
    }

    #[test]
    fn test_validate_user_input_empty() {
        assert!(validate_user_input("").is_err());
    }

    #[test]
    fn test_validate_user_input_too_long() {
        let too_long = "a".repeat(MAX_USER_INPUT_LENGTH + 1);
        assert!(validate_user_input(&too_long).is_err());
    }

    #[test]
    fn test_validate_session_id_valid() {
        assert!(validate_session_id("session_123").is_ok());
        assert!(validate_session_id("session-abc").is_ok());
        assert!(validate_session_id("session_xyz").is_ok());
        assert!(validate_session_id("session_123-abc_xyz").is_ok());
        assert!(validate_session_id("session_550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn test_validate_session_id_empty() {
        assert!(validate_session_id("").is_err());
    }

    #[test]
    fn test_validate_session_id_invalid_chars() {
        assert!(validate_session_id("session@123").is_err());
        assert!(validate_session_id("session#abc").is_err());
        assert!(validate_session_id("session space").is_err());
    }

    #[test]
    fn test_validate_session_id_too_long() {
        let too_long = "a".repeat(MAX_SESSION_ID_LENGTH + 1);
        assert!(validate_session_id(&too_long).is_err());
    }

    #[test]
    fn test_validate_timeout_secs_valid() {
        assert!(validate_timeout_secs(1).is_ok());
        assert!(validate_timeout_secs(60).is_ok());
        assert!(validate_timeout_secs(3600).is_ok());
    }

    #[test]
    fn test_validate_timeout_secs_too_low() {
        assert!(validate_timeout_secs(0).is_err());
    }

    #[test]
    fn test_validate_timeout_secs_too_high() {
        assert!(validate_timeout_secs(3601).is_err());
    }

    #[test]
    fn test_validate_nats_url_valid() {
        assert!(validate_nats_url("nats://localhost:4222").is_ok());
        assert!(validate_nats_url("nats://127.0.0.1:4222").is_ok());
        assert!(validate_nats_url("localhost:4222").is_ok());
    }

    #[test]
    fn test_validate_nats_url_empty() {
        assert!(validate_nats_url("").is_err());
    }

    #[test]
    fn test_validate_nats_url_invalid() {
        assert!(validate_nats_url("not-a-url").is_err());
    }
}
