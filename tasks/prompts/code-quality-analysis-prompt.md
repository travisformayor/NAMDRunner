# Code Quality Analysis: Architecture Violations, Anti-Patterns & Design Problems

You are analyzing a codebase for architecture violations, anti-patterns, and design problems. Your goal is to identify issues that make code fragile, unnavigable, or violate architectural principles—even if the code currently "works."

## Investigation Philosophy

**You're hunting for design problems, not just bugs:**
- Code that works but is fragile
- Patterns that will cause issues when requirements change
- Violations of stated architectural principles
- Things that make you think "why is this here?"
- **BUT: Also report any bugs you spot during investigation**

**Look for "death by a thousand patches":**
- Code that works but is far more complex than it needs to be
- Systems that grew organically and never got cleaned up
- Backwards compatibility for requirements that no longer exist
- The question isn't "does it work?" but "is this the simplest way to solve the current problem?"

**Trust your instincts:** If something feels wrong, it probably is. If a module name doesn't match what it does, that's a problem. If you see the same pattern 3+ times, it should be shared.

## Core Principles (The "Vibes" to Enforce)

### 1. Explicit Over Implicit
Code should have clear contracts and know exactly what it wants. No guessing, no fallback paths, no "try things until something works."

### 2. Single Source of Truth
Every piece of knowledge should exist in exactly one place. Changes should require updates in one location, not hunting through the codebase.

### 3. Module Boundaries Are Sacred
Modules should do what their name suggests and nothing else. Misplaced code makes the codebase unnavigable.

### 4. Architecture Constraints Must Be Enforced
Code should follow documented architectural principles. Violations compound and spread.

### 5. Platform-Appropriate Patterns
Use patterns appropriate for your platform. Desktop apps use native APIs, not browser hacks. Wrong patterns feel janky and break on edge cases.

### 6. Iterative Development Cruft Must Be Cleaned
Working code that accumulated patches over time should be refactored. You're building for current requirements, not maintaining complexity for historical ones.

## Research Strategy: Use Sub Task Agents Extensively

**Launch parallel Task sub agents to investigate different areas simultaneously.** Maximize what you can discover with your context budget.

### Initial Investigation Wave (Launch in Parallel)

1. **Module Boundary Violations Agent**
   - Check EVERY module to see if it's doing more than its name suggests
   - Look for business logic in utility/validation modules
   - Look for infrastructure concerns in domain modules
   - Search for functions that feel "out of place"
   - Example findings: Validation module generating paths, SSH module with domain knowledge

2. **Duplication Detective Agent**
   - Find repeated string literals (especially paths, file names, directory names)
   - Find duplicated logic (same pattern in 3+ places)
   - Search for similar-looking functions that should be unified
   - Count occurrences of common domain terms
   - Example findings: Directory names hardcoded 48 times, identical error handling blocks

3. **Architecture Auditor Agent**
   - Read architecture docs to understand rules (check `docs/README.md`, `docs/ARCHITECTURE.md`, `CLAUDE.md`, `docs/CONTRIBUTING.md`, etc.)
   - Search for patterns that violate documented rules
   - Check for security violations (credential persistence, unsafe input handling)
   - Verify separation of concerns (IPC boundaries, offline-first, etc.)
   - **Check if implementation has diverged from documented architecture**
   - Example findings: App reading from temporary directories, IPC boundary violations, docs say "simple cache" but implementation is complex

4. **Anti-Pattern Hunter Agent**
   - Look for "try/fallback" patterns (loops with continue, multiple if/else paths trying things)
   - Find "magic number/string" patterns (unexplained constants, hardcoded values)
   - Search for god functions (100+ lines, doing too many things)
   - Look for silent failures (errors ignored, missing propagation)
   - Find incomplete features (stubs, setTimeout placeholders, mock data in production)

5. **Cross-Cutting Concerns Agent**
   - Find scattered error handling that should be centralized
   - Look for logging/debugging inconsistencies
   - Check for security validation gaps
   - Find configuration scattered across files
   - Look for resource management issues

6. **Frontend Architecture Agent**
   - Business logic in UI components
   - Mock data or stubs in production code
   - Orphaned code (utilities/services never imported)
   - Type safety issues (any types, missing interfaces)
   - IPC boundary violations (direct backend calls bypassing abstraction)

7. **Data Model Archaeology Agent**
   - Investigate database schema, migrations, data transformations
   - Look for version-handling code, backwards compatibility layers
   - Check if data types/structs match schema cleanly
   - Find redundant storage (same data in multiple columns, JSON duplicating relational data)
   - Look for complex queries that seem too complex for what they do
   - Check migration files for excessive changes to same tables
   - Find transformation utilities that reshape data from DB before use
   - Report: "This data model has grown organically and accumulated cruft"

