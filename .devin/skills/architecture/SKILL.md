---
name: architecture
description: Architectural decision-making framework. Requirements analysis, trade-off evaluation, ADR documentation
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# Architecture Decision Framework

## Purpose

Architectural decision-making framework for Wireframe-AI: analyze requirements, evaluate trade-offs, select appropriate patterns, and document decisions with ADRs. Ensures architecture is driven by requirements, informed by trade-offs, and captured for future reference.

## When to Use

**Use this skill when:**
- Making architecture decisions for new features or systems
- Analyzing system design and architectural patterns
- Documenting architectural trade-offs with ADRs
- Evaluating technology choices and patterns
- Designing system structure and component relationships
- Refactoring existing architecture

**Do NOT use when:**
- Making simple implementation decisions (use `/implementation` instead)
- Choosing between equivalent alternatives (use `/karpathy-guidelines` for simplicity)
- Documenting code-level decisions (use code comments instead)

## Protocol

### Phase 1: Requirements Analysis

1. **Understand requirements**
   - Read `context-discovery.md` for questions to ask
   - Clarify functional requirements
   - Identify non-functional requirements (performance, security, scalability)
   - Understand constraints (budget, timeline, team expertise)

2. **Classify project type**
   - MVP: Minimum viable product, time-sensitive
   - SaaS: Multi-tenant, subscription-based
   - Enterprise: Complex, security-critical
   - Internal tool: Simpler, rapid iteration

3. **Identify key constraints**
   - Scale requirements (users, data volume, throughput)
   - Team constraints (size, expertise, availability)
   - Budget constraints (infrastructure, development time)
   - Technology constraints (existing stack, compatibility)

### Phase 2: Trade-off Analysis

1. **Read trade-off framework**
   - Reference `trade-off-analysis.md` for ADR templates
   - Use trade-off framework for decision evaluation
   - Consider multiple alternatives

2. **Evaluate alternatives**
   - List 2-3 viable options
   - Analyze pros and cons of each
   - Consider short-term vs long-term implications
   - Assess alignment with constraints

3. **Select pattern**
   - Read `pattern-selection.md` for decision trees
   - Choose pattern based on requirements
   - Avoid anti-patterns
   - Validate against examples in `examples.md`

### Phase 3: Decision Documentation

1. **Write ADR for significant decisions**
   - Use ADR template from `trade-off-analysis.md`
   - Document context, decision, consequences
   - Capture trade-offs and rationale
   - Store in ADR log

2. **Validate architecture**
   - Run validation checklist
   - Ensure requirements are met
   - Verify simpler alternatives were considered
   - Confirm team expertise matches chosen patterns

### Phase 4: Implementation Guidance

1. **Provide implementation guidance**
   - Reference `patterns-reference.md` for implementation details
   - Suggest specific patterns and libraries
   - Identify integration points
   - Note potential risks and mitigations

2. **Monitor and iterate**
   - Track architecture decisions in production
   - Gather feedback on trade-offs
   - Update ADRs if assumptions change
   - Refine architecture based on learnings

## Selective Reading Rule

**Read ONLY files relevant to the request!**

| File | Description | When to Read |
|------|-------------|--------------|
| `context-discovery.md` | Questions to ask, project classification | Starting architecture design |
| `trade-off-analysis.md` | ADR templates, trade-off framework | Documenting decisions |
| `pattern-selection.md` | Decision trees, anti-patterns | Choosing patterns |
| `examples.md` | MVP, SaaS, Enterprise examples | Reference implementations |
| `patterns-reference.md` | Quick lookup for patterns | Pattern comparison |

---

## Related Skills

| Skill | Use For |
|-------|---------|
| `@[skills/database-design]` | Database schema design |
| `@[skills/api-patterns]` | API design patterns |
| `@[skills/deployment-procedures]` | Deployment architecture |

---

## Core Principle

"Simplicity is the ultimate sophistication."

- Start simple
- Add complexity ONLY when proven necessary
- You can always add patterns later
- Removing complexity is MUCH harder than adding it

---

## Validation Checklist

Before finalizing architecture:

- [ ] Requirements clearly understood
- [ ] Constraints identified
- [ ] Each decision has trade-off analysis
- [ ] Simpler alternatives considered
- [ ] ADRs written for significant decisions
- [ ] Team expertise matches chosen patterns

