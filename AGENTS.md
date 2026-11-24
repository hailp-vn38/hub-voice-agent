# AGENTS.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:

- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

## Code organization

### Short files

- One file, one responsibility.
- Keep a file under about 300 lines, not counting tests. A file over 400 lines must be split into submodules.
- The same limit applies to `.slint` files and to test files.

### Splitting into submodules

- Split a long `foo.rs` into a `foo/` directory: `foo/mod.rs` plus one submodule per responsibility.
- `mod.rs` declares submodules and exposes the module's API. Keep logic in the submodules.
- Keep items private by default. Use `pub(crate)` or `pub` only for what callers outside the module need, and re-export it from `mod.rs`.
- Do not create a submodule for a single small function. Split by responsibility, not by line count alone.
- When splitting an existing file, move code without changing behavior, in its own commit. Tests must pass before and after.

### Tests in separate files

- Never write an inline `mod tests { ... }` block in a source file.
- In a directory module, `foo/mod.rs` declares `#[cfg(test)] mod tests;` and the tests live in `foo/tests.rs`.
- For a single-file module `bar.rs`, declare:

  ```rust
  #[cfg(test)]
  #[path = "bar_tests.rs"]
  mod tests;
  ```

  The tests live in `bar_tests.rs`, next to `bar.rs`.
- When tests outgrow one file, use a `foo/tests/` directory: `tests/mod.rs` plus one file per scenario.
- Tests that exercise only a crate's public API go in `crates/<crate>/tests/`.

### UI (Slint)

- One screen per `.slint` file under `apps/hub/ui/screens/`.
- Shared components go in `apps/hub/ui/components/`, one component or one small group per file.

## Agent skills

### Issue tracker

Issues and specs live as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five canonical labels (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`), recorded as a `Status:` line in each issue file. See `docs/agents/triage-labels.md`.

### Adding a feature

Thêm tính năng theo checklist `docs/adding-a-feature.md`; định tuyến nằm ở `crates/hub-core/src/features/registry.rs` (ADR 0015).

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
