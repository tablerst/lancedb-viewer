---
name: repo-memory-curator
description: Govern and maintain repository-scoped memory under memories/repo with a closed institutional schema. Use only when the user explicitly asks to curate repo memory, clean up or migrate repo-memory notes, enforce note taxonomy, merge duplicates, retire stale notes, normalize dates, distill durable findings, audit dev_docs boundaries, or update repo_memory_current_state_index.md without changing unrelated code.
---

# Repo Memory Curator

Use this skill only for explicit repository-memory governance. Treat `memories/repo/` as the canonical repository-scoped memory store for this workspace.

This is not an autosave workflow. Do not silently add memories during ordinary coding, planning, or debugging tasks.

Consult [repo-memory-rubric.md](./references/repo-memory-rubric.md) when a keep, merge, demote, retire, or note-writing decision is not obvious.

## Mission

Keep repository memory compact, durable, source-aligned, and useful across future sessions.

## Storage Boundary

- Operate under `memories/repo/` by default.
- Treat `memories/repo/repo_memory_current_state_index.md` as the entry point and schema-controlled registry.
- Do not write repository memory to `.agenta/`, `.serena/memories/`, Codex user/workspace memories, or tool-private caches.
- Treat `.agenta/` as task ledger and recovery context, not durable repo-memory storage.
- Treat `.serena/memories/` as Serena project-tool memory, not the canonical repo-memory store.
- Treat `dev_docs/memory/` or `dev_docs/memory-module/` as product memory-module documentation, not this skill's storage area.
- If `memories/repo/` or the index is missing during an explicit curation request, create the minimal directory and index before adding notes.

## Institutional Model

Use a closed note taxonomy:

- `contract`: durable current rule, compatibility boundary, or validated positive repository pattern.
- `finding`: current but incomplete observation that needs caution, validation, or tighter boundaries.
- `historical`: dated evidence, profiling, migration context, or background that must not be treated as live implementation truth.
- `pointer`: compact navigation note to a stable supporting reference when the durable takeaway is already captured or the location itself is the durable fact.

Use a closed confidence layer:

- `high`: strong enough to guide future work as current repo knowledge.
- `partial`: useful but incomplete, conditional, or under-verified.

Allowed active index tags:

- `[contract][high]`
- `[finding][partial]`
- `[historical]`
- `[pointer]`

Retirement is a lifecycle state, not an active note type. Remove retired notes from active reading lists and record merge or retirement traceability only when it helps future maintenance.

## Hard Boundaries

- Do not create or edit user memory, session memory, Codex memory, Agenta task records, or Serena memories unless the user explicitly asks for that separate system.
- Do not change application code, tests, or product docs just to make old memory remain true.
- Treat current code, tests, root guidance, and canonical docs as the source of truth. If repo memory conflicts with them, update or retire the memory.
- Prefer updating an existing note over creating a new note when the topic already exists.
- Keep repo memory as distilled long-lived knowledge, not task tracking, chat transcripts, or conversation residue.
- Keep formal repo-memory notes in English unless a note explicitly declares another language.

## What Belongs in Repo Memory

Keep only information that remains useful across sessions and cannot be recovered as reliably from current repository state alone.

Good candidates:

- Durable contracts, compatibility boundaries, and validated repository patterns.
- Recurring gotchas with clear application boundaries.
- Historical evidence that still helps future decisions when clearly marked as dated.
- Rare pointers to actively maintained supporting docs when the repo-memory note still carries a self-contained durable takeaway.

Do not save:

- Plain code structure, file paths, or line numbers that should be re-read from current code.
- Git history, PR lists, temporary debugging transcripts, current task state, or TODOs.
- Raw stack traces, long logs, or step-by-step repair transcripts.
- Content already fully covered by `AGENTS.md`, root guidance, or canonical docs unless the repo memory adds a narrower durable takeaway.
- Personal style preferences or user-specific interaction feedback.
- Bare pointers to task-specific execution plans or archives without a self-contained durable takeaway.

## File Naming

Target canonical filenames:

- `contract__<topic>.md`
- `finding__<topic>.md`
- `historical__<topic>.md`
- `pointer__<topic>.md`

Filename rules:

- Use lower-case kebab-case after the prefix.
- Make names topic-centered, not task-log-centered.
- Include dates in the body when needed; put dates in filenames only when the date is part of the note identity.
- Avoid vague names such as `misc`, `notes`, `tmp`, `review`, or ticket-only identifiers unless the ticket itself is the durable retrieval handle.
- In routine curation, avoid mass-renaming untouched legacy notes. In a dedicated institutional migration pass, rename touched notes to the canonical pattern.

## Index Convention

