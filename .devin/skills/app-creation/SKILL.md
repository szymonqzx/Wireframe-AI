---
name: app-creation
description: Create new applications from scratch with systematic planning, team registration, and coordinated expert agents. Use for MVPs, prototypes, and new project setup.
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

# Application Creation

"Create new applications with systematic planning, team registration, and coordinated expert agents."

## When to Use
- Creating a new application from scratch
- Starting a new project with no existing codebase
- Building MVP or prototype applications
- Setting up initial project structure and tech stack
- Implementing features for greenfield projects

## When NOT to Use
- Modifying existing applications (use enhancement skill instead)
- Simple bug fixes or small changes
- Adding features to existing codebase
- Non-application tasks (scripts, utilities)

## Core Process

### Step 1: Request Analysis

**Understand what the user wants:**
- What type of application?
- What are the basic features?
- Who will use it?

**If information is missing, use brainstorming skill to ask clarifying questions.**

### Step 2: Project Planning

**Use plan-writing skill for systematic planning with files:**
- Register team per global rules Rule 2
- Determine tech stack
- Plan file structure
- Create plan file in `docs/PLAN-{slug}.md` and proceed to building

### Step 3: Application Building (After Approval)

**Orchestrate with expert agents:**
- `database-architect` → Schema
- `backend-specialist` → API
- `frontend-specialist` → UI

### Step 4: Preview

**Start with auto_preview.py when complete and present URL to user.**

## Subagent Usage

**CRITICAL:** Use subagents for parallel implementation to maximize development velocity.

### When to Use Subagents

- **Independent components:** When the application has multiple components that can be implemented separately
- **Parallel development:** When different parts of the application don't depend on each other
- **Testing coordination:** When you need to run multiple test suites in parallel
- **Code generation:** When generating boilerplate or repetitive code across multiple files
- **Documentation:** When writing documentation alongside implementation

### Subagent Strategy

**Parallel Implementation Pattern:**

```powershell
# Lead agent orchestrates, subagents implement in parallel

# Phase 1: Planning (lead agent)
# Create detailed implementation plan with task breakdown

# Phase 2: Parallel Implementation
$implementationTasks = @(
    @{Task = "Implement user model"; Files = @("src/models/user.rs")},
    @{Task = "Implement auth service"; Files = @("src/services/auth.rs")},
    @{Task = "Create API endpoints"; Files = @("src/api/auth.rs")},
    @{Task = "Write tests"; Files = @("tests/auth_test.rs")}
)

foreach ($task in $implementationTasks) {
    # Launch implementation subagent
    # Each subagent works on isolated file set
}

# Phase 3: Integration and Testing
# Lead agent integrates components, runs full test suite
```

### Subagent Profiles

Use appropriate subagent profiles based on task needs:

- **subagent_general:** General-purpose subagent with full tool access for implementation
- **database-architect:** Database architect for schema design and data modeling
- **backend-specialist:** Backend architect for API development and server logic
- **test-runner:** Execute tests and report results
- **subagent_explore:** Read-only investigation of codebase structure and patterns

### Subagent Coordination

1. **Planning Phase:**
   - Lead agent creates detailed implementation plan
   - Identify tasks that can be parallelized
   - Define clear file ownership to avoid conflicts

2. **Implementation Phase:**
   - Launch subagents with `is_background: true` for parallel execution
   - Each subagent gets specific files to modify
   - Monitor progress with `read_subagent`

3. **Integration Phase:**
   - Lead agent integrates all subagent changes
   - Resolve any conflicts or dependencies
   - Run full test suite

4. **Validation Phase:**
   - Use test-runner subagent for comprehensive testing
   - Run integration tests
   - Verify application functionality

### Subagent Guardrails

- **File isolation:** Each subagent works on distinct files to avoid conflicts
- **Clear dependencies:** Define task dependencies before launching subagents
- **Integration responsibility:** Lead agent must integrate and test all changes
- **Test coverage:** Subagents must include tests for their implementations
- **Code quality:** Follow project coding standards and patterns

## Before Starting

**If request is unclear, ask these questions:**
- What type of application?
- What are the basic features?
- Who will use it?

**Use defaults, add details later.**

## Edge Case Handling
- **Vague requirements:** When request lacks detail, ask 3 minimum questions (purpose, users, features)
- **Conflicting constraints:** Highlight trade-offs when requirements conflict (e.g., speed vs. cost)
- **Tech stack uncertainty:** Recommend stack based on project type, get user approval before proceeding
- **Scope creep:** Define MVP boundaries clearly, defer non-essential features to later
- **Resource constraints:** Consider development time, team size, budget when recommending approaches

## Failure Modes
- **Wrong tech stack:** Choosing inappropriate stack leads to rework - validate against requirements
- **Over-engineering:** Building too much complexity for MVP - start simple, iterate
- **Missing dependencies:** Incomplete setup causes runtime errors - verify all dependencies install
- **Broken build:** Code doesn't run immediately - test build before marking complete
- **Poor structure:** Disorganized codebase hinders maintenance - follow established patterns

## Performance Considerations
- **Setup time:** Complete initial project structure within 5-10 minutes
- **Build time:** Optimize for fast initial builds (avoid heavy dependencies initially)
- **Development velocity:** Choose stack with good tooling and hot reload for rapid iteration
- **Bundle size:** Consider initial bundle size impact for web projects

## Security Notes
- **Dependency security:** Audit initial dependencies for known vulnerabilities
- **Secrets management:** Never commit API keys, use environment variables from start
- **Authentication defaults:** Don't include hardcoded credentials or default passwords
- **HTTPS enforcement:** Configure SSL/TLS from the start for web applications
- **Input validation:** Include basic validation patterns in initial scaffolding

## Guardrails
- Always clarify requirements before starting
- Get user approval for tech stack choices
- Use established project patterns when available
- Create proper project structure from the start
- Ensure code is immediately runnable

## Related Skills

- `brainstorming` - Socratic questioning protocol and user communication
- `architecture` - Architectural decision-making framework
- `plan-writing` - Structured task planning with clear breakdowns
- `database-design` - Database design principles and decision-making
