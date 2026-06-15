# Agenta Version And Task Planning

Use this file when creating, repairing, decomposing, validating, or closing Agenta versions and tasks.

## 1. Purpose

Agenta versions and tasks should model real project recovery boundaries. They must not mirror every local TODO row, touched file, or command.

Repository files remain the source of truth for detailed implementation tracking. Agenta stores the durable lane pointer, task recovery boundaries, evidence, validation, risk, and closeout state.

## 2. Version Standard

A `Version` represents one bounded implementation, investigation, or stabilization lane.

Create or keep a version only when the lane has:

1. A clear scope and non-goals.
2. A closeout point that can be recognized later.
3. Expected validation or evidence.
4. A reason future Agents may need to restore it.
5. A local execution plan when detailed TODO tracking is needed.

Do not create a version for:

- A date bucket, work session, or conversation.
- A single small bug fix that fits an existing lane.
- A documentation-only update for an already-open lane.
- A checklist split whose only purpose is to mirror a local execution plan.
- A personal preference, reminder, or chat residue.

## 3. Version Required Shape

Every non-trivial version should have:

1. A readable lane name and description with scope, non-goals, expected validation, and residual-risk expectations.
2. `status=active` while implementation, validation, or integration work remains open.
3. `project.default_version` pointing to it when this is the lane the next Agent should restore.
4. One index task with `task_kind=index`, a stable `task_code`, and a recovery note.
5. A machine-friendly note linking the active local execution plan when one exists.
6. Child tasks only for independently recoverable phase, risk, acceptance, or blocker boundaries.

A version may have only an index task. Do not create child tasks just to make the lane look detailed.

## 4. Task Types

Use child tasks only when the existing index task is not enough.

- `Index task`: required for a non-trivial version. Use code suffix `-00` when the lane uses numbered tasks. It owns recovery, scope, non-goals, plan linkage, summary status, and final conclusion.
- `Phase task`: a coherent implementation or integration phase that can be resumed or closed independently.
- `Risk task`: a hypothesis, compatibility concern, migration risk, performance risk, or operational risk that needs separate evidence.
- `Acceptance task`: an independently reviewed validation gate, soak gate, operator-signoff gate, or closeout gate.
- `Context task`: a reusable module or architecture map when future recovery benefits from a stable context entry.
- `Blocker task`: a dependency that blocks another task and should be resolved explicitly.

Do not create tasks for:

- One file, folder, or component touched by the same change.
- One command to run.
- One TODO row from the local execution plan.
- A status update that fits as a note on an existing task.
- A named future step with no independent recovery or validation value.

## 5. Task Required Shape

Each planned task should make these fields explicit in the title, summary, description, or first note:

1. Purpose: what recovery or risk boundary this task owns.
2. Scope: included surfaces and important non-goals.
3. Exit condition: what must be true before the task can be `done`.
4. Validation evidence: automated tests, build commands, instrumentation/log evidence, manual acceptance steps, readback, or a clear reason validation is not applicable.
5. Local plan link: active execution-plan file and section when detailed TODOs live there.
6. Expected artifacts: files, notes, attachments, logs, screenshots, or command output that prove progress.
7. Dependencies or blockers when the task cannot progress alone.

For investigation tasks, "validation evidence" means the method used to prove or falsify the conclusion, not necessarily a unit test.

## 6. Validation Expectations

Every implementation, risk, or acceptance task should name at least one validation path:

- Automated test command.
- Build or typecheck command.
- Runtime smoke command.
- Structured log, trace, or instrumentation evidence.
- Manual acceptance step with observable pass/fail criteria.
- Agenta readback evidence for ledger-only changes.

Leave validation empty only when the task is purely administrative and the reason is explicit.

When validation is deferred, record:

1. Why it could not run now.
2. Which command, probe, or manual check should run later.
3. Whether the task can still be marked `done` without that evidence.

## 7. Task Tree Size

Prefer the smallest useful tree:

- Simple lane: one index task.
- Normal lane: index task plus 2-6 child tasks.
- Large lane: child tasks grouped by phase, risk, or acceptance, not by every submodule.

If the proposed tree has more than 7 child tasks, first move detailed TODOs into the local execution plan and keep Agenta at phase/risk/acceptance granularity.

## 8. Numbering And Fields

Use stable `task_code` values when a task set will be restored later:

- Index: `LANE-00`
- Child tasks: `LANE-01`, `LANE-02`, ...
- Closeout or acceptance: use a normal number only when it has independent acceptance work; otherwise close through the index task.

Set first-class fields deliberately:

- `task_kind=index` for index and recovery tasks.
- `task_kind=context` for reusable context maps.
- `task_kind=standard` for normal implementation, risk, and acceptance tasks.
- `note_kind=scratch`, `finding`, or `conclusion` for notes.

## 9. Positive And Negative Examples

Good task boundaries:

- `ACUX1-02`: rate-limit, degradation, and retry behavior, because it has a separate runtime/UX risk and validation path.
- `L2DPerf-03`: real runtime soak acceptance, because synthetic tests do not close the live Cubism/WebGL risk.
- `DBG-MCP-00`: startup recovery index, because it is the restore entry for a specific recovery lane.

Weak task boundaries:

- `Update useRendererHost.ts`.
- `Run UI build`.
- `Fix docs`.
- `Implement TODO 1`.
- `Check common files`.

Convert weak boundaries into local execution-plan TODOs unless they become independent recovery, risk, or acceptance surfaces.