Keep `repo_memory_current_state_index.md` compact and registry-like.

Default section order:

1. Schema baseline.
2. Start-here working set.
3. Active contracts.
4. Active findings.
5. Stable pointers.
6. Historical background.
7. Merged or retired log.

Each active entry should fit on one line:

```markdown
- `[contract][high]` `contract__import-cycle-lazy-exports.md`: package-level eager import traps and the preferred lazy-export pattern.
```

Keep the start-here set intentionally small. Prefer about 8 to 12 notes unless the user explicitly wants a larger surface.

## dev_docs Linkage Policy

- Treat `dev_docs/` primarily as working documentation and historical repository context, not as default long-term memory backing.
- Prefer distilling the lasting conclusion into repo memory instead of pointing readers at task-specific `dev_docs` files.
- Link to active, intentionally maintained `dev_docs/` guides sparingly.
- Treat archived or deprecated execution plans as historical background, not primary support for active memory.
- Never make a repo-memory note depend on a `dev_docs` file remaining current. The note must stand on its own.

## Operating Procedure

1. Read `memories/repo/repo_memory_current_state_index.md` first.
2. Decide whether the pass is routine curation, targeted note work, or explicit institutional migration.
3. Read only the notes relevant to the requested scope.
4. Identify overlap, duplication, contradictions, stale anchors, vague wording, and legacy taxonomy drift.
5. Verify live claims against current code, tests, root guidance, or canonical docs when a note refers to present behavior.
6. Decide for each touched note: `contract`, `finding`, `historical`, `pointer`, merge, or retire.
7. Apply `high` or `partial` only where the active schema allows it.
8. Convert relative dates to absolute `YYYY-MM-DD` form when the date can be resolved from context.
9. Rewrite touched notes in concise bullet style and align them to the institutional schema.
10. In an explicit migration pass or substantial rewrite, migrate touched filenames and index entries to canonical naming and tagging.
11. Update the index last so it reflects the final file set, schema, and reading order.

When consulting `dev_docs/`, search narrowly:

- Search current working docs only when current documentation is relevant.
- Search archives only when the user explicitly asks for historical context or when a current note references historical background.
- Avoid broad searches over all of `dev_docs/` during ordinary curation because stale archives can pollute decisions.

## Writing Rules

- One note should cover one coherent topic.
- Prefer concise bullets over long prose.
- Every active note must fit exactly one allowed note type.
- Preserve validated positive guidance, not just warnings, when it reflects a durable repository pattern.
- If a note is time-sensitive, mark it as dated evidence rather than evergreen truth.
- If detailed narrative is still valuable, keep repo memory distilled and link only to stable supporting references.
- Do not manufacture rationale or application rules beyond the evidence. If confidence is limited, say so.
- Avoid file-and-line assertions unless they are only verification leads and not the durable takeaway.
- Prefer topic-first naming and structure over incident-first storytelling.

## Decision Logic

Keep or upgrade a note when it contains a durable conclusion future work is likely to need, especially when it captures a validated boundary, contract, failure mode, or positive pattern.

Merge notes when they describe the same underlying topic and separate files would fragment recall or duplicate maintenance.

Demote a note to `[historical]` when it remains useful as dated context but should no longer guide implementation decisions as live truth.

Retire or fold away a note when it is redundant, derivable from current code or docs, superseded by a better canonical note, or no longer meaningful after repository evolution.

## Migration Rules

- Legacy `current-high` usually migrates to `[contract][high]`, but verify it is a durable current rule rather than a background summary.
- Legacy `current-partial` usually migrates to `[finding][partial]`, unless the uncertainty is resolved and it should become `[contract][high]`.
- Legacy `historical-process` usually migrates to `[historical]`.
- Legacy filenames may remain during transition, but touched notes in an institutional cleanup should move toward canonical `type__topic.md` names.
- Do not preserve both legacy and canonical copies of the same note.

## Required Quality Checks

Before finishing, verify that:

- Every touched note became clearer, smaller, more durable, or was removed.
- No touched active note duplicates another active note.
- Every touched active note fits the allowed public schema.
- Confidence tags match certainty and freshness.
- Newly written dates are absolute.
- Current notes do not present historical anchors as live facts.
- The index reflects merges, retirements, current recommended reading order, and the institutional schema.
- No writes escaped `memories/repo/` unless the user explicitly expanded scope.

## Response Requirements

When finishing a curation pass, report:

- Which notes were created, updated, merged, demoted, or retired.
- Why the biggest changes were made.
- Any ambiguities or evidence gaps that still need human judgment.

## Default Posture

Be conservative. A smaller, cleaner repo-memory set is better than an encyclopedic one that drifts out of sync with reality.
