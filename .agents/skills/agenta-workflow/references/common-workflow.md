# Agenta Common Workflow

Use this file after selecting MCP or CLI mode. It gives the shared contract and points to the detailed reference that should be loaded next.

## 1. Contract

Agenta is a local workflow ledger with these responsibilities:

- Keep one project per repository or product workspace.
- Represent each bounded implementation, investigation, or stabilization lane as a version.
- Keep the project default version aligned to the lane a future Agent should restore first.
- Store task-level recovery, evidence, validation, risk, and closeout notes.
- Stay synchronized with repository execution plans.

Agenta is not:

- A replacement for Git history.
- A replacement for canonical repository documentation.
- A long-term memory store for personal preferences or chat residue.
- A loose TODO board where every small action becomes a task.
- A mirror of the repository execution plan's TODO table.

## 2. First Pass

Before doing Agenta ledger work:

1. Read repository-maintained context first.
2. Decide whether the Agenta Minimalism Gate passes.
3. Select MCP or CLI mode from `operating-surfaces.md`.
4. Run `workflow_check` when available for Agenta-relevant work.
5. Choose the narrowest next reference from the map below.

If the gate does not pass, do not write Agenta. Update the local execution plan when one exists, or report directly to the user.

## 3. Reference Selection

- Creating, repairing, decomposing, or closing versions/tasks: read `version-task-planning.md`.
- Restoring context, executing a lane, verifying work, repairing drift, or reporting `ledger_delta`: read `workflow-state-machine.md`.
- Initializing projects, restoring contexts, updating task progress, closing phases, closing lanes, or submitting feedback: read `scenario-playbooks.md`.
- Writing notes, changing task status, or closing tasks/versions: read `note-and-closeout.md`.
- Using MCP tools: read `mcp-mode.md`.
- Using CLI commands: read `cli-mode.md`.
- Reporting Agenta workflow friction: read `feedback-loop.md`.

## 4. Synchronization Rule

Treat source files, local execution plans, and Agenta ledger state as one workflow surface:

1. Finish repository work first.
2. Run relevant verification.
3. Update the local execution plan when one exists.
4. Write Agenta notes/status only when Agenta state should change.
5. Read back every Agenta write.

Do not defer synchronization long enough for future recovery to become misleading.
