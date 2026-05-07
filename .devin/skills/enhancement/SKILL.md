---
name: enhancement
description: Add or update features in existing applications with systematic planning, user approval, incremental implementation, and thorough testing. Use for iterative development with rollback safety
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
  - exec
triggers:
  - model
---

# Feature Enhancement

"Add or update features in existing applications with systematic planning, user approval, and thorough testing."

## When to Use
- Adding new features to existing applications
- Updating or modifying existing functionality
- Iterative development and improvements
- Refactoring existing code with architectural considerations
- Integrating new libraries or dependencies
- Performance optimization of existing features
- Security enhancements to current codebase

## When NOT to Use
- Creating new applications from scratch (use app-creation skill instead)
- Simple bug fixes (use systematic-debugging skill instead)
- Non-application tasks
- Breaking changes without user approval
- Conflicting with existing architecture without warning
- Emergency hotfixes (use direct editing instead)

## Subagent Usage

**CRITICAL:** Use subagents for parallel investigation and implementation to maximize efficiency.

### When to Use Subagents

- **Codebase investigation:** When you need to understand multiple parts of the codebase simultaneously
- **Dependency analysis:** When changes affect multiple modules or services
- **Parallel implementation:** When different aspects of a feature can be implemented independently
- **Cross-module changes:** When a feature requires coordinated changes across multiple files
- **Testing coordination:** When you need to run tests in parallel or investigate test failures

### Subagent Strategy

**Smart Lead, Fast Workers Pattern:**

```powershell
# Lead agent orchestrates, fast subagents do the work

# Phase 1: Parallel Investigation
$investigationTasks = @(
    "Analyze current authentication implementation",
    "Review database schema for user tables",
    "Check existing API endpoints",
    "Examine error handling patterns"
)

foreach ($task in $investigationTasks) {
    # Launch read-only subagent for investigation
    # Each subagent returns: findings, file paths, recommendations
}

# Phase 2: Synthesis and Planning
# Lead agent synthesizes findings, creates implementation plan

# Phase 3: Parallel Implementation (if applicable)
# Launch implementation subagents for independent components
```

### Subagent Profiles

Use appropriate subagent profiles based on task needs:

- **subagent_explore:** Read-only investigation of codebase structure and patterns
- **rust-researcher:** Read-only research for Wireframe-AI Rust codebase architecture
- **subagent_general:** General-purpose subagent with full tool access for implementation
- **test-runner:** Execute tests and report results

### Subagent Coordination

1. **Investigation Phase:**
   - Spawn read-only subagents in parallel for codebase investigation
   - Each subagent focuses on a specific aspect (modules, patterns, dependencies)
   - Collect and synthesize findings

2. **Planning Phase:**
   - Lead agent creates implementation plan based on subagent findings
   - Identify tasks that can be parallelized

3. **Implementation Phase:**
   - For independent components, spawn implementation subagents
   - Use `is_background: true` for parallel execution
   - Monitor progress with `read_subagent`

4. **Validation Phase:**
   - Use test-runner subagent for test execution
   - Run tests in parallel where possible

### Subagent Guardrails

- **Investigation vs. Implementation:** Use read-only subagents for investigation, implementation subagents only for isolated changes
- **Clear scope:** Each subagent gets a specific, well-defined task
- **No conflicts:** Ensure subagents don't modify the same files simultaneously
- **Coordination:** Lead agent must synthesize results and make final decisions
- **Testing:** Always run tests after subagent implementation changes

## Pre-flight Checks

```powershell
# Verify project state - adapt to your project's configuration file
# Examples: Cargo.toml, package.json, pom.xml, build.gradle, requirements.txt
if (-not (Test-Path "<your-config-file>")) {
    Write-Error "No recognized project configuration found"
    exit 1
}

# Check for uncommitted changes
$gitStatus = git status --porcelain
if ($gitStatus) {
    Write-Warning "Uncommitted changes detected. Consider committing before enhancement."
}

# Verify build passes - adapt to your project's build command
# Examples: cargo check, npm run build, python -m py_compile, mvn compile
<build-check-command> 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Error "Project does not build. Fix errors before enhancement."
    exit 1
}

# List existing tests - adapt to your project's test file pattern
# Examples: *test*.rs, *.test.ts, *_test.py, *Test.java
$testFiles = Get-ChildItem -Recurse -Filter "<your-test-pattern>" -ErrorAction SilentlyContinue
Write-Host "Found $($testFiles.Count) test files"
```

## The Enhancement Process

### Phase 1: Understand Current State

**Actions:**
- Read main configuration files (adapt to your project: Cargo.toml, package.json, pom.xml, etc.)
- Understand existing features and tech stack
- Review current architecture (ARCHITECTURE.md if exists)
- Identify entry points and core modules
- Check existing test coverage

**Deliverable:** Project context summary with tech stack, key files, and architectural patterns.

### Phase 2: Plan Changes

**Actions:**
- Use plan-writing skill for systematic planning with files
- Register team per global rules Rule 2
- Determine what will be added/changed
- Detect affected files using dependency analysis
- Check compatibility with existing dependencies
- Identify potential breaking changes
- Estimate implementation effort

**Questions to Answer:**
- Which files need modification?
- New files required?
- Dependencies to add?
- Migration path needed?
- Testing strategy?

**Deliverable:** Change plan in `docs/PLAN-{slug}.md` with file list, dependency changes, and risk assessment.

### Phase 3: Present Plan to User

**For major changes (threshold: >5 files or breaking changes):**

```
"To add [feature]:
- I'll create X new files: [list]
- Update Y existing files: [list]
- Add Z dependencies: [list]
- Estimated effort: ~[time]
- Risk level: [low/medium/high]
- Breaking changes: [yes/no]

Should I proceed?"
```

**For minor changes:**
- Proceed directly but log changes
- Still check for breaking changes

### Phase 4: Apply Changes

**Actions:**
- Call relevant agents based on domain (frontend, backend, database)
- Make changes incrementally
- Test each change before proceeding
- Commit after each logical unit

**Order of Operations:**
1. Update dependencies (if any)
2. Create new files
3. Modify existing files
4. Update configuration
5. Add tests
6. Run test suite

**Deliverable:** Applied changes with passing tests.

### Phase 5: Update and Verify

**Actions:**
- Hot reload if supported (dev server)
- Restart application if needed
- Manual verification of new functionality
- Regression testing of affected areas
- Performance baseline comparison (if applicable)

**Deliverable:** Verified working enhancement with test results.

## Additional Resources

For edge case handling, failure modes, performance considerations, security notes, and guardrails, see `@[skills/enhancement/SKILL-advanced.md]`.

For success criteria, rollback procedures, and related skills, see `@[skills/enhancement/SKILL-examples.md]`.