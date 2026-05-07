---
name: project-routing
description: Project type routing for Wireframe-AI - agent and skill selection based on task type
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# Project Type Routing

## Purpose

Routing framework for Wireframe-AI: select appropriate agent and skills based on task type. Ensures tasks are handled by specialists with the right expertise and tools, improving efficiency and quality.

## When to Use

**Use this skill when:**
- Starting a new task and unsure which agent to use
- Determining the right skill combination for a task
- Routing complex tasks to appropriate specialists
- Ensuring task type matches Wireframe-AI architecture

**Do NOT use when:**
- Task is clearly defined with known agent/skill
- Running simple mechanical operations (edit, format)
- The task is already in progress with an agent

## Protocol

### Phase 1: Task Classification

1. **Analyze task requirements**
   - Read the task description carefully
   - Identify the primary domain (architecture, database, security, etc.)
   - Note any cross-domain dependencies

2. **Match to task type**
   - Compare task requirements to routing table
   - Identify closest matching task type
   - Note any secondary skill requirements

3. **Verify architecture fit**
   - Ensure task aligns with Wireframe-AI distributed system
   - Check if web framework patterns are incorrectly requested
   - Validate async/event-driven approach is appropriate

### Phase 2: Agent Selection

1. **Select primary agent**
   - Use routing table to identify agent for task type
   - Consider agent expertise and capabilities
   - Check agent availability and permissions

2. **Select supporting skills**
   - Identify skills from routing table for task type
   - Add any additional skills for cross-domain tasks
   - Ensure skills are compatible with agent profile

### Phase 3: Task Execution

1. **Invoke agent with context**
   - Provide task description and selected skills
   - Include relevant file paths and context
   - Set appropriate permissions for tools

2. **Monitor progress**
   - Track agent progress through checkpoints
   - Verify agent is using appropriate skills
   - Adjust routing if task scope changes

## Project Type Routing

Use this routing table to select the appropriate agent and skills for Wireframe-AI tasks.

| Type | Agent | Skills |
| --- | --- | --- |
| ARCHITECTURAL DECISION | architect | architecture, orchestration-patterns |
| KERNEL/RUST MODULE | backend-specialist | rust-pro, async-tokio-patterns |
| PYTHON ADAPTER | backend-specialist | python-patterns |
| SDK DEVELOPMENT | backend-specialist | rust-pro, database-design |
| SCHEMA DEFINITION | schema-validator | wireframe-workflow, architecture-decision-records |
| DATABASE DESIGN | database-architect | database-design, database-migrations |
| PERFORMANCE OPTIMIZATION | performance-optimizer | performance-profiling, rust-pro |
| SECURITY REVIEW | security-reviewer | rust-security, code-review-checklist |
| RUST CODE REVIEW | rust-reviewer | rust-pro, code-review-checklist |
| INTEGRATION TESTING | test-runner | rust-testing-wireframe, systematic-debugging |
| CODEBASE RESEARCH | rust-researcher | parallel-search, research-architecture |
| FAST CODEBASE MAPPING | fast-researcher | parallel-search |

### Important Note

This is a distributed system, not a web app. Use system architecture and async patterns, not web frameworks.

### Task Type Examples

**ARCHITECTURAL DECISION:**
- Designing new module architecture
- Evaluating NATS messaging patterns
- Planning system scalability
- Making technical trade-off decisions

**KERNEL/RUST MODULE:**
- Implementing new kernel functionality
- Modifying NATS communication layer
- Adding new module interfaces

**PYTHON ADAPTER:**
- Adding AI/ML adapters
- Implementing Python tool interfaces
- MCP protocol integration

**SDK DEVELOPMENT:**
- Creating new SDK features
- Updating agentic-sdk-macros
- Extending agentic-sdk-py

**SCHEMA DEFINITION:**
- Modifying envelope structure
- Adding new message contracts
- Schema validation changes

**DATABASE DESIGN:**
- Designing new database schemas
- Planning data migrations
- Optimizing database queries
- Designing indexing strategies

