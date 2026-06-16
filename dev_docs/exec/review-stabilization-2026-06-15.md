# Review Stabilization 2026-06-15

本文件记录 Agenta version `Review Stabilization 2026-06-15` 的 repo 侧执行状态。
Agenta 只作为 recovery ledger；具体实现顺序、设计取舍、验证矩阵以本文件为准。

## Lane Rules

- 保留现有 Agenta task tree，不新增 child task，不迁移到新 version。
- `LDBREV-01`、`LDBREV-02`、`LDBREV-08` 保持 `done`。
- 剩余推进顺序：`LDBREV-04` -> `LDBREV-03` -> `LDBREV-06` ->
  `LDBREV-05` -> `LDBREV-07` -> `LDBREV-00` closeout。
- 采用完整边界关闭：子集完成只写 finding；满足 exit condition 后才标 `done`。
- 每个任务关闭前先更新本文件，再写 Agenta conclusion note、更新状态并 readback。

## Prior Closed Work

`LDBREV-01` / `LDBREV-02` / `LDBREV-08` 已关闭，保留的关键事实：

- JSON row write 支持 fixed-size float vector 的后端转换路径。
- `update_rows_v1` / `delete_rows_v1` 对空过滤和明显全表过滤保留 guardrail。
- Search 空输入本地校验、跨路由 transient error 清理、中文 locale、基础 a11y
  和窄屏侧栏策略已完成。

## Current Pass 2026-06-16

### LDBREV-04: True Hybrid Search

实现状态：repo 侧完成，待最终验证后关闭 Agenta task。

已落地：

- `combined_search_v1` 保持 IPC command 名不变，但语义改为 true hybrid。
- 后端要求 query text 和 vector 同时存在；vector-only / text-only 直接返回
  `InvalidArgument`，不再隐式拼接或降级。
- 使用 LanceDB 0.23.1 hybrid query：
  - `nearest_to(vector)`
  - `full_text_search(query)`
  - `RRFReranker`
  - `NormalizeMethod::Rank`
- 移除旧的 JSON row 字符串去重、vector/FTS 两路结果拼接逻辑。
- 响应 schema/rows 增加 hybrid metadata：
  - LanceDB emitted: `_relevance_score`，以及在具体路径可用时的 `_distance` /
    `_score`
  - app annotation: `_hybrid_rank`、`_hybrid_source`
- Search UI 文案从“组合查询”改为“混合检索”。
- Search UI 显示融合分数、向量距离、全文分数、融合排序、来源；缺失列显示为空。
- Search paging 使用上一页/下一页按钮，内部保留 offset 语义。

关键文件：

- `src-tauri/src/services/v1.rs`
- `src-tauri/tests/commands_v1.rs`
- `src/views/SearchView.vue`
- `src/views/search/searchRequests.ts`
- `src/views/search/searchRequests.test.ts`

### LDBREV-03: Arrow-First Data Browsing

实现状态：repo 侧完成，待最终验证后关闭 Agenta task。

已落地：

- 新增前端依赖 `apache-arrow`。
- 新增 `src/lib/arrowDecoder.ts`，把 Arrow IPC base64 解码为
  `DataGrid` 可消费的 rows/schema。
- `DataTab` 默认调用 `scan_v1({ format: "arrow" })`。
- Arrow 成功时显示 `传输：Arrow IPC`。
- Arrow 解码或兼容失败时回退到 `scan_v1({ format: "json" })`，并显示
  `Arrow 解码失败，已回退 JSON`。
- 保留 `limit` / `offset` / `nextOffset`，本轮不引入 cursor。

关键文件：

- `package.json`
- `bun.lock`
- `src/lib/arrowDecoder.ts`
- `src/lib/arrowDecoder.test.ts`
- `src/views/explorer/DataTab.vue`

### LDBREV-06: LanceDB Management Surface

实现状态：repo 侧完成，待最终验证后关闭 Agenta task。

已落地：

- `IndexDefinitionV1` 增加 index stats 字段：
  `numIndexedRows`、`numUnindexedRows`、`distanceType`、`numIndices`、`loss`。
- `CreateIndexRequestV1` 增加常用 LanceDB 参数：
  `distanceType`、`numPartitions`、`sampleRate`、`maxIterations`、
  `targetPartitionSize`、`numSubVectors`、`numBits`、`numEdges`、
  `efConstruction`。