## Edge Case Handling
- **Evolving requirements**: Architecture must accommodate change - design for extensibility
- **Scale uncertainty**: Unknown future scale - design for horizontal scaling when possible
- **Team size changes**: Architecture for small team may not scale - consider team coordination overhead
- **Technology shifts**: New tech may invalidate choices - design for modularity and replaceability
- **Budget constraints**: Ideal architecture too expensive - prioritize based on ROI

## Failure Modes
- **Over-engineering**: Building too much complexity for current needs - start simple, iterate
- **Under-engineering**: Not enough structure for growth - identify minimum viable architecture
- **Wrong abstractions**: Leaky abstractions cause pain everywhere - validate abstractions with real usage
- **Tight coupling**: Changes ripple through system - design clear boundaries and interfaces
- **Ignoring constraints**: Architecture doesn't fit team/scale/budget - validate against real constraints

## Performance Considerations
- Latency targets: Design architecture to meet SLA requirements
- Throughput capacity: Plan for peak load, not average load
- Data locality: Consider where data lives vs. where it's processed
- Caching strategy: Design caching at appropriate layers (CDN, application, database)
- Database performance: Design schema and queries for expected query patterns

## Security Notes
- **Defense in depth**: Security at multiple layers (network, application, data)
- **Least privilege**: Minimize permissions for all components and services
- **Data classification**: Design based on data sensitivity (public, internal, confidential)
- **Audit trails**: Log access and modifications to critical resources
- **Secure defaults**: Default to secure configurations, opt-in to insecure features

## Common Pitfalls

| Pitfall | Why Bad | Correct Approach |
|---------|---------|------------------|
| Over-engineering for current requirements | Wastes time, adds complexity | Start simple, add complexity only when necessary |
| Choosing patterns without understanding trade-offs | May not fit actual needs | Analyze trade-offs before selecting patterns |
| Not documenting architectural decisions | Lost rationale, difficult to maintain | Write ADRs for significant decisions |
| Ignoring team expertise and constraints | Architecture may not be implementable | Validate against team skills and constraints |
| Premature optimization before understanding scale | Optimizes wrong problems | Understand scale before optimizing |
| Copying architectures without context adaptation | May not fit your context | Adapt architectures to your specific requirements |
| Under-engineering for growth | Requires costly refactoring later | Design for extensibility when needed |
| Tight coupling between components | Changes ripple through system | Design clear boundaries and interfaces |

## ADR Template Example

```markdown
# ADR-001: Use NATS for Inter-Module Communication

## Context
Wireframe-AI needs a message bus for inter-module communication. Modules need to publish and subscribe to events in a decoupled manner.

## Decision
Use NATS as the message bus for inter-module communication.

## Rationale
- NATS is lightweight and fast
- Supports both pub/sub and request/response patterns
- JetStream provides durability when needed
- Simple to operate and scale
- Good Rust client support (async-nats)

## Consequences
- Positive: Decoupled modules, easy to add new modules
- Positive: Built-in load balancing with queue groups
- Negative: Requires NATS server to be running
- Negative: Adds operational complexity
- Mitigation: Use NATS in development, document setup process

## Alternatives Considered
- Kafka: More complex, overkill for current scale
- RabbitMQ: Heavier than NATS, more operational overhead
- Direct HTTP calls: Tight coupling, no pub/sub

## References
- NATS documentation: https://docs.nats.io
- async-nats crate: https://docs.rs/async-nats
```

## Code Examples

**Example: Architecture decision for message bus**

```rust
// Decision: Use NATS for inter-module communication
// ADR-001: Use NATS for Inter-Module Communication

use async_nats::Client;

pub struct Module {
    nats: Client,
    name: String,
}

impl Module {
    pub async fn publish(&self, topic: &str, payload: &[u8]) -> anyhow::Result<()> {
        // Topic naming: namespace.noun.verb
        let full_topic = format!("{}.{}.{}", self.name, topic, "publish");
        self.nats.publish(full_topic, payload.to_vec()).await?;
        Ok(())
    }
}
```

## Integration

**Related skills:**
- **superpowers:architecture-decision-records** - Capture architectural decisions as ADRs
- **superpowers:database-design** - Database schema design
- **superpowers:research-architecture** - Understand codebase structure before making changes

**Workflow context:**
- Use before starting new features or major refactors
- Use when evaluating technology choices
- Use with `/karpathy-guidelines` for simplicity-first approach
- Use with `/wireframe-workflow` for Wireframe-AI specific patterns