8. **Iterative Cruft Detector Agent**
   - Look for "compatibility layers" and "format versioning" in production code
   - Find transformation utilities that shouldn't need to exist
   - Spot comments mentioning "legacy", "old way", "deprecated but still used"
   - Check for feature flags that are always on/off (dead code)
   - Look for fields like `schema_version`, `format_type`, `data_version`
   - Find code that checks "which version/format is this?"
   - Search for multiple serialization/deserialization transforms (data reshaped multiple times)
   - Report: "These patches on patches suggest need for clean-slate redesign"

### Follow-Up Investigation (Spawn Based on Findings)

**After initial sub agents report back, spawn additional focused review-refactor sub agents to investigate:**
- Suspicious patterns that need deeper analysis
- Related issues in adjacent modules
- Root causes of duplication
- Architecture violations that might have spread

**Example:** If a sub agent finds duplicated validation logic, spawn a sub agent to trace all validation call sites and determine the correct centralization point.

#### How to Launch

After initial sub agents report, launch the review-refactor sub agent for **targeted deep analysis** (not initial wave):
- Modules flagged for boundary violations
- Subsystems with high duplication
- Areas with suspected iterative cruft
- Any module where initial findings suggest structural problems

**Instruction template to review-refactor sub agent:**
```
Analyze [specific module/subsystem] for refactoring opportunities.
Initial investigation found: [specific findings from initial agents].
Focus on: [whether it's duplication, boundaries, cruft, etc.]
```
Use the agent defined in `.claude/agents/review-refactor.md` as the assigned sub agent

**Why use review-refactor sub agent:**
- Is thorough but context-heavy
- Can't analyze entire codebase in one shot
- Perfect for deep-diving suspicious modules identified by initial agents
- Initial agents identify WHERE to look, review-refactor agent determines WHAT it is

## Code Smell Taxonomy

Organized by WHAT you're looking for (symptoms), not WHY it's bad (principles).

### "Path Guesser" Smell
- Loops trying multiple file locations until one works
- Functions with "possible", "try", "fallback" in the name
- Multiple path constructions for the same logical file
- **Violates:** Explicit Over Implicit

### "Magic Spreader" Smell
- Same string literal in 5+ places
- Duplicated path construction logic
- Hard-coded values that should be configuration
- **Violates:** Single Source of Truth

### "Boundary Violator" Smell
- Validation module that generates/transforms data
- Utility module with business rules
- Data model with UI logic
- Infrastructure module with domain knowledge
- **Violates:** Module Boundaries Are Sacred

### "God Function" Smell
- Functions doing too many things (100+ lines)
- Nested conditionals 4+ levels deep
- Functions that orchestrate, validate, transform, and persist all in one
- **Violates:** Module Boundaries Are Sacred

### "Architecture Rebel" Smell
- Code violating documented rules
- Breaking stated "NEVER do X" principles
- Security violations
- Platform-inappropriate patterns (browser tricks in desktop apps)
- **Violates:** Architecture Constraints Must Be Enforced, Platform-Appropriate Patterns

### "Incomplete Production Code" Smell
- setTimeout used as placeholder for real implementation
- Mock data exported from production files
- Functions that return empty/null in production but work in demo
- Stubs with TODO comments
- **Violates:** Multiple principles (depends on context)

### "Silent Failure" Smell
- Errors ignored without good reason
- Missing error propagation
- unwrap()/expect() in production code (Rust)
- Empty catch blocks
- **Violates:** Architecture Constraints (reliability)

