# Codebase Simplification & Quality Audit

You are conducting a comprehensive analysis to reduce code, eliminate duplication, and improve quality. Your goal is to find opportunities to simplify the codebase while identifying architecture violations and design problems.

## The "From Scratch" Philosophy

**Critical mindset:** When analyzing existing code, constantly ask:

> "If I were redoing this from scratch with what I know now, what design would I have actually ended up with?"

**Do NOT:**
- Bias toward existing code just because it exists
- Preserve the status quo by default
- Accept design decisions made during iterative development as correct
- Think something is "right" just because code implements it

**DO:**
- Challenge every design decision critically
- Question why multiple types/methods exist for similar things
- Consider whether specialized methods should be ONE general method with parameters
- Ask if types can be consolidated or eliminated
- Think about what you'd build if starting fresh today

## Critical Anti-Patterns to Avoid

### ❌ DO NOT Suggest "Extract Helper" as a Solution

**Bad approach:** "Extract the core logic into a helper, have the methods call it"
- Result: Keeps all methods intact but now as thin wrappers around the real method
- Makes code MORE complex with additional indirection

**Good approach:** "Collapse these methods into ONE method with parameters"
- Result: Callers determine what params to pass to get what they need
- Single method, less code, clear contract

**Exception:** Keep distinct methods when their purposes are fundamentally different (e.g., `sanitize_job_id` vs `sanitize_username` - different enough to warrant separate names).

### ❌ DO NOT Accept Type Proliferation

**When you see:** Multiple similar types (e.g., `RemoteFileInfo` vs `OutputFile`)

**Don't think:** "They return different types, so keep both methods"

**Do think:** "Why do we have 2 types? Can we unify them? Do callers need different data, or just access different fields?"

**Approach:** ONE method returning complete info where callers access what they need, rather than multiple specialized methods with built-in filtering.

### ❌ DO NOT Preserve Complexity

**When you see:** Similar methods with slight variations

**Don't think:** "Each serves a specific use case, keep all"

**Do think:** "Could this be ONE method where differences are parameters? What would the from-scratch design look like?"

## What to Look For

### Dead & Unused Code
- Code never called in production
- Orphaned utilities/services never imported
- Code left for "future features" (remove it - rebuild when needed)
- Test-only code in production files
- Methods deleted upstream but wrappers remain

### Duplicate Implementations
- Nearly identical code paths (collapse into ONE with parameters)
- Same pattern repeated 3+ times
- Parallel systems solving the same problem
- Debug modes that redo non-debug work plus extras
- Multiple similar types that should be unified

### Thin Wrappers & Over-Abstraction
- Methods that just pass through to another method
- 95%+ pass-through layers (delete the layer)
- Wrapper methods adding nothing but indirection
- Abstraction layers called by only 1-2 places

