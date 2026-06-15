# Repo Memory Rubric

Use this reference when applying the `repo-memory-curator` skill and a keep, merge, demote, retire, or note-writing choice is not obvious.

## Canonical Store

Repository memory lives under `memories/repo/`.

Do not use these locations for canonical repo memory:

- `.agenta/`: task ledger and recovery context.
- `.serena/memories/`: Serena project-tool memory.
- Codex user or workspace memories: assistant-side memory, not repo-owned knowledge.
- `dev_docs/memory/` or `dev_docs/memory-module/`: product memory-module documentation.

## Golden Rule

Only keep information that is both:

- still useful across future sessions, and
- not recoverable as reliably from current code, tests, root guidance, or canonical docs.

If either condition fails, do not preserve it as repo memory.

## Institutional Schema

Use a closed note taxonomy:

- `contract`: durable current repo contract, compatibility boundary, or validated positive pattern.
- `finding`: current but incomplete or still-bounded observation.
- `historical`: dated background, incident evidence, profiling, migration context, or old decision trail.
- `pointer`: compact navigation note to a stable supporting reference.

Use a closed confidence layer for active current knowledge:

- `high`
- `partial`

Allowed active public index tags:

- `[contract][high]`
- `[finding][partial]`
- `[historical]`
- `[pointer]`

Do not invent custom labels such as `almost-current`, `background-high`, `current-medium`, or topic-specific tags.

## Triage Matrix

| Situation | Action |
| --- | --- |
| Durable current repo contract or validated pattern | Keep as `[contract][high]` |
| Useful current finding with incomplete evidence or missing regression coverage | Keep as `[finding][partial]` |
| Dated profiling, migration story, or process evidence that still adds context | Keep as `[historical]` |
| Stable supporting navigation note whose location matters | Keep as `[pointer]` |
| Same topic appears in multiple files | Merge into one canonical note |
| Entire note is derivable from current code or docs | Retire or compress into a narrower takeaway |
| Note conflicts with current code, tests, or canonical docs | Update or retire the note; do not bend source to fit memory |

## Promotion and Demotion Rules

- Promote `finding` to `contract` when uncertainty is resolved and the guidance is durable.
- Demote `contract` to `historical` when it remains useful only as dated context.
- Prefer retirement over keeping a weak note alive under a vague label.
- Keep `pointer` notes secondary. If a pointer starts carrying substantive guidance, rewrite it as `contract`, `finding`, or `historical`.

## Naming Convention

Target canonical filenames:

- `contract__<topic>.md`
- `finding__<topic>.md`
- `historical__<topic>.md`
- `pointer__<topic>.md`

Rules:

- Use lower-case kebab-case after the prefix.
- Use topic-centered names.
- Avoid vague suffixes like `misc`, `temp`, `review`, or `notes`.
- Add dates in the body when needed; put dates in filenames only when they are part of note identity.

## What Not to Save

Do not preserve these as repo memory unless they have already been distilled into a durable takeaway:

- Exact file and line anchors.
- Raw error logs or stack traces.
- Temporary debugging steps.
- Current task state, plans, or TODOs.
- Lists of recent commits or PRs.
- Large copies of `dev_docs` content.
- User-specific interaction preferences.
- Bare references to task-specific `dev_docs` files when the durable takeaway has not been distilled into repo memory.

## Date Rules

- Never store newly written relative dates such as `today`, `yesterday`, `last week`, or `next Thursday`.
- Convert to absolute `YYYY-MM-DD` when you can resolve the date from context.
- If the date cannot be resolved confidently, keep the fact out of repo memory or explicitly flag the ambiguity instead of guessing.

## Writing Template: Contract Note

- Durable takeaway.
- Why it matters, including failure mode or decision boundary.
- How to apply it or when the note is relevant.
- Optional freshness caveat if live-code verification is still recommended.

## Writing Template: Finding Note

