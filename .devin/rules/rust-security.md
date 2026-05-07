---
paths:
  - "**/*.rs"
---
# Rust Security for Wireframe-AI

Wireframe-AI specific security guidelines for Rust development.

## Secrets Management

- Never hardcode API keys, tokens, or credentials in source code
- Use environment variables: `std::env::var("API_KEY")`
- Fail fast if required secrets are missing at startup
- Keep `.env` files in `.gitignore`
- For provider credentials, use the vault system in `vault/` directory

```rust
// BAD
const API_KEY: &str = "sk-abc123...";

// GOOD — environment variable with early validation
fn load_api_key() -> anyhow::Result<String> {
    std::env::var("PAYMENT_API_KEY")
        .context("PAYMENT_API_KEY must be set")
}

// GOOD — use vault for provider credentials
fn load_provider_credentials(provider_name: &str) -> anyhow::Result<ProviderCredentials> {
    vault::get_credentials(provider_name)
        .context(format!("Failed to load credentials for provider: {}", provider_name))
}
```

## SQL Injection Prevention

- Always use parameterized queries — never format user input into SQL strings
- Use query builder or ORM (sqlx, diesel, sea-orm) with bind parameters

```rust
// BAD — SQL injection via format string
let query = format!("SELECT * FROM users WHERE name = '{name}'");
sqlx::query(&query).fetch_one(&pool).await?;

// GOOD — parameterized query with sqlx
// Placeholder syntax varies by backend: Postgres: $1  |  MySQL: ?  |  SQLite: $1
sqlx::query("SELECT * FROM users WHERE name = $1")
    .bind(&name)
    .fetch_one(&pool)
    .await?;
```

## Input Validation

- Validate all user input at system boundaries before processing
- Use the type system to enforce invariants (newtype pattern)
- Parse, don't validate — convert unstructured data to typed structs at the boundary
- Reject invalid input with clear error messages

```rust
// Parse, don't validate — invalid states are unrepresentable
pub struct Email(String);

impl Email {
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let trimmed = input.trim();
        let at_pos = trimmed.find('@')
            .filter(|&p| p > 0 && p < trimmed.len() - 1)
            .ok_or_else(|| ValidationError::InvalidEmail(input.to_string()))?;
        let domain = &trimmed[at_pos + 1..];
        if trimmed.len() > 254 || !domain.contains('.') {
            return Err(ValidationError::InvalidEmail(input.to_string()));
        }
        // For production use, prefer a validated email crate (e.g., `email_address`)
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

## Provider System Security

### Credential Management

- Provider credentials must be stored in the vault, not in source code
- Never log or expose provider API keys in error messages
- Rotate provider credentials regularly
- Use least-privilege access for provider API keys

```rust
// GOOD — load credentials from vault
pub async fn initialize_provider(
    provider_type: ProviderType,
    vault: &Vault,
) -> anyhow::Result<Box<dyn Provider>> {
    let credentials = vault.get_provider_credentials(&provider_type)
        .context("Failed to load provider credentials")?;

    match provider_type {
        ProviderType::OpenAI => {
            let provider = OpenAIProvider::new(credentials.api_key)?;
            Ok(Box::new(provider))
        }
        ProviderType::Anthropic => {
            let provider = AnthropicProvider::new(credentials.api_key)?;
            Ok(Box::new(provider))
        }
    }
}

// BAD — hardcode credentials
pub fn initialize_provider_bad() -> anyhow::Result<Box<dyn Provider>> {
    let provider = OpenAIProvider::new("sk-abc123...")?;
    Ok(Box::new(provider))
}
```

### Capability Negotiation

- Always negotiate provider capabilities before use
- Validate that requested features are supported
- Fail gracefully when capabilities are unavailable

```rust
pub async fn execute_with_capabilities(
    provider: &dyn Provider,
    request: ChatRequest,
) -> anyhow::Result<ChatResponse> {
    let capabilities = provider.capabilities();

    if request.requires_streaming && !capabilities.supports_streaming {
        return Err(anyhow::anyhow!("Provider does not support streaming"));
    }

    if request.requires_function_calling && !capabilities.supports_function_calling {
        return Err(anyhow::anyhow!("Provider does not support function calling"));
    }

    provider.chat_completion(request).await
}
```

## NATS Security

- Use NATS authentication in production
- Enable TLS for NATS connections
- Use subject-based access control (JetStream) when available
- Never publish sensitive data in clear text over NATS

```rust
// GOOD — secure NATS connection
pub async fn connect_nats_secure(
    url: &str,
    credentials: &NatsCredentials,
) -> anyhow::Result<async_nats::Client> {
    let options = async_nats::ConnectOptions::new()
        .user_and_password(credentials.user.clone(), credentials.password.clone())
        .require_tls(true);

    async_nats::connect_with_options(url, options).await
}

// BAD — insecure connection
pub async fn connect_nats_insecure(url: &str) -> anyhow::Result<async_nats::Client> {
    async_nats::connect(url).await
}
```

## Unsafe Code

- Minimize `unsafe` blocks — prefer safe abstractions
- Every `unsafe` block must have a `// SAFETY:` comment explaining the invariant
- Never use `unsafe` to bypass the borrow checker for convenience
- Audit all `unsafe` code during review — it is a red flag without justification
- Prefer `safe` FFI wrappers around C libraries

```rust
// GOOD — safety comment documents ALL required invariants
let widget: &Widget = {
    // SAFETY: `ptr` is non-null, aligned, points to an initialized Widget,
    // and no mutable references or mutations exist for its lifetime.
    unsafe { &*ptr }
};

// BAD — no safety justification
unsafe { &*ptr }
```

## Dependency Security

- Run `cargo audit` to scan for known CVEs in dependencies
- Run `cargo deny check` for license and advisory compliance
- Use `cargo tree` to audit transitive dependencies
- Keep dependencies updated — set up Dependabot or Renovate
- Minimize dependency count — evaluate before adding new crates

```bash
# Security audit
cargo audit

# Deny advisories, duplicate versions, and restricted licenses
cargo deny check

# Inspect dependency tree
cargo tree
cargo tree -d  # Show duplicates only
```

## Error Messages

- Never expose internal paths, stack traces, or database errors in API responses
- Log detailed errors server-side; return generic messages to clients
- Use `tracing` or `log` for structured server-side logging
- Never expose provider API keys or credentials in error messages

```rust
// Map errors to appropriate status codes and generic messages
match order_service.find_by_id(id) {
    Ok(order) => Ok((StatusCode::OK, Json(order))),
    Err(ServiceError::NotFound(_)) => {
        tracing::info!(order_id = id, "order not found");
        Err((StatusCode::NOT_FOUND, "Resource not found"))
    }
    Err(e) => {
        tracing::error!(order_id = id, error = %e, "unexpected error");
        Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"))
    }
}

// BAD — exposing internal details
Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))
```

## Sandbox Security

When executing code in the sandbox module:
- Validate all inputs before execution
- Use resource limits (CPU, memory, time)
- Never allow access to filesystem or network from sandbox
- Implement proper error boundaries

```rust
pub struct SandboxConfig {
    pub max_cpu_time: Duration,
    pub max_memory: usize,
    pub allow_network: bool,
    pub allow_filesystem: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_cpu_time: Duration::from_secs(30),
            max_memory: 128 * 1024 * 1024, // 128MB
            allow_network: false,
            allow_filesystem: false,
        }
    }
}
```

## References

See skill: `rust-pro` for unsafe code guidelines and ownership patterns.
See skill: `security-auditor` for comprehensive security review patterns.