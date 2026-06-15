# Agenta Notes And Closeout

Use this file before writing notes, changing task status, closing tasks, or closing versions.

## 1. Note Style

Write reusable context rather than a chat transcript.

Set `note_kind` explicitly:

- `scratch`: temporary draft or process note.
- `finding`: verified finding, usually the default.
- `conclusion`: reusable conclusion.

Recommended structure:

1. Topic and date.
2. Verified key conclusions.
3. Recommended reading order.
4. Key files.
5. Main risks, contracts, or cautions.
6. Recommended verification path when useful.

Writing rules:

- Lead with conclusions, not only file names.
- File paths should help future readers locate the relevant code directly.
- Explain why a risk is risky.
- Use `note_kind=conclusion` when the note is reusable as a conclusion.
- If an older note is stale or wrong, append a superseding note and update task `summary` or `description` so search summaries steer readers to the current truth.

## 2. When To Append A Note

Append a note only when at least one condition is true:

- The work adds reusable context to an existing task.
- The work records verified findings, validation, residual risk, or closeout state.
- The work supersedes stale task history.
- The work records why validation or closure is deferred.

Do not append a note just to mirror an incremental progress update already visible in the local execution plan.

## 3. When To Create A New Task

Create a new task only when the existing index task is not sufficient and at least one condition is true:

- The topic has an independent risk boundary.
- The topic has an independent acceptance or validation gate.
- The work is a phase boundary that could be implemented or closed independently.
- The conclusion is enough to become the entry point for the next work session.
- The content does not fit cleanly as an addendum to another task.

Do not create a task only because a module will be touched, a TODO row exists, a status update needs recording, or a future implementation step can be named.

## 4. When To Mark Done

Mark a task as `done` only when these conditions are mostly true:

- Notes contain enough context for future recovery.
- Project, version, and task ownership are correct.
- The current goal is closed.
- Required validation evidence is recorded or the deferral is explicit.
- Task status and notes were read back after writing.

If the task was just created and does not yet contain useful context, keep it `ready` or `in_progress`.

## 5. When To Close A Version

Close a version only when these conditions are mostly true:

- The version has no open in-scope implementation tasks.
- The index task contains enough context for future recovery.
- Validation and residual risks are recorded.
- Local active plans and indexes have been synchronized or archived.
- The project default version has been moved to the next active lane or the no-active-lane state is explicit.
- Version and project state were read back after writing.

## 6. Final Checks

- The project exists and the slug is correct.
- The default version points at the correct current lane.
- Closed versions are not left as the default unless explicitly documented.
- Stale active versions are closed or explained.
- New tasks are attached to the correct version.
- Each non-trivial active version has an index task.
- Numbered tasks have `task_code`.
- Context and index tasks have the correct `task_kind`.
- Notes use `note_kind` to mark scratch, finding, or conclusion.
- Status matches the true completion state.
- Local active execution plans match Agenta version/task state.
- Writes were confirmed by reading back the resulting state.
- Closeout includes `ledger_delta` when Agenta state changed.

## 7. Avoid

- Creating task titles without useful notes.
- Creating many tasks without clear ordering or numbering.
- Creating one Agenta task per small TODO.
- Mirroring every execution-plan row as an Agenta child task.
- Leaving completed plans in `active/`.
- Leaving a closed version as the default recovery lane by accident.
- Leaving completed versions marked `active`.
- Creating a new version when a conclusion note on an existing version is enough.
- Assuming writes succeeded without reading back.
- Putting everything into one giant task.
- Running multiple write operations in parallel and causing storage lock conflicts.
- Marking exploratory tasks as `done` too early.
