# Agenta Scenario Playbooks

Use these playbooks after selecting the operating surface and reading the shared workflow rules. Adapt field names to CLI or MCP, but preserve read-before-write and write-readback behavior.

## 1. Project Initialization

1. Select MCP or CLI mode.
2. List projects and reuse a matching project before creating one.
3. List versions for that project and reuse an active/default baseline when appropriate.
4. Create a baseline version only when no suitable version exists.
5. Set the project default version only when this is the active restore lane.
6. Run `context_init` or the CLI equivalent only when the workspace needs a manifest hint or migration.
7. Create or reuse an index/context task only when a work lane needs reusable recovery beyond repository files.
8. Write a conclusion note that records the project slug, active version, relevant repository files, and task-level recovery path when one exists.
9. Read back the project, version, task, and note state.

## 2. Context Restore

1. Prefer an explicit task id, task code prefix, project, or version from the user or local plan.
2. Run `workflow_check` when available for recoverability, open tasks, missing surfaces, and recommended next actions.
3. Check whether the project default version is active and whether it matches the lane named by the user or local active plan.
4. Use sorted task listing or search to find the version index or recovery task when the digest is not enough.
5. Read full task context, including notes and attachments when useful.
6. Summarize reusable conclusions, relevant files, active-plan linkage, and open risks before continuing.
7. Do not create replacement tasks when an existing reusable context task already fits.

## 3. Task Progress

1. Read the target task or version index before implementation.
2. Confirm the task still belongs to the current active/default version or explain the mismatch.
3. Do the implementation, documentation, or investigation work in repository files first.
4. Run focused verification.
5. Update the local execution plan when one exists.
6. Append a reusable finding or conclusion note only when Agenta state should change.
7. Read back changed task state.

## 4. Phase Closeout

1. Finish code, documentation, and verification first.
2. Update any local execution plan that exists.
3. Append one note per directly affected Agenta task.
4. Update task status only when the task state truly changed.
5. Read back the updated task or task context.
6. Run `workflow_check` when current-lane scope drift, missing readback, or execution-plan linkage is unclear.
7. If the work exposed Agenta workflow friction, use `feedback-loop.md`.
8. Report `ledger_delta` when Agenta state changed.

## 5. Lane Closeout

Use this when a version/lane appears complete.

1. List tasks for the version and check open, blocked, cancelled, and ready counts.
2. Read the version index task and directly affected child tasks.
3. Verify the local active execution plan TODO table, validation record, and active index entries.
4. Move completed local plans out of `active/` when repository rules require archive closeout.
5. Append a reusable conclusion note to the index task.
6. Update stale task summaries/descriptions if append-only note history could mislead future readers.
7. Mark the version `closed`.
8. Set the next active/default version when one exists.
9. Read back the project, closed version, next default version, and index task.
10. Run `workflow_check` again when available and include remaining current-lane warnings in `ledger_delta`; classify unrelated repo-hygiene warnings separately.

## 6. Workflow Feedback

1. Confirm the feedback is about Agenta workflow, tools, docs, or usability, not the current product task.
2. Prefer the configured feedback task from `feedback_task_id` or `feedback_task_code` in `project.yaml`.
3. Use `feedback_submit` in MCP mode or `agenta feedback submit` in CLI mode.
4. Keep feedback concise: surface, severity, friction, expected behavior, suggested change, and evidence.
5. Read back the created note or task context when the selected surface supports it.