### Code Smells
- Really large files (suspicious design)
- Really small files (code misplaced or unnecessary)
- Long method names (4+ words) without good reason
- Inconsistent naming (get_ vs fetch_ vs load_ for same operation)
- Special case handling that could be generalized
- Fallback code (fail fast instead - expose errors, don't hide them)
- Silent failures (errors swallowed without propagation)
- Magic strings/numbers (hardcoded values appearing 5+ times)

### Architecture Violations
- Module boundary violations (code in wrong module)
- Business logic in UI components
- UI concerns in domain/infrastructure layers
- Security violations (credential persistence, unsafe input)
- Platform-inappropriate patterns (browser tricks in desktop apps)

### Iterative Development Cruft
- "Legacy", "old format", "backwards compat" comments
- Version handling code (schema_version, format_type fields)
- Complex data transformations that seem unnecessary
- Compatibility layers for requirements that no longer exist
- Migration accumulation (same table modified repeatedly)
- "Death by patches" - code that works but is far more complex than needed

### Type & API Design Issues
- Multiple types for similar concepts (question why they exist)
- Specialized methods that could be ONE general method
- Return types that differ only in filtering (return complete data instead)
- Inconsistent type naming (_Info vs _Data vs _Entry vs _File)
- Custom result types duplicating existing patterns

### Missing Unification
- Ad-hoc solutions where centralized systems exist
- Inconsistent use of established infrastructure
- One-off implementations (even if only 10-20 lines) when unified systems available
- Pattern duplication across similar contexts (stores, components, modules)

## Investigation Strategy: Sub-Agent Orchestration

### Initial Wave (Launch 8-10 Agents in Parallel)

Use Task tool with `subagent_type=Explore` for initial investigation:

1. **Module Boundary Violations** - Functions/code in wrong modules
2. **Duplication Detective** - Repeated patterns, string literals, similar methods
3. **Architecture Auditor** - Violations of documented principles (read docs first)
4. **Type Proliferation** - Multiple types for similar things, questionable type distinctions
5. **Method Naming & Consolidation** - Similar methods that could collapse, naming inconsistencies
6. **Anti-Pattern Hunter** - Try/fallback patterns, magic values, god functions, silent failures
7. **Dead Code Scanner** - Unused code, orphaned utilities, future-feature code
8. **Thin Wrapper Detector** - Pass-through methods, abstraction layers adding no value
9. **Frontend Architecture** - Business logic in UI, mock data, IPC boundary violations
10. **Iterative Cruft Detector** - Legacy code, compatibility layers, version handling

### Follow-Up Wave (Based on Findings)

**After initial agents report:**
- Spawn additional Explore agents to investigate suspicious patterns deeper
- Use review-refactor agent (`.claude/agents/review-refactor.md`) for flagged modules needing deep analysis
- Research related issues in adjacent modules
- Trace root causes of duplication
- Verify findings with targeted searches

**Example:** If agent finds similar methods, spawn follow-up to:
- Find all call sites
- Understand what callers actually need
- Determine if ONE method with parameters would work
- Identify what types could be unified

## Critical Analysis Guidelines

### Question Everything

**When you find similar methods:**
- Don't default to "extract helper"
- Ask: "Could this be ONE method with parameters?"
- Check: "What do callers actually need from this?"
- Consider: "What would from-scratch design look like?"

**When you find multiple types:**
- Don't default to "keep both, different purposes"
- Ask: "Why do we have 2 types? Can we unify?"
- Check: "Do they represent the same concept at different stages?"
- Consider: "Would ONE type with optional fields work?"

**When you find specialized methods:**
- Don't default to "each serves a use case"
- Ask: "Could callers access fields from complete response?"
- Check: "Is the specialization filtering or actual different behavior?"
- Consider: "Would ONE method returning all data be simpler?"

### Evaluate Simplification Value

**Good simplification removes:**
- Lines of code
- Duplicate patterns
- Ad-hoc solutions (alignment with unified systems)
- Thin wrappers/indirection
- Complexity and cognitive load

**Remember:** Removing 10 lines that eliminates ad-hoc code and uses unified infrastructure is MORE valuable than removing 100 lines of genuinely needed logic.

## Output Format

### Report Structure

**DO NOT include:**
- Phased implementation plans
- Prioritized lists
- Implementation order recommendations
- Final summaries restating findings

**DO include:**
- Executive summary of findings
- Each finding ONCE with your recommendation
- Clear evidence (file:line locations)
- WHY it matters
- Proposed solution (from-scratch thinking)

### Finding Format

```
[Category]: [Brief Description]

Location: path/to/file.ext:line

Finding: [What exists now and why it's problematic]

From-Scratch Question: [The critical design question to ask]

Recommendation: [Proposed solution - collapse, unify, delete, etc.]
```

### For Findings Needing Discussion

When you're uncertain or multiple approaches are valid:

```
[Category]: [Brief Description]

Location: path/to/file.ext:line

Finding: [The issue]

Options:
- Option A: [Description]
- Option B: [Description]

Recommendation: [Your analysis of which option and why]

Needs Decision: [What user must confirm]
```

### Organize Findings Into

**Section 1: Definite Deletions**
- Dead code, orphaned utilities, unused methods
- Clear cut - no discussion needed

**Section 2: Definite Consolidations**
- Duplicate types/methods to merge
- Clear collapse opportunities

**Section 3: Type & Method Design Questions**
- Type proliferation to investigate
- Methods that might collapse to ONE
- Naming inconsistencies

**Section 4: Architecture & Quality Issues**
- Module boundary violations
- Anti-patterns and code smells
- Missing unification opportunities

**Section 5: Needs Discussion**
- Multiple valid approaches
- Unclear best solution
- Decisions requiring user input

## Agent Instructions Template

### For Initial Investigation Agents:

```
Analyze [area] for [specific focus].

Apply the "from scratch" philosophy:
- Question why code exists as-is
- Don't accept status quo by default
- Look for consolidation opportunities

Search for:
[List specific patterns from Code Smells above]

Report format:
- Location (file:line)
- What exists
- Why it's problematic
- From-scratch question
- Recommendation

Do NOT:
- Make code changes
- Create implementation plans
- Extract helpers as solutions - suggest collapsing instead
```

### For Follow-Up Agents:

```
Deep-dive [module/pattern] based on initial finding:
[What was found]

Questions to answer:
- Can methods collapse to ONE with parameters?
- Can types unify?
- What do callers actually need?
- What would from-scratch design look like?

Provide evidence and recommendation.
```

## Success Criteria

A successful audit:
- Uses 8-10 parallel agents initially (Explore type)
- Spawns targeted follow-ups based on findings
- Applies "from scratch" thinking throughout
- Questions type proliferation and method specialization
- Recommends method collapsing over helper extraction
- Challenges status quo assumptions
- Identifies both code reduction AND quality improvements
- Organizes findings into clear categories
- Provides specific evidence (file:line)
- Explains WHY each issue matters
- No implementation plans - just findings and recommendations

## Final Report Requirements

1. **Executive Summary** - High-level findings, total reduction potential
2. **Findings by Category** - Each finding once, with recommendation
3. **No Implementation Planning** - User will decide what to implement and when
4. **Evidence-Based** - Specific locations, clear explanations
5. **"From Scratch" Perspective** - Every recommendation challenges existing design

Present findings for discussion and decision-making, not for immediate implementation.
