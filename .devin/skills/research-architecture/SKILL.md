---
name: research-architecture
description: Understand codebase structure before making changes
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# Research Architecture

## Purpose

Understand the codebase structure, dependencies, and architectural patterns before making changes. This skill prevents breaking changes by identifying risks, dependencies, and safe modification patterns.

## When to Use

Use this skill when:
- Starting work on a new feature or bug fix in an unfamiliar codebase
- Needing to understand the impact of proposed changes
- Planning refactoring or architectural modifications
- Onboarding to a new project or module
- Debugging complex issues that require architectural context

## Protocol

### Phase 1: Discovery

1. **Identify Top-Level Structure**
   - List all top-level directories/packages/modules
   - Identify entry points (main functions, CLI commands, API endpoints)
   - Note configuration files and documentation

2. **Spawn Read-Only Subagents**
   - Create one subagent per top-level package/module
   - Use `subagent_explore` profile for read-only access
   - Provide each subagent with specific scope and questions

3. **Subagent Investigation Tasks**
   Each subagent should identify:
   - Purpose and responsibility of the package/module
   - Key files and their roles
   - Public APIs and interfaces
   - Dependencies (internal and external)
   - Test coverage and strategy
   - Likely risks or edge cases

### Phase 2: Synthesis

1. **Compile Findings**
   - Gather all subagent reports
   - Identify cross-module dependencies
   - Note shared patterns and conventions

2. **Create Dependency Graph**
   - Map relationships between modules
   - Identify circular dependencies
   - Highlight critical paths

3. **Risk Assessment**
   - Rate each module by risk level (LOW/MEDIUM/HIGH)
   - Identify files that require special care
   - Note areas with poor test coverage

4. **Implementation Guidance**
   - Recommend safe implementation order
   - Identify integration points to test
   - List files to avoid touching

## Output Format

Present findings in this structure:

### Dependency Graph
```
Module A → Module B → Module C
         ↘ Module D ↗
```

### Risk Assessment
| Module | Risk Level | Reason |
|--------|-----------|---------|
| core | HIGH | No tests, used everywhere |
| utils | LOW | Well-tested, stable API |
| api | MEDIUM | External dependencies |

### Implementation Order
1. Start with low-risk, independent modules
2. Progress to higher-risk modules
3. Test integration at each step

### Risky Files
- `core/internal.rs` - No tests, critical path
- `api/handler.rs` - Complex state, poor error handling

## Integration

This skill integrates with:
- `/parallel-search` - For fast initial codebase mapping
- `/karpathy-guidelines` - For Think Before Coding principle
- `/architecture` - For deeper architectural analysis
- `/project-routing` - For identifying the right approach based on architecture

## Common Pitfalls

- **Skipping subagent synthesis**: Don't just collect reports - synthesize them into actionable guidance
- **Ignoring dependencies**: Missing dependencies leads to breaking changes
- **Overlooking test coverage**: Untested areas are higher risk
- **Rushing to implementation**: Complete architecture research first
