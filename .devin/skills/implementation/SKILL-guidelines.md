---
name: implementation-guidelines
description: Edge cases, performance considerations, and security notes for Wireframe-AI implementation
---

## Edge Case Handling
- **Ambiguous requirements:** When feature description is unclear, ask clarifying questions before planning
- **Scope creep:** Feature grows during implementation - define clear boundaries and change process
- **Dependency conflicts:** New dependencies break existing code - validate in isolation first
- **Team number conflicts:** Multiple teams working simultaneously - use unique team numbers and coordinate
- **Plan drift:** Implementation diverges from plan.md - update plan.md to reflect actual implementation
- **Baseline test failures:** Tests don't pass before starting - fix tests or document known issues
- **Context loss:** Long implementation exceeds context window - rely on persistent plan.md and team file

## Failure Modes
- **Incomplete implementation:** Feature partially implemented but marked complete - verify against checklist
- **Broken build:** Code doesn't compile - stop and fix before proceeding
- **Test failures:** New code breaks existing tests - fix before marking complete
- **Regression bugs:** Changes break existing functionality - run baseline tests before and after
- **Dead code left behind:** Unused code not removed - violates Rule 6, must clean up
- **Missing documentation:** Changes not documented in team file - update team file continuously
- **Handoff incomplete:** Next team cannot understand changes - write comprehensive handoff notes

## Performance Considerations
- **Implementation velocity:** Balance speed with quality - don't rush at expense of correctness
- **Build time impact:** New dependencies may slow builds - measure before and after
- **Test execution time:** New tests increase suite duration - optimize or categorize as integration tests
- **Memory usage:** New features may increase memory footprint - profile for RAM disk impact
- **I/O patterns:** File operations affect RAM disk performance - consider junction/symlink overhead
- **Async overhead:** Tokio async patterns add complexity - ensure proper async/await usage
- **Resource cleanup:** Ensure RAM disks and processes are cleaned up to prevent leaks

## Security Notes
- **Unsafe code review:** All unsafe blocks must be reviewed for memory safety and correctness
- **Windows API security:** Win32 API calls must handle HRESULT errors properly and validate parameters
- **Input validation:** New features must validate all inputs, especially from external sources
- **Secrets scanning:** Ensure no secrets are introduced (use secret-scrubber skill)
- **Resource isolation:** Ensure RAM disk isolation between projects using hash-based subdirectories
- **Process security:** aim_ll.exe runs as user process - verify no privilege escalation
- **Error disclosure:** Error messages should not leak sensitive information (paths, internal details)

## Related Skills

- `plan-writing` - Structured task planning with clear breakdowns
- `architecture` - Architectural decision-making framework
- `rust-pro` - Master Rust 1.75+ with modern async patterns
- `async-tokio-patterns` - Tokio async/await patterns for Rust applications