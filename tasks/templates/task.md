# Task: [Brief Title]

## Objective
[One sentence goal describing the user-facing outcome]

## Context
- **Starting state**: [What exists now - specific features/capabilities]
- **Delivered state**: [What will exist after - specific outcomes]
- **Foundation**: [What we're building on - existing systems/patterns]
- **Dependencies**: [Required prerequisite work]
- **Testing approach**: [How this aligns with NAMDRunner testing philosophy (as outlined in `docs/CONTRIBUTING.md#testing-strategy`)]

## Implementation Plan

### Critical Priority (Blockers)
- [ ] **Core Component 1**
  - [ ] Specific implementation detail
  - [ ] Integration requirement
  - [ ] Validation step

- [ ] **Core Component 2**
  - [ ] Specific implementation detail
  - [ ] Security consideration
  - [ ] Error handling requirement

### High Priority (Core Functionality)
- [ ] **Feature Set 1**
  - [ ] Implementation detail
  - [ ] Testing requirement

### Medium Priority (Enhancements)
- [ ] **Polish/Optimization Items**
  - [ ] Enhancement detail
  - [ ] User experience improvement

## Success Criteria

### Functional Success
- [ ] [User-facing workflow works end-to-end]
- [ ] [Integration with existing systems seamless]
- [ ] [Error scenarios handled gracefully]

### Technical Success
- [ ] [Performance/reliability requirements met]
- [ ] [Security/safety requirements satisfied]
- [ ] [Follows established architectural patterns]

### Quality Success
- [ ] [All tests pass - aligned with NAMDRunner testing philosophy]
- [ ] [Code quality standards maintained]
- [ ] [Documentation updated with implementation details]

## Key Technical Decisions

### Why [Decision 1]
- **Reasoning**: [Technical rationale and benefits]
- **Alternatives considered**: [What we didn't choose and why]
- **Trade-offs**: [What we're optimizing for vs what we're accepting]

### Why [Decision 2]
- **Reasoning**: [Technical rationale]
- **Integration**: [How this works with existing systems]

## Integration with Existing Code

### Leverage Existing Patterns
- **Use [System 1]**: [How to build on existing work without duplication]
- **Follow [Pattern 1]**: [Established conventions to maintain consistency]
- **Apply [Standard 1]**: [Security/validation patterns to reuse]

### Where to Hook In
```rust
// Existing functions to enhance (don't rebuild)
existing_function() // Add: new capability description

// New functions to add (follow established patterns)
new_function() // Purpose and integration point
```

## References
- **NAMDRunner patterns**: [Internal architecture/testing/security docs]
- **Implementation files**: [Key source files and functions to modify/reference]
- **Specific docs**: [What documentation from `docs/` to check first]
- **Python reference (optional)**: [Proven patterns to adapt from reference implementation]

## Progress Log
[Date] - [What was done, blockers, decisions made]

## Completion Process
After implementation and testing:
- [ ] Run code review using `.claude/agents/review-refactor.md`
- [ ] Implement recommended refactoring improvements
- [ ] Update and archive task to `tasks/completed/[task-name].md`
- [ ] Update `tasks/roadmap.md` progress/changes to future work
- [ ] Update `docs/ARCHITECTURE.md` with implementation details

## Open Questions
- [ ] Question 1
- [ ] Question 2