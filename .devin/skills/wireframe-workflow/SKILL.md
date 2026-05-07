---
name: wireframe-workflow
description: Wireframe-AI development workflow - file dependency awareness, schema validation, and coordinated updates
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# Wireframe-AI Development Workflow

## Purpose

Ensure coordinated development across Wireframe-AI modules by understanding dependencies, validating schema contracts, and making coordinated updates. Prevent breaking changes by requiring awareness of file relationships and system architecture before making changes.

## When to Use

Use this skill when:
- Making changes to Wireframe-AI modules
- Modifying schema definitions
- Working on kernel or interface changes
- Updating SDK or adapter code
- Making changes that affect multiple modules
- Planning refactoring or architectural changes

## Protocol

### Step 1: Documentation Review

1. **Read Project Documentation**
   ```bash
   # Read core project documentation
   cat docs/Project-Core.md
   cat docs/Project-Architecture.md
   ```
   Understand system overview and architecture

2. **Understand System Principles**
   - Event-driven architecture with NATS messaging
   - State ownership via Context module
   - Provider system for LLM integrations
   - Schema contracts for message envelopes

### Step 2: Schema Contract Validation

1. **Review Schema Definitions**
   ```bash
   # Check schema directory
   ls schemas/v1/
   ```
   Review: envelope, agent_job, agent_result, and other schemas

2. **Understand Contract Requirements**
   - Never change root fields in message envelopes
   - Provide migration paths for breaking changes
   - Validate schemas before deployment
   - Maintain backward compatibility for consumers

### Step 3: Dependency Identification

1. **Identify Module Dependencies**
   - `kernel/` - Interface and NATS communication
   - `modules/context/` - State ownership module
   - `modules/orchestrator/` - Task orchestration module
   - `modules/sandbox/` - Code execution module
   - `sdk/` - Rust SDKs (agentic-sdk, agentic-sdk-macros)
   - `adapter/python/` - Python adapter for AI/ML

2. **Map File Relationships**
   - Which modules depend on this file?
   - Which files will break if this changes?
   - What contracts will be affected?

### Step 4: Coordinated Updates

1. **Plan Coordinated Changes**
   - Schema changes require coordinated updates across modules
   - Never update schema in isolation
   - Identify all files that need updates together
   - Plan the order of changes

2. **Update All Affected Files**
   - Make changes to all affected files in one commit
   - Ensure schema and code changes are synchronized
   - Verify all modules use updated contracts

### Step 5: Apply Principles

1. **Read → Understand → Apply**
   - Don't just read and start coding
   - Understand WHY the change is needed
   - Apply Wireframe-AI principles
   - Consider how this differs from generic approaches

2. **Before Coding Checklist**
   - What is the GOAL?
   - What PRINCIPLES apply?
   - How does this DIFFER from generic?

### Step 6: Verification

1. **Run Integration Tests**
   ```bash
   cargo test --test integration_test
   ```
   Verify integration across modules

2. **Verify with Release Build**
   ```bash
   cargo build --release
   ```
   Ensure all modules compile successfully

3. **Code Review**
   - Use review skill before opening PR
   - Verify coordinated updates work correctly
   - Check for unintended side effects

## Dependency Mapping

| Module | Depends On | Affected By |
|--------|-----------|-------------|
| kernel | schemas, NATS | All modules |
| context | schemas, NATS | orchestrator, sandbox |
| orchestrator | context, schemas, NATS | SDK |
| sandbox | schemas, NATS | orchestrator |
| SDK (Rust) | schemas, context | adapter/python |
| SDK (Python) | SDK (Rust) | External applications |
| adapter/python | SDK (Rust), schemas | External applications |

## Schema Change Protocol

| Change Type | Required Actions |
|-------------|------------------|
| Add field to schema | Add optional field, maintain backward compatibility |
| Modify field type | Create new version, provide migration |
| Remove field | Deprecate in previous version, remove in next |
| Breaking change | Increment version, update all consumers |

## Common Pitfalls

| Pitfall | Consequence | Prevention |
|---------|------------|------------|
| Updating schema in isolation | Breaking changes in dependent modules | Always update all affected files together |
| Ignoring dependencies | Breaking changes cascade | Map dependencies before changing |
| Skipping documentation | Team doesn't understand changes | Always document architectural decisions |
| Not testing integration | Issues discovered in production | Run integration tests before committing |

## Integration

This skill integrates with:
- `/karpathy-guidelines` - For Think Before Coding principle
- `/architecture` - For architectural decision-making
- `/database-migrations` - For schema migration planning
- `/final-checks` - For comprehensive verification