**PERFORMANCE OPTIMIZATION:**
- Optimizing message throughput
- Reducing latency in critical paths
- Memory optimization
- Benchmarking and profiling

**SECURITY REVIEW:**
- Reviewing code for security vulnerabilities
- Checking provider credential handling
- Validating input sanitization
- Reviewing NATS authentication

**RUST CODE REVIEW:**
- Reviewing Rust code for best practices
- Checking ownership and lifetime usage
- Validating error handling patterns
- Ensuring Wireframe-AI pattern compliance

**INTEGRATION TESTING:**
- Writing module-to-module tests
- Benchmark tests
- End-to-end system tests

**CODEBASE RESEARCH:**
- Understanding module dependencies
- Tracing message flows
- Finding implementation patterns
- Researching specific features

**FAST CODEBASE MAPPING:**
- Quick file location searches
- Finding function definitions
- Identifying code structure
- Rapid pattern detection

## Common Pitfalls

| Pitfall | Why Bad | Correct Approach |
|---------|---------|------------------|
| Using web framework patterns for distributed system | Wrong architecture for Wireframe-AI | Use async/event-driven patterns with NATS |
| Routing to wrong agent type | Agent lacks expertise for domain | Use routing table to match task type to agent |
| Not verifying architecture fit | May introduce incompatible patterns | Ensure task aligns with distributed system architecture |
| Skipping skill selection | Agent may lack necessary guidance | Select appropriate skills from routing table |
| Ignoring cross-domain dependencies | Incomplete solution for complex tasks | Identify and address cross-domain requirements |
| Using general-purpose agent for specialist task | Suboptimal quality and efficiency | Route to specialist agent (e.g., database-architect for schema work) |

## Code Examples

**Example: Routing a database schema task**

```python
# Task: "Design a new schema for user preferences"

# Phase 1: Task Classification
# Primary domain: Database design
# Task type: DATABASE DESIGN

# Phase 2: Agent Selection
# Agent: database-architect
# Skills: database-design, database-migrations

# Phase 3: Task Execution
run_subagent(
    title="Database schema design",
    task="Design schema for user preferences in Wireframe-AI Context module. Use SQLite with proper indexing. Consider migration path.",
    profile="database-architect",
    skills=["database-design", "database-migrations"]
)
```

**Example: Routing a security review**

```python
# Task: "Review provider credential handling for security vulnerabilities"

# Phase 1: Task Classification
# Primary domain: Security
# Task type: SECURITY REVIEW

# Phase 2: Agent Selection
# Agent: security-reviewer
# Skills: rust-security, code-review-checklist

# Phase 3: Task Execution
run_subagent(
    title="Security review of provider credentials",
    task="Review provider credential handling in provider-core/ and providers/ for security vulnerabilities. Check for hardcoded secrets, insecure storage, and proper vault integration.",
    profile="security-reviewer",
    skills=["rust-security", "code-review-checklist"]
)
```

**Example: Verifying architecture fit**

```python
# Task: "Add a REST API endpoint for user management"

# Phase 1: Task Classification
# Primary domain: API development
# Task type: ???

# Phase 3: Verify architecture fit
# This is a distributed system, not a web app
# REST API endpoints should use NATS messaging instead
# Redirect to: KERNEL/RUST MODULE with NATS patterns

# Correct routing:
# Agent: backend-specialist
# Skills: rust-pro, async-tokio-patterns
# Task: "Add NATS subscription for user management requests"
```

## Integration

**Related skills:**
- **superpowers:intelligent-routing** - Automatic agent selection and intelligent task routing
- **superpowers:karpathy-guidelines** - Behavioral standards for task execution
- **superpowers:wireframe-workflow** - Wireframe-AI specific development workflow

**Workflow context:**
- Use at the start of complex tasks to ensure correct routing
- Use when task type is ambiguous or unclear
- Use with `/intelligent-routing` for automatic agent selection
- Use with `/karpathy-guidelines` for behavioral standards after routing