- Current useful finding.
- What is known versus still unverified.
- Where to apply caution.
- What future validation would upgrade this note.

## Writing Template: Historical Note

- `YYYY-MM-DD` dated evidence summary.
- Explicit statement that the note is historical context, not a timeless contract.
- Current anchors that still seem relevant, if any.
- Optional pointer to deeper supporting analysis.

## Writing Template: Pointer Note

- Stable location or canonical supporting reference.
- Why this location matters.
- What durable takeaway is already captured elsewhere, if applicable.

## Index Maintenance Rules

When touching repo memory, update `memories/repo/repo_memory_current_state_index.md` last.

The index should:

- stay compact and skimmable,
- point to the canonical note for each topic,
- reflect the canonical schema accurately,
- record merges and retirements when they matter for traceability,
- avoid becoming the place where detailed reasoning lives.

Recommended section order:

1. Schema baseline.
2. Start-here working set.
3. Active contracts.
4. Active findings.
5. Stable pointers.
6. Historical background.
7. Merged or retired log.

Each active entry should fit on one line and use the canonical public schema.

Example:

```markdown
- `[contract][high]` `contract__import-cycle-lazy-exports.md`: package-level eager import traps and the preferred lazy-export pattern.
```

## dev_docs Reference Policy

Default posture: do not depend on `dev_docs/`.

- Reference active `dev_docs/` guides sparingly when they are intentionally maintained and support the note.
- Treat archived, deprecated, or execution-plan docs as background-only unless the user explicitly asks for historical context.
- If historical `dev_docs` content matters, the repo-memory note must still carry the durable conclusion by itself.
- Avoid notes whose meaning collapses if a linked `dev_docs` file becomes stale, moved, or forgotten.

## Search Hygiene

To avoid context pollution during investigations:

- Search current working docs only when current documentation is needed.
- Avoid broad searches over `dev_docs/**` by default.
- Search archived or deprecated docs only when explicitly chasing historical background, regressions, or decision provenance.

## Migration Rules

- Legacy `current-high` usually maps to `[contract][high]`, but verify that it is not a historical summary.
- Legacy `current-partial` usually maps to `[finding][partial]`, unless the uncertainty is resolved and the note should become `[contract][high]`.
- Legacy `historical-process` usually maps to `[historical]`.
- During a dedicated institutional reform pass, rename touched notes to canonical `type__topic.md` names.
- During routine maintenance, avoid gratuitous mass renames of untouched legacy notes.
- Never keep both a legacy-tagged active entry and a canonical entry for the same underlying topic.

## Merge Heuristics

Prefer a merge when:

- Two notes describe different symptoms of the same underlying contract.
- One note is a strict subset of another.
- The split forces readers to open both files every time.

Do not merge when:

- One note is current guidance and the other is historical background that would muddy the main takeaway.
- The notes are related but serve different retrieval intents.

## Good Compression Pattern

Avoid:

- long timelines,
- repeated raw observations,
- exact old file locations,
- unresolved speculation.

Prefer:

- distilled conclusions,
- clear boundaries or failure modes,
- dated evidence only when still decision-useful,
- pointers to stable supporting docs only when needed.

## Positive Guidance Rule

Preserve validated positive patterns, not only failures.

Good:

- Prefer package-level `__getattr__` plus `import_module` lazy exports in this repo when eager re-export chains trigger import cycles.

Weak:

- Do not use eager imports.

The positive form gives future sessions a recommended replacement, not just a prohibition.

## Final Sanity Check

Before finishing a curation pass, ask:

- Would a future session benefit from this note without reopening the whole investigation?
- Is this still safer than reading the current code directly?
- Did the edit preserve durable meaning while removing transient noise?
- Did the repo memory become smaller or better structured, rather than merely different?
- Does every touched active note fit exactly one canonical type and allowed tag shape?
- Did all repo-memory writes stay under `memories/repo/` unless the user explicitly expanded scope?
