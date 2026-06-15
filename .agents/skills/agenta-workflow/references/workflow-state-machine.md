# Agenta Workflow State Machine

Use this file for substantial Agenta work after selecting MCP or CLI mode.

## 1. Stages

Use these stages as the default loop:

1. `bootstrap`: select MCP or CLI, read repository context, run `workflow_check` when available, and identify project/version/task scope.
2. `restore`: read the chosen version index, recovery task, or task lane. Minimum output is scope, recovery entry, active-plan linkage, warnings, and missing surfaces.
3. `execute`: perform the requested code, documentation, or investigation work. Keep adjacent Agenta tasks together when one batch advances them.
4. `verify`: run relevant checks and update the local active execution plan.
5. `closeout`: write Agenta notes/statuses, close or transition the version when appropriate, read back writes, and report `ledger_delta`.

Run the full loop only after the Agenta Minimalism Gate passes or when the user explicitly asks for Agenta ledger work.

## 2. Bootstrap And Restore

Before restoring Agenta task context, read project-maintained files that govern the work:

- Root agent instructions.
- README and architecture notes.
- Relevant active execution plans.
- Local skills.

Then use Agenta to recover task history, decisions, evidence, and status.

When available, run `workflow_check` as the first read-only pass. Inspect `digest`, `missing_surfaces`, `warnings`, and `recommended_next_actions` before expanding to task context or search.

Classify warnings before acting:

- Current-lane blocker: stale default version, missing current recovery entry, or current active plan not linked. Resolve or report before implementation.
- Repo hygiene: feedback route, old completed active plans, unrelated unlinked plans, or parallel lanes. Record separately; do not expand the current task unless the user asked for ledger cleanup.
- Historical stale state: closed lanes or archived work with outdated notes. Ignore unless directly relevant to recovery.

## 3. Execute And Verify

During implementation or investigation:

1. Keep repository changes in the appropriate source files and execution plan.
2. Treat Agenta tasks as recovery and evidence boundaries, not as the running checklist.
3. When one implementation batch advances multiple adjacent tasks, record which tasks were directly advanced and which only received enabling work.
4. Run the verification commands appropriate to the changed surfaces.
5. Record deferred validation explicitly, including why it was deferred and what should run later.

## 4. Closeout

Before writing closeout state:

1. Finish code, documentation, and verification first.
2. Update the local execution plan when one exists.
3. Decide which Agenta tasks were directly affected.
4. Append one note per directly affected task.
5. Update task or version status only when the true state changed.
6. Read back every changed task, note, version, project, attachment, or context manifest.

## 5. Ledger Delta

`ledger_delta` is required only when Agenta state changes.

Include:

- Updated task ids/codes.
- Version status or default-version changes.
- Note kinds written.
- Verification commands and outcomes.
- Local plan sync or archive actions.
- Remaining risks.
- Next recovery entry.

Do not report a `ledger_delta` for read-only Agenta checks.

## 6. Drift Repair

If the default version is closed, an active version has no open tasks, local active plans contain completed lanes, or `workflow_check` reports current-lane linkage drift:

1. Stop treating the stale pointer as current state.
2. Read back the project, relevant versions, version tasks, and local active plan index.
3. Decide whether to close stale active versions, archive completed active plans, or set a different active/default version.
4. Apply writes serially.
5. Read back each changed project, version, task, or note.
6. Report the drift and repair in `ledger_delta`.
