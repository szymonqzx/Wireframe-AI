---
name: orchestration-patterns-examples
description: Practical examples of swarm orchestration for Wireframe-AI
---

## Practical Swarm Examples for Wireframe-AI

### Example 1: Adding a New Module to Wireframe-AI

**Lead Agent:** Opus (smartest model)

**Batch 1 (3 workers, swe model):**
1. **Kernel integration researcher** - Analyze `kernel/` for module registration patterns
2. **Schema researcher** - Review `schemas/v1/` for envelope contracts
3. **NATS topic researcher** - Examine existing topic naming conventions

**Batch 2 (3 workers, if needed):**
4. **Context module researcher** - Study `modules/context/` for state patterns
5. **SDK researcher** - Review `sdk/agentic-sdk/` for module macro usage
6. **Test researcher** - Analyze `tests/` for integration test patterns

**Each worker reports:**
- File paths and line numbers for relevant patterns
- Required interfaces and contracts
- NATS topic suggestions
- Test requirements
- Dependencies on other modules

**Lead agent synthesizes after each batch:**
- Module implementation plan
- NATS topic name following conventions
- Schema changes required
- Integration test strategy
- Step-by-step implementation order

### Example 2: Debugging NATS Communication Issue

**Lead Agent:** Sonnet (smart model)

**Batch 1 (3 workers, swe model):**
1. **Kernel NATS researcher** - Check `kernel/interface/src/main.rs` NATS setup
2. **Publisher researcher** - Find which module publishes the problematic message
3. **Subscriber researcher** - Find which module should receive the message

**Batch 2 (1 worker, if needed):**
4. **Envelope researcher** - Verify message structure matches schema

**Each worker reports:**
- NATS connection configuration
- Topic subscription patterns
- Message envelope structure
- Queue group configuration
- Error handling in message flow

**Lead agent synthesizes:**
- Root cause identification
- Specific file paths and line numbers for the issue
- Fix recommendations
- Test strategy to verify fix

### Example 3: Performance Optimization of Message Flow

**Lead Agent:** Sonnet (smart model)

**Batch 1 (3 workers, swe model):**
1. **Serialization researcher** - Analyze serde usage in message paths
2. **Async pattern researcher** - Review tokio async/await usage
3. **NATS configuration researcher** - Check connection pooling and batching

**Batch 2 (2 workers, if needed):**
4. **Database researcher** - Analyze SQLite query patterns in context module
5. **Benchmark researcher** - Review existing benchmark tests

**Each worker reports:**
- Performance bottlenecks with metrics
- Specific file paths and line numbers
- Optimization opportunities
- Before/after comparison suggestions

**Lead agent synthesizes:**
- Prioritized optimization plan
- Expected performance improvements
- Risk assessment for each change
- Benchmark strategy to validate improvements

### Example 4: Schema Migration Planning

**Lead Agent:** Sonnet (smart model)

**Batch 1 (3 workers, swe model):**
1. **Current schema researcher** - Analyze `schemas/v1/` current state
2. **Usage researcher** - Find all modules using the schema
3. **Migration researcher** - Review existing migration patterns

**Batch 2 (1 worker, if needed):**
4. **Test researcher** - Identify tests affected by schema change

**Each worker reports:**
- Current schema structure
- All dependent modules and files
- Backward compatibility requirements
- Test coverage for schema changes

**Lead agent synthesizes:**
- Migration plan with backward compatibility
- Order of module updates
- Test strategy for validation
- Rollback procedure if needed

### Example 5: Security Audit of Message Flow

**Lead Agent:** Opus (smartest model)

**Batch 1 (3 workers, swe model):**
1. **Input validation researcher** - Check message validation in kernel
2. **Envelope researcher** - Verify envelope field access controls
3. **NATS security researcher** - Review NATS authentication and authorization

**Batch 2 (2 workers, if needed):**
4. **Module boundary researcher** - Check sandbox isolation
5. **Secret researcher** - Scan for hardcoded secrets or credentials

**Each worker reports:**
- Security vulnerabilities with severity
- File paths and line numbers
- Risk assessment
- Fix recommendations

**Lead agent synthesizes:**
- Prioritized security issues
- Fix implementation plan
- Security testing strategy
- Ongoing security monitoring recommendations