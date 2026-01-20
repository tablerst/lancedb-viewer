# LanceDB Viewer (LanceDB Studio)

基于 Tauri v2 + Vue 3 + Rust 的 LanceDB 桌面可视化工具。当前版本以 JSON 为主的 IPC v1 打通连接与 Schema 预览，同时保留后续 Arrow IPC 的扩展位。

近期目标：重构前端信息架构为“**左侧可收缩侧边栏 + 右侧正文工作区**”，支持**多连接并行**、列表虚拟化，并用 TailwindCSS（CSS Transition/Animation）/Lucide 提升交互与视觉一致性。

## 当前能力

- 连接档案（profiles）持久化：`name`、`uri`、`storageOptions`
- 连接 LanceDB：`connect_v1`
- 列出表：`list_tables_v1`
- 打开表并查看 Schema：`open_table_v1`、`get_schema_v1`
- 数据扫描：`scan_v1` 支持 JSON/Arrow（Arrow 为 base64 IPC）
- 资源浏览与数据闭环：Schema + 数据浏览（分页/列投影/过滤表达式）
- 路由分区布局：资源浏览 / 检索工作台（占位）
- 查询命令（后端可用）：`query_filter_v1`、`vector_search_v1`、`fts_search_v1`

## UI 重构（进行中）

规划中的交互形态：

- 左侧 Sidebar（可收缩）：
	- 顶部快捷操作：筛选（Local/S3/Remote 等）、新建连接（Modal）
	- 连接项卡片：状态灯、连接类型 Tag、展开表树
	- 列表虚拟化：连接列表/表列表在规模增大时保持流畅
- 右侧正文工作区：
	- 选中连接：展示连接信息与连接状态
	- 选中表：展示 Schema + 数据表格，并支持搜索/过滤/列投影/分页
	- 排序策略（MVP）：仅做前端排序（对当前已加载的数据片段排序）

动效与图标：动效使用 TailwindCSS（如 `transition-*` / `duration-*` / `ease-*`）实现折叠/展开与局部过渡；图标统一使用 `lucide-vue-next`。

## IPC v1 约定（JSON-first）

- 所有命令返回 `ResultEnvelope<T>`，包含 `apiVersion` 与 `ok` 标记
- 扫描数据通过 `DataChunk` 返回：支持 `format: "json"` 与 `format: "arrow"`
- Arrow IPC 目前仅用于 `scan_v1`，查询类命令仍为 JSON

## 连接档案结构

`ConnectProfile`（前端/后端对齐）：

- `name`: 显示名
- `uri`: `/path/to/db`、`s3://bucket/path`、`db://host:port` 等
- `storageOptions`: 作为扩展键值对传入 LanceDB（与官方文档一致）
- `options.readConsistencyIntervalSeconds`: 读取一致性间隔（可选）
- `auth`: 预留字段（后续 Stronghold 接入）

## 开发

- 安装依赖：`bun install`
- 前端开发：`bun dev`
- 桌面联调：`bun tauri dev`
- 运行测试：`bun run test`（避免与 Bun 内置 `bun test` 语义混淆）
- 代码格式化：`bun format`
- 前端代码调整完成后：及时运行 `bun lint` 或 `bun check` 查看是否有错误（需要纯检查可用 `bun ci`）

### Rust 后端测试

- 集成测试目录：`src-tauri/tests/`
- 执行：`cargo test --manifest-path src-tauri/Cargo.toml`
- 测试默认优先使用仓库根目录的 `sample-db`；若不存在，会自动生成临时库

### 示例数据库

- 生成示例库：`cargo run --manifest-path src-tauri/Cargo.toml --bin seed_db -- sample-db`
- 可选参数：`--table <name>`、`--rows <count>`（默认表名 `items`，默认 50 行）

## 安全与权限

当前 `capabilities/default.json` 开启：

- `core:default`
- `opener:default`
- `store:default`

并包含：

- `log:default`
- `dialog:default`

敏感凭证尚未落盘，后续会接入 Stronghold 并扩展 `auth` 字段。

## Roadmap

- UI Shell：可收缩 Sidebar + 右侧正文（多连接并行、虚拟列表、TailwindCSS 动效、Lucide 图标）
- Search 工作台：前端接入 `query_filter_v1` / `vector_search_v1` / `fts_search_v1`，并复用结果表格能力
- Data Explorer：补齐“前端排序/列宽/列固定/快捷筛选”等常用交互（对齐 DBeaver 类体验）
- Arrow IPC：`scan_v1` 已启用，后续扩展到查询结果

## 参考文档

- LanceDB connect/URI：https://lancedb.github.io/lancedb/js/functions/connect/
- LanceDB storage options：https://docs.lancedb.com/storage/configuration
- Tauri Store 插件：https://v2.tauri.app/plugin/store/
- Tauri Stronghold 插件：https://v2.tauri.app/plugin/stronghold/
