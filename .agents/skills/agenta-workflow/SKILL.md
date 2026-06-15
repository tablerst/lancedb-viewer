---
name: agenta-workflow
description: "Use when managing Agenta as a local project/version/task workflow ledger: restore or repair an existing lane, keep the default active version aligned to the current implementation lane, capture reusable recovery/risk/acceptance conclusions, close versions cleanly, or submit Agenta workflow feedback through CLI or MCP. Using this skill does not imply creating Agenta versions or tasks; default to repository execution plans unless the Agenta Minimalism Gate passes."
---

# Agenta Workflow

Use Agenta as a local version-lane ledger and closeout surface, not as a loose TODO board and not as the project's long-term memory system.

Using this skill can be read-only. Calling the skill means "check whether Agenta is relevant", not "write to Agenta".

## Core Rules

1. Read repository-maintained context first: root agent instructions, README, architecture notes, active execution plans, and local skills.
2. Do not create or update Agenta projects, versions, tasks, or notes unless the Agenta Minimalism Gate passes.
3. Keep detailed implementation TODOs in repository execution plans. Agenta records recovery pointers, reusable findings, validation, risk, and closeout state.
4. Treat `project.default_version` as the primary recovery pointer for the next Agent.
5. Serialize Agenta writes and read back every changed project, version, task, note, attachment, or context manifest before reporting completion.

## Terms

- `Project`: one repository or product workspace.
- `Version`: one bounded implementation, investigation, or stabilization lane with scope, non-goals, validation, and a closeout point.
- `Default version`: the lane a future Agent should restore first. It should not point at a closed lane unless the no-active-lane exception is explicit.
- `Task`: a recovery, risk, phase, or acceptance boundary inside a version. Do not create one task per incidental TODO.
- `Index task`: the recovery entry for a version or complex lane. It links the local execution plan, summarizes scope and non-goals, and points to child tasks when they exist.
- `Local execution plan`: the repository-maintained implementation tracker. Active plans track detailed TODOs; Agenta tracks durable lane state and evidence.

## Agenta Minimalism Gate

Before creating or updating Agenta state, prove that at least one condition is true:

- Cross-session recovery: a future Agent needs a stable entry point that repository files alone do not provide.
- Independent risk or acceptance: the topic can be validated, blocked, or closed independently.
- Lane ownership: the work is a coherent capability, investigation, or stabilization lane.
- Durable conclusion: the result is reusable evidence or a decision that should be found through task search later.
- Drift repair: existing Agenta state or execution-plan linkage is misleading future recovery.

If none are true, do not write Agenta. Update the local execution plan when one exists, or report directly to the user.

## Reference Map

Read only the files needed for the current request:

- `references/operating-surfaces.md`: decide between MCP mode and CLI mode.
- `references/common-workflow.md`: shared contract, first-pass workflow, and reference selection.
- `references/version-task-planning.md`: required when creating, repairing, decomposing, or closing versions and tasks.
- `references/workflow-state-machine.md`: required for restore, execute, verify, closeout, drift repair, or `ledger_delta` work.
- `references/scenario-playbooks.md`: required for project initialization, context restore, task progress, lane closeout, or feedback scenarios.
- `references/note-and-closeout.md`: required before writing notes, changing statuses, closing tasks, or closing versions.
- `references/mcp-mode.md`: required after selecting MCP mode.
- `references/cli-mode.md`: required after selecting CLI mode.
- `references/feedback-loop.md`: required when Agenta workflow, tools, docs, or usability caused friction that should be reported.

## Output Contract

After read-only use, report the current project/version/task scope, chosen recovery entry, active-plan linkage, and warnings that affect the current lane.

After Agenta writes, include a concise `ledger_delta` with updated task ids/codes, version/default-version changes, note kinds written, verification commands, local plan sync or archive actions, remaining risks, and the next recovery entry.