### "Death by Patches" Smell
- Backwards compatibility code for requirements that no longer exist
- Version-handling in production (schema_version, format_type fields)
- Complex data transformations that seem unnecessary
- Migration accumulation (many migrations for same table)
- Comments like "legacy", "old format", "for backwards compatibility"
- Data stored redundantly (relational columns + JSON blob with same data)
- Complex queries for simple questions (get user's jobs needs 4 joins)
- Transformation utilities that exist only to reshape badly-structured data
- **Violates:** Iterative Development Cruft Must Be Cleaned

## Investigation Techniques

### For Each Module/Area:
1. **Launch Task Sub Agent** to deeply investigate that area
2. **Read module name** - What should it do based on the name?
3. **Read function names** - Do they belong in this module?
4. **Check for duplication** - Seen this pattern elsewhere?
5. **Verify architecture** - Following documented rules?
6. **Look for bugs** - Spot any actual defects?

### Useful Search Patterns (Examples):

**General duplication:**
- Same string in many files: `rg '"specific_string"' --type [language]`
- Magic numbers: `rg '\b(1024|100|1000)\b'`

**Anti-patterns:**
- Potential guessing: `rg "for .* in |\.iter\(\).*continue"`
- Error patterns: `rg 'map_err\(|\.ok\(\);$'`
- Incomplete code: `rg 'TODO|FIXME|setTimeout'`
- Debug prints: `rg 'println!|console.log|print\('`

**Iterative cruft:**
- Legacy markers: `rg 'legacy|deprecated|old.*format|backwards.*compat'`
- Version handling: `rg 'schema_version|format_type|data_version|version.*check'`
- Migration accumulation: Check migration file count per table
- Complex transforms: `rg 'serialize.*deserialize|map.*map|reshape|transform.*transform'`

**Platform-inappropriate:**
- Browser tricks in desktop: `rg 'blob.*URL|createObjectURL|<a.*download'`

### Reading Architecture:
- Check project README.md, CLAUDE.md, ARCHITECTURE.md, and CONTRIBUTING.md for stated principles
- Look for stated patterns and anti-patterns
- Understand the documented module structure by reading `docs/README.md`
- Note any security or platform constraints
- **Check if current implementation matches documented architecture**

## Output Format

### For Each Issue Found:

```
[Pattern Type/Smell Name]: [Brief Description]
Location: path/to/file.ext:line-range
The Smell: [What looks wrong - be specific with code examples if helpful]
Principle Violated: [Which core principle or architectural rule this breaks]
Why It Matters: [Impact - fragility, maintenance burden, security risk, etc.]
Proposed Fix: [Describe the proper long-term solution, including any refactoring needed]
```

### For Each Bug Found:

```
BUG: [Brief Description]
Location: path/to/file.ext:line-range
Problem: [What's broken - be specific]
Impact: [Consequences]
Fix: [How to correct it]
```

## Reporting Structure

Organize findings into these sections:

### Section 1: Architecture Violations
Issues that violate documented architectural principles or create security/reliability risks.

### Section 2: Module Boundary Violations
Code in the wrong modules, god functions, infrastructure mixed with domain logic.

### Section 3: Iterative Development Cruft
Systems that accumulated patches over time, backwards compatibility for dead requirements, overly complex data models.

### Section 4: Duplication Patterns
Repeated logic, string literals, or patterns that should be unified.

### Section 5: Anti-Patterns & Code Smells
Magic values, incomplete code, wrong platform patterns, silent failures.

### Section 6: Bugs Discovered
Any actual defects found during investigation (not just design issues).

### Section 7: Positive Findings
What the codebase does well - architectural strengths to preserve.

## Proposed Fix Sequencing

**Organize fixes by dependency order, not by priority or effort:**

### Phase 1: Foundation Changes
- Changes that other fixes depend on
- Example: Creating centralized modules, extracting constants, defining interfaces
- Example: Redesigning data models from scratch (clean-slate replacements for cruft)

### Phase 2: Module Refactoring
- Changes that depend on Phase 1 foundations
- Example: Moving functions to correct modules, breaking up god functions
- Example: Removing backwards compatibility layers after data model redesign

### Phase 3: Duplication Elimination
- Changes that depend on refactored module boundaries
- Example: Using centralized utilities now in correct locations

### Phase 4: Bug Fixes & Cleanup
- Changes that depend on architectural improvements
- Example: Fixing bugs enabled by better abstractions, removing orphaned code

**Do NOT organize by:**
- High/medium/low priority
- Critical/important/nice-to-have
- Quick wins vs long-term improvements
- Effort required

**Focus on:** What's the correct final architecture?

## Agent Instructions Summary

**To each initial investigation agent you spawn, tell them:**
- Their specific investigation focus (from Initial Investigation Wave list)
- What patterns to search for (use Code Smell Taxonomy)
- What to report back (use Output Format)
- To note any bugs they spot
- NOT to make changes, only investigate and report

**To follow-up agents (including review-refactor):**
- What initial agents found
- Specific module/area to deep-dive
- What questions to answer or patterns to trace
- NOT to make changes, only investigate and report

**After agents report back:**
- Synthesize findings across all agents
- Spawn follow-up agents for suspicious patterns (including review-refactor for flagged modules)
- Identify root causes of duplication and cruft
- Trace violations to their sources
- Compile comprehensive final report
- Format report as plain text markdown

## Key Reminders

1. **Use Task agents extensively** - Launch multiple agents in parallel, then follow up based on findings
2. **Use review-refactor as follow-up only** - Launch it on specific flagged modules, not entire codebase
3. **Report bugs you find** - Even though focus is design, bugs are valuable discoveries
4. **Trust your instincts** - If something feels wrong, investigate deeper
5. **Think long-term** - Propose complete proper fixes, not band-aids
6. **Consider blast radius** - How many places would need updating if X changes?
7. **Check documentation** - Architecture violations are only violations if rules are documented
8. **Question complexity** - "Is this the simplest way to solve the CURRENT problem?" not "Does it work?"

## Success Criteria

A successful analysis:
- Uses 8 parallel Task agents initially (all from Initial Investigation Wave)
- Spawns follow-up agents based on findings (including review-refactor for flagged modules)
- Identifies both design problems AND bugs
- Identifies iterative cruft and "death by patches" patterns
- Proposes complete long-term fixes (including clean-slate redesigns where appropriate)
- Organizes fixes by dependency order
- Shows clear evidence (file locations, code patterns)
- Explains WHY each issue matters
- Highlights what's well-architected (positive findings)