- 后端 `list_indexes_v1` 调用 LanceDB `index_stats` 并把可用统计返回给 UI。
- 后端 `create_index_v1` 将常用参数映射到 IVF / PQ / RQ / HNSW index builder。
- Indexes UI 显示索引状态、距离类型、分片/索引数、loss，并提供关键创建参数。
- FTS 创建本轮只暴露列、名称、replace；tokenizer 等更深参数不在本轮开放。
- 早前 Versions metadata 已改为结构化 key/value 展示。

关键文件：

- `src-tauri/src/ipc/v1.rs`
- `src-tauri/src/services/v1.rs`
- `src/ipc/v1.ts`
- `src/lib/tauriClient.ts`
- `src/views/explorer/IndexesTab.vue`

### LDBREV-05: Test Matrix

实现状态：repo 侧完成，待最终验证后关闭 Agenta task。

新增/扩展覆盖：

- `combined_search_v1` true hybrid 输入要求、返回 metadata、rank/source annotation。
- Arrow IPC decode utility，包含 real Arrow IPC stream、wide-ish values、vector-like typed array。
- IPC wrapper payload for index creation tuning fields。
- `secret_ref` unsupported frontend wrapper behavior。
- 保留 mutation guard tests。

关键文件：

- `src-tauri/tests/commands_v1.rs`
- `src/lib/arrowDecoder.test.ts`
- `src/lib/tauriClient.test.ts`
- `src/views/search/searchRequests.test.ts`

### LDBREV-07: UX / Auth / Docs Closeout

实现状态：repo 侧完成，待最终验证后关闭 Agenta task。

已落地：

- Frontend `connectV1` 对 `secret_ref` 返回 friendly `not_implemented` envelope，
  不再调用后端。
- Backend 仍保留 `secret_ref` 的 `NotImplemented` 防线。
- README 更新 search、Arrow、auth、validation 当前状态。
- UI_DESIGN 更新 Search、Arrow-first data browsing、management surface、auth UX
  和 validation checklist。
- 本执行文档替换过期描述，记录当前 lane 关闭规则和验收状态。
- Search pagination 从裸 `nextOffset` 改为上一页/下一页控件。

关键文件：

- `src/lib/tauriClient.ts`
- `src/lib/tauriClient.test.ts`
- `src/views/SearchView.vue`
- `README.md`
- `UI_DESIGN.md`
- `dev_docs/exec/review-stabilization-2026-06-15.md`

## Validation Matrix

2026-06-16 最终验证：

- `cargo test --manifest-path src-tauri/Cargo.toml`：通过。
  - Rust unit: 1 passed。
  - `tests/commands_v1.rs`: 9 passed。
- `bun run test`：通过，7 files / 26 tests。
- `bun run test:coverage`：通过，overall statements 90%，branches 84.78%，
  functions 100%，lines 90%。
- `bun run build`：通过。
  - 仅保留 Vite 常规提示：Browserslist data 过旧、主 bundle 超过 500 kB。
- Targeted touched-surface Biome：通过。
  - `bunx biome ci package.json src/components.d.ts src/ipc/v1.ts
    src/lib/tauriClient.ts src/lib/tauriClient.test.ts src/lib/arrowDecoder.ts
    src/lib/arrowDecoder.test.ts src/views/SearchView.vue
    src/views/search/searchRequests.ts src/views/search/searchRequests.test.ts
    src/views/explorer/DataTab.vue src/views/explorer/IndexesTab.vue
    README.md UI_DESIGN.md
    dev_docs/exec/review-stabilization-2026-06-15.md`
- `git diff --check`：通过；仅输出 Windows checkout 的 LF -> CRLF 提示。

全量 `bun run ci` 仍不是本 lane 的关闭要求；此前已记录 repo-wide formatting drift
超出本轮范围。本轮只要求 touched files 的 Biome CI 通过。

## Closeout Notes

全部子任务关闭后，`LDBREV-00` conclusion 需要记录：

- 最终验证矩阵。
- 剩余风险。
- 是否存在迁移到新 version 的工作。
- 若没有新的 active lane，不迁移本 version；仅在后续明确开启新 lane 时再调整
  project default version。
