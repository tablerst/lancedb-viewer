# DESIGN.md

本文件定义 **LanceDB Studio** 前端的标准 UI 规范与信息架构，用于：

- 指导后续页面/组件的实现与重构
- 保证交互一致性（侧边栏、连接卡片、表格操作）
- 为动效（TailwindCSS / CSS Transition）与图标（Lucide）提供统一约束

> 技术栈：Vue 3 + TypeScript + Naive UI + Tailwind + Lucide

---

## 1. 设计目标与原则

### 1.1 产品目标

- 以 **连接（Connections）→ 表（Tables）→ 数据/检索（Data/Search）** 为主线
- 支持 **多连接并行**：多个连接同时保持在线，互不影响
- 大列表与大表数据要“不卡”：连接列表/表列表 **虚拟化**，数据表格先做 **分页/片段渲染**

### 1.2 交互原则

- **稳定画布**：右侧正文尽量不因侧边栏交互产生大幅 reflow
- **就地反馈**：成功用 toast（`NMessage`），错误用持久 `NAlert`（右侧正文顶部），详见 §6.1
- **低认知负担**：相同动作在不同页面/组件保持同样入口和同样文案

### 1.3 MVP 约束

- 排序：仅做 **前端排序**（对“当前已加载数据片段”排序），不要求后端 scan 支持 order by
- 检索页：后端命令已存在，UI 后续按工作台方式接入（见 Roadmap）

---

## 2. 信息架构（IA）

整体布局采用“**左侧可收缩侧边栏 + 右侧正文工作区**”。

- 左侧 Sidebar：以连接管理为主（含表树）；全局导航后续补齐（Explorer/Search）
- 右侧正文：按当前路由展示 Explorer / Search 等工作台

### 2.1 面包屑导航

右侧正文顶部采用**面包屑**取代外层"表 Tab"，仅保留单层功能 Tab：

- 面包屑格式：`连接名 > 表名`
- 点击连接名跳回连接概况；点击表名可下拉切换同连接下其他表
- 内层 Tab（Schema / 数据浏览 / …）作为唯一的 Tab 层级，消除双层 Tab 嵌套

### 2.2 Explorer 子路由约定

Explorer 常驻 Tab 映射为子路由，支持 URL 直达与浏览器后退：

```
/connections/:id/table/:name/schema
/connections/:id/table/:name/data       # 浏览 + 写入合并
/connections/:id/table/:name/indexes
/connections/:id/table/:name/versions
```

> 低频功能（导入导出、维护）不再映射为独立路由，改为通过 Modal 访问。

### 2.3 当前落地文件

- Layout Shell：`src/App.vue`
- Sidebar：`src/components/sidebar/Sidebar.vue`
- 连接项卡片：`src/components/sidebar/ConnectionItem.vue`
- Explorer 容器：`src/views/explorer/ExplorerView.vue`
- Explorer 各 Tab：`src/views/explorer/SchemaTab.vue` / `DataBrowseTab.vue` / …

---

## 3. 视觉规范（基线）

### 3.1 主题与颜色

Naive UI 主题覆盖在 `src/theme/naiveTheme.ts`。

- 主色（Primary）：`#38bdf8`（sky-400）
- 成功（Success）：`#14b8a6`
- 警告（Warning）：`#f59e0b`
- 错误（Error）：`#ef4444`
- 背景：`#f8fafc`
- 文本：`#0f172a/#334155/#64748b`

### 3.2 圆角与卡片

- 全局圆角：`10px`
- 卡片：白底 + 轻边框（`#e2e8f0`）+ 圆角 `12px`
- 阴影：默认尽量克制（可无阴影）；仅在 **hover/selected** 等需要强调层级时使用轻量阴影
  - 选中态允许加入“主题色微光”（glow shadow）以表达悬浮
  - 避免在同一元素上叠加 `ring-*` 与 `shadow-*`（`ring` 基于 box-shadow，易在圆角与抗锯齿处产生混色/脏边）

### 3.3 字体与排版

- 默认字体：`"Inter", "Noto Sans SC", system-ui, sans-serif`
- 标题层级：
  - 页面标题：`text-lg font-semibold`
  - 卡片标题：Naive `NCard` 默认样式
  - 次级说明：`text-xs text-slate-500`

---

## 4. Sidebar 规范

### 4.1 宽度与折叠

- 展开宽度：`320px`
- 折叠宽度：`72px`
- 折叠时：
  - 隐藏文字，仅保留图标与关键状态
  - 默认收起表树
  - 连接卡片使用**首字母圆形图标 + 状态灯**作为主视觉
  - Hover 显示 Tooltip 信息卡（名称 + URI + 状态 + 表数量）
  - 选中态增加左侧竖线指示器（与展开态一致）
  - 去掉连接类型 Tag（改在 Tooltip 中展示）
  - 卡片最小高度压缩到 `64px`（原 `84px`），提升可见数量

对应实现：`Sidebar.vue` 中 `sidebarWidth`。

### 4.2 结构与区域

Sidebar 由上到下：

1. **品牌区**：应用标识 + 版本 Tag + 折叠按钮
2. **连接列表区**（滚动容器）：连接项卡片（可展开表树）

> 后续计划：在 Sidebar 顶部加入“筛选/新建连接”快捷入口，并增加全局导航按钮。

### 4.3 虚拟列表（必须）

- 组件：`NVirtualList`
- 启用条件：列表长度较大时开启虚拟化（当前实现：折叠态且连接数 > 12）
- 连接项 itemSize：`92px`（需与 `ConnectionItem` 实际高度保持一致）

原则：

- 先保证滚动流畅，再考虑复杂渲染
- 虚拟列表内每项避免昂贵计算（深层 watch / 大量 computed）

---

## 5. 连接项卡片（ConnectionItem）规范

### 5.1 展示信息

- 状态灯（左上/标题前）：
  - 绿色：已连接
  - 灰色：未连接
- 名称 + URI（折叠态隐藏）
- 连接类型 Tag：基于 URI 推断（`Local/S3/GCS/Azure/Remote/Unknown`）
  - 解析逻辑：`src/lib/connectionKind.ts`

### 5.2 动作与交互

- 点击卡片主体：选中该连接（成为右侧正文的“活跃连接”）
- 连接（Connect）：对该连接发起连接（不影响其它连接）
- 刷新（Refresh）：刷新该连接的表列表
- 表（Tables）：已连接时可展开/收起表列表
#### 5.2.1 内联操作按钮

卡片展开态必须在底部（表列表上方）显示**内联操作按钮行**：

- 未连接时：`[连接]`（primary 按钮）
- 已连接时：`[刷新]`（secondary）+ `[断开]`（quaternary/ghost）
- 连接中/断开中：按钮 `loading` 状态

右键上下文菜单作为**补充入口**保留（包含编辑/凭证/删除等低频操作），但 **不得是连接/断开/刷新的唯一入口**。

删除卡片上"右键该连接以查看更多操作"的提示文字，将空间还给内联操作按钮。
一致性要求：

- 任何“连接/刷新/打开表”的动作都必须 **先选中该连接**，确保右侧正文同步。

### 5.3 视觉状态（默认 / Hover / Selected / Focus）

- 默认（未选中、未 hover）：卡片保持干净，仅展示边框与内容，避免常驻“边缘发色”
- Hover：提供轻量阴影（例如 `shadow-sm`）与轻微边框加深，提示可交互
- Selected（选中）：必须具备清晰的“抬起/悬浮”感
  - 使用更明显的 elevation 阴影 + 主题色微光（glow shadow）
  - 配合主题色边框（例如 `border-sky-200`）作为选中锚点
  - 不使用常驻外圈 `ring-*` 来表达选中（避免与阴影叠加导致的混色/杂边）
- Hover on Selected：选中卡片在 hover 时也应提供额外反馈
  - 建议使用轻微上浮位移（例如 `hover:-translate-y-0.5`）+ 更强阴影
  - 过渡需包含 `transform` 与 `box-shadow`（并考虑 `prefers-reduced-motion`）
- Focus（键盘可访问性）：卡片内部的主要按钮应使用 `focus-visible:outline-*` 统一焦点样式
  - 禁用浏览器默认 outline（避免出现“奇怪蓝边”伪装成卡片边缘）

### 5.4 表列表（树形/展开）与虚拟化

- 当前为“表列表”最小形态（后续可扩展为树形）
- 展开高度：最多 `200px`，内部使用 `NVirtualList`（itemSize `32px`）

---

## 6. 右侧正文（Content）规范

### 6.1 反馈机制（统一规范）

**操作反馈（成功/信息）**：使用 `NMessage`（toast），持续时间 3-5 秒，适用于"连接成功""写入完成"等短暂反馈。

**错误反馈**：在右侧正文顶部（`<main>` 内部、Tab 上方）渲染 **持久 NAlert**（`type=error`），用户需手动关闭或下次操作时自动清除。禁止仅用会消失的 toast 展示错误。

**全局错误 Banner**：App.vue 中 `<main>` 内部增加一个固定区域，当 `errorMessage` 存在时渲染红色 NAlert。

原则：

- 成功→ toast，错误→ persistent Alert，进度→ 内联 loading
- 文案要可行动：尽量提示"选择连接/点击连接按钮"等下一步

### 6.2 正文区 Sticky Header（固定头部）

右侧正文采用 **Sticky Header** 布局，确保面包屑导航和 Tab 栏始终可见：

```
<main class="flex flex-col overflow-hidden">
  ├── Error Alert（固定区域，有错误时显示）
  ├── Sticky Header（sticky top-0 z-20 bg-white border-b）
  │     ├── 面包屑（连接名 > 表名）
  │     └── NTabs（仅 Tab 栏，不含内容）
  └── Tab Content（flex-1 overflow-y-auto）
</main>
```

- 面包屑和 Tab 栏**不随内容滚动**
- Tab 内容区独立滚动，占满剩余空间
- 数据表格**不设 `max-h`**，依靠父容器约束自然占满
- 错误 Alert 位于 Sticky Header 上方，不影响内容区布局

### 6.3 内容区宽度约束

- **数据浏览 / 检索结果**等表格密集页面：**不设 max-width**，让表格占满可用宽度
- **Schema、凭证、设置**等信息密度低的页面：保留 `max-w-[1600px]` 容器
- 通过各 View 自身的 class 或 route meta 控制，而非全局统一限制

---

## 7. Explorer（表工作台）规范

### 7.0 组件拆分与 Tab 精简

Explorer Tab 栏从 7 个精简为 **4 个高频 Tab**，低频功能降级为 Modal / 右键菜单入口：

#### 常驻 Tab（4 个）
| Tab | 路由片段 | 功能 |
|-----|----------|------|
| Schema | `schema` | 表结构 + 列操作 |
| 数据 | `data` | 浏览 + 写入/更新/删除（合并） |
| 索引 | `indexes` | 索引列表 + 创建 + 删除 |
| 版本 | `versions` | 版本列表 + 时间旅行 + 克隆 |

#### 降级功能（Modal / 右键菜单）
| 功能 | 入口 |
|------|------|
| 导入数据 | 表右键菜单"导入数据…" → Modal |
| 导出数据 | 表右键菜单"导出数据…" / 数据 Tab 工具栏按钮 → Modal |
| 维护（Compact/Vacuum） | 表右键菜单"维护…" → Modal |

#### 目录结构

```
src/views/explorer/
├── ExplorerView.vue           # Sticky Header + Tab 容器 + 面包屑
├── SchemaTab.vue              # Schema 展示 + 列操作入口
├── DataTab.vue                # 数据浏览 + 底部写入/更新/删除操作面板
├── IndexesTab.vue             # 索引列表 + 创建 + 删除
├── VersionsTab.vue            # 版本列表 + 时间旅行 + 克隆
├── ImportDialog.vue           # 导入 Modal
├── ExportDialog.vue           # 导出 Modal
├── MaintenanceDialog.vue      # 维护 Modal（Compact + Vacuum）
├── CreateTableDialog.vue      # 创建表 Modal（从 ExplorerView 拆出）
├── RenameTableDialog.vue      # 重命名表 Modal（从 ExplorerView 拆出）
├── DataResultTable.vue        # 公共数据表格组件（Explorer + Search 复用）
├── explorerShared.ts          # 共享常量 / 工具函数
└── composables/
    └── useExplorerTable.ts    # 共享 tableId / schema / reset 逻辑
```

每个 Tab 通过 `inject/provide` 或 `props` 获取 `tableId` / `connectionId` / `schema` 等必要上下文。
Tab 切换/表切换时，由组件生命周期自然重置局部状态，无需在父组件手动清零 50+ 个 ref。

**NTabs 渲染策略**：使用 `display-directive="if"` 按需渲染 Tab 内容，避免所有 Tab 同时挂载导致的不必要 IPC 调用和内存占用。

### 7.1 Schema Tab

- 表结构表格：`NDataTable`
- 字段信息：name / dataType / nullable
- 底部操作区：新增列 / 修改列 / 删除列 按钮
- **"删除表"操作**放此 Tab 底部或面包屑右侧 `···` 菜单中，不放在数据浏览 Tab

### 7.2 数据 Tab（浏览 + 编辑一体化）

采用 **DBeaver 模式**：数据浏览和数据编辑合并为单个"数据"Tab，使用**自定义 DataGrid 组件**（非 NDataTable）。

> 完整设计文档：`dev-docs/DATA_TABLE_REDESIGN.md`

#### 整体布局（自上而下）

```
┌─────────────────────────────────────────────────────────────────┐
│ 🔧 工具栏  [刷新] [过滤器▾] [高级筛选▾]     [+行] [保存] [导出] │
├─────────────────────────────────────────────────────────────────┤
│ ▼ 高级筛选面板（可折叠，默认收起）                                │
│   过滤表达式 + 列投影选择 + 查询按钮                              │
├─────────────────────────────────────────────────────────────────┤
│  列名A ▲ │ 列名B   │ 列名C   │ 列名D          │  ← 表头（可排序）│
│  [___]   │ [___]   │ [___]   │ [___]          │  ← 过滤器行      │
├──────────┼─────────┼─────────┼────────────────┤                  │
│  数据行 1 ...                                  │                  │
│  数据行 2 ...                                  │                  │
├─────────────────────────────────────────────────────────────────┤
│ ◀ 1 ▶ 50/page │ 已加载 50 行 (0.23s) │ 2 行待保存 │  ← 状态栏    │
├─────────────────────────────────────────────────────────────────┤
│ ▶ 写入数据 / ▶ 更新数据 / ▶ 删除数据（折叠面板，批量操作入口）    │
└─────────────────────────────────────────────────────────────────┘
```

#### 核心功能

1. **表头过滤器行（Header Filter Row）**
   - 表头下方固定一行输入框，每列一个
   - 输入过滤条件后按 Enter 触发**后端过滤**（构建 filter expression 发送 `scanV1`）
   - 支持操作符（`> 5`、`!= "abc"`、`NULL`）、范围（`1..100`）、文本匹配
   - 多列过滤器用 AND 组合
   - 可通过工具栏按钮切换显示/隐藏

2. **单元格内联编辑**
   - 双击（或 `F2`）进入编辑模式
   - 编辑后标记为 dirty（黄色高亮），支持 `Escape` 撤销
   - 工具栏"保存"按钮批量提交（`updateRowsV1` + `writeRowsV1` + `deleteRowsV1`）
   - 行定位策略：使用所有标量列值组合构建 filter 定位行

3. **数据刷新**
   - 工具栏"刷新"按钮 + `F5` 快捷键
   - 保留当前 offset/limit/filter 状态

4. **高级筛选面板（Advanced Filter Panel）**
   - 表格上方可折叠面板（默认收起）
   - 固定在表格内容区顶部（sticky），不遮盖数据
   - 包含：原始过滤表达式输入、列投影选择
   - 收起时如有活跃过滤条件，在工具栏用 badge 显示摘要

#### 工具栏按钮

| 左侧 | 右侧 |
|-------|-------|
| 刷新（RefreshCcw） | 添加行（Plus） |
| 过滤器行 toggle（Filter） | 删除行（Trash2） |
| 高级筛选 toggle（SlidersHorizontal） | 保存（Save） |
| | 导出（Download） |

#### 底部操作面板（保留）

写入/更新/删除的 NCollapse 面板保留作为**批量操作入口**（JSON 写入、表达式更新、条件删除），位于状态栏下方。

#### 自定义组件目录

```
src/components/datagrid/
├── DataGrid.vue              # 核心表格组件
├── DataGridToolbar.vue       # 工具栏
├── DataGridHeader.vue        # 表头 + 排序
├── DataGridFilterRow.vue     # 过滤器行
├── DataGridBody.vue          # 表体渲染
├── DataGridCell.vue          # 单元格（展示/编辑双模式）
├── DataGridStatusBar.vue     # 底部状态栏 + 分页
├── types.ts                  # 类型定义
├── useDataGridEditing.ts     # 编辑状态管理
├── useDataGridFilters.ts     # 过滤器状态管理
├── useDataGridColumns.ts     # 列定义生成
└── filterParser.ts           # 列过滤表达式解析
```

### 7.4 排序（UI-only）

- 仅对当前已加载数据片段排序（Naive DataTable `sorter`）
- 排序比较规则：
  - 可转数字则按数值
  - 否则按字符串比较（`localeCompare`）
  - `null/undefined` 排序靠前

> 说明：此策略保证 MVP 能用，但不等价于"全表排序"。当用户需要全表排序时，应升级后端能力或明确提示。

### 7.5 数据表格单元格渲染（DataGrid）

DataGrid 自定义组件的单元格渲染规范：

- **Vector 列**：默认折叠显示 `[维度d] val1, val2, …`，hover/click 展示完整值；**只读不可编辑**
- **布尔列**：使用 ✅/❌ 图标；编辑模式为 true/false/null 三态选择
- **null 值**：`text-slate-300 italic` 样式，显示 `NULL`；编辑时输入 `NULL` 设为 null
- **长文本**：支持行内展开（点击单元格）；编辑模式为 textarea
- **数字列**：右对齐显示；编辑模式为 number input
- **Binary 列**：显示 `Binary(N)`；**只读不可编辑**
- **dirty 单元格**：黄色底色（`amber-50`）+ 左侧 3px 黄色竖线指示
- **待删除行**：红色底色 + 删除线 + 半透明
- **新增行**：绿色底色（`green-50`）

### 7.6 DataGrid 键盘快捷键

| 快捷键 | 操作 |
|--------|------|
| `F5` / `Ctrl+R` | 刷新数据 |
| `Ctrl+Enter` | 执行查询（高级筛选面板内） |
| `Ctrl+S` | 保存所有更改 |
| `Escape` | 取消编辑 / 收起面板 |
| `Enter` | 确认编辑，移到下一行 |
| `Tab` / `Shift+Tab` | 移到下一/上一列 |
| `F2` / 双击 | 进入编辑模式 |
| `Delete` | 标记行待删除 |
| `Ctrl+Shift+F` | 切换高级筛选面板 |
| `Ctrl+N` | 添加新行 |

---

## 8. 动效（TailwindCSS）规范

### 8.1 使用范围

- ✅ 允许：
  - Sidebar 宽度折叠/展开（CSS transition width）
  - 连接项“表列表”展开/收起（CSS transition height）
  - 小范围淡入淡出（CSS transition opacity）
- ❌ 避免：
  - 对右侧大表格区域做大幅动画（容易卡顿/重排）

### 8.2 参数建议（统一手感）

统一使用 TailwindCSS 的过渡工具类，避免在 WebView 内引入额外动画运行时。

- Sidebar 宽度：`transition-[width] duration-[250ms] ease-out`
- 展开/收起：`transition-[height] duration-200 ease-out`
- 淡入淡出：`transition-opacity duration-150 ease-out`

> 说明：能用 `transform/opacity` 的动画优先用它们；仅在必要时（Sidebar 宽度、折叠面板高度）使用 `width/height` 过渡。

### 8.3 可维护性要求

- 动效由“状态（ref/computed）→ class/style 绑定”驱动，避免手动操作 DOM
- 依赖尺寸的过渡：尽量使用明确的像素高度（例如表列表 `min(行数 * 行高, maxHeight)`）
- 动效实现应可被 Biome/TS 静态检查覆盖（不依赖运行时注入）

---

## 9. 图标（Lucide）规范

- 图标库：`lucide-vue-next`
- 默认尺寸：
  - 主按钮：`h-4 w-4`
  - 小按钮：`h-3 w-3`
- 图标与文字间距：`ml-1`（小按钮）或 `ml-2`（普通按钮）

原则：

- 图标语义优先（避免“装饰性图标”过多）
- 同一动作使用同一图标（例如 Refresh 一律 `RefreshCcw`）

---

## 10. 状态模型（多连接并行）

### 10.1 核心状态

- Profiles：`useProfiles()`
  - `profiles: StoredProfile[]`
  - `activeProfileId: string | null`
- Connections：`useConnection(profiles, activeProfileId)`
  - `connectionStates: Record<profileId, ConnectionState>`
  - `activeConnection: ConnectionState | null`

每个 `ConnectionState`（按 profileId 隔离）：

- `connectionId`
- `tables`
- `activeTableName` / `activeTableId`
- `schema`
- `isConnecting/isRefreshing/isOpening`

### 10.2 交互约束

- 多连接可同时连接
- 右侧正文始终以 **activeProfileId** 作为“当前上下文”
- 打开表时只影响对应连接的 `activeTable*` 与 `schema`

---

## 11. 工程约束（UI 相关）

- 组件统一使用 `<script setup lang="ts">`
- TypeScript `strict`：避免 `any`，优先 `unknown` 并做 narrowing
- Tailwind 仅用于布局/间距/字体颜色等“轻量样式”；复杂交互依赖 Naive 组件
- 前端代码调整完成后，及时跑 `bun lint` 或 `bun check` 来发现问题；大改动前后都需要跑 `bun lint`（Biome）与 `bun build`

---

## 12. 状态 / 空态 / 加载态规范

### 12.1 空态（Empty）— 分级设计

| 场景 | 组件/实现 | 描述 |
|------|----------|------|
| 首次使用（无连接） | 大插图 + CTA 按钮 | "欢迎使用 LanceDB Studio — 创建您的第一个连接" |
| 有连接但未选表 | `NEmpty` + 引导 | "在左侧选择一个表以开始" |
| 已连接但表列表为空 | `NEmpty` + "创建表"按钮 | "当前数据库中没有表" |
| 查询无结果 | `NEmpty` | "没有匹配的数据，尝试调整过滤条件" |
| 未连接 | 灰色状态灯 + 内联操作按钮 | 表列表区域提示连接 |

### 12.2 加载态（Loading）— Skeleton 优先

- **Sidebar 连接列表首次加载**：显示 2-3 个 skeleton card 占位
- **Schema 加载**：`NDataTable :loading` + skeleton 行占位（避免整页空白）
- **数据浏览 Scan**：表格区域 `NDataTable :loading`，保留列头
- **连接中**：连接按钮 `loading` 状态；禁止重复触发
- **刷新表列表**：刷新按钮 `loading` 状态；禁止重复触发
- **打开表**：表按钮/行显示 loading；Schema 清空后再加载

### 12.3 错误与成功反馈

反馈机制遵循 §6.1 统一规范：

- **操作成功**：`NMessage`（toast），持续 3-5 秒
- **操作失败**：右侧正文顶部 `NAlert type=error`（persistent），手动关闭或下次操作时自动清除
- **操作进度**（scan、index build）：内联 loading 状态 + 进度文本

错误文案要求：
  - 优先给用户“下一步动作”（例如：检查 URI/权限/网络）
  - 不展示不可读的原始堆栈（必要时提供“复制详情”入口，后续可做）

---

## 13. 可访问性（A11y）与交互细节

- 键盘可用：
  - 主要按钮可 Tab 聚焦
  - Enter/Space 触发按钮
- 可点击区域：连接项主体按钮使用 `<button>`，确保可聚焦与可读
- 颜色不是唯一信息：状态灯之外应辅以文案/Tag（例如“已连接/未连接”可在后续补齐）
- 动效可控：后续建议支持“减少动效”偏好（例如检测 `prefers-reduced-motion`）

---

## 14. 性能与虚拟化规范

- 列表（连接/表）默认支持虚拟化：`NVirtualList`
- 虚拟化渲染项必须：
  - itemSize 与真实高度一致（避免滚动抖动）
  - 每个 item 避免昂贵同步测量（如频繁读写 layout）
- 表格：
  - 先做分页/片段渲染（当前实现：scan + limit/offset）
  - 未来 Arrow IPC 落地后，再评估更大行数与更丰富交互

---

## 15. UI 开发 Checklist（提交前自检）

- [ ] 新组件使用 `<script setup lang="ts">`，无 `any`
- [ ] 交互入口一致（连接/刷新/打开表都会先选中连接）
- [ ] 空态/加载态/错误态齐全
- [ ] 动效不影响正文主区域性能（避免大面积 reflow）
- [ ] 列表规模大时仍流畅（虚拟化开启、滚动不抖）
- [ ] 通过 `bun lint`（或 `bun check`）与 `bun build`

---

## 16. Roadmap（UI 视角）

### Phase 1 — 核心体验修复（立即执行）

- ✅ ConnectionItem 增加内联操作按钮（连接/断开/刷新）
- ✅ 数据浏览查询区精简 + "删除表"按钮移至 Schema Tab
- ✅ 取消数据页面 `max-width` 限制
- ✅ 错误反馈改为 persistent `NAlert`
- ✅ 表单标签样式升级（`text-sm text-slate-600 font-medium`）
- **面包屑 + Tab 栏固定为 Sticky Header**（最高优先级）
- **去掉 DataBrowseTab 的 `max-h-[70vh]` 限制**
- **NTabs 启用 `display-directive="if"` 按需渲染**

### Phase 2 — 信息架构重组

- **Tab 精简**：7 个 → 4 个（Schema / 数据 / 索引 / 版本）
- **数据写入与浏览合并**为单个"数据"Tab（DBeaver 模式）
- **导入导出改为 Modal**，入口为表右键菜单 + 工具栏按钮
- **维护（Compact/Vacuum）改为 Modal**，入口为表右键菜单
- ExplorerView 拆分：创建表 / 重命名表 Modal 独立为子组件
- 面包屑导航 + 单层 Tab 替代双层 Tab（参见 §2.1）
- 折叠态连接卡片重设计（首字母图标 + Tooltip 信息卡）
- 表概要信息条（行数 / 版本 / 索引数 / 文件大小）

### Phase 3 — 体验提升

- 空态/加载态 Skeleton 完善（参见 §12.2）
- ~~数据表格单元格渲染优化（Vector/布尔/null 列，参见 §7.5）~~ → 由 DataGrid 统一实现
- ~~**数据表格组件抽取复用**（Explorer + Search 共用 `DataResultTable.vue`）~~ → Explorer 改用 DataGrid
- 版本时间线可视化（NTimeline 组件）
- 键盘快捷键（`Ctrl+Enter` 执行查询等）
- `useCommand` composable 统一 IPC 调用模式（参见 §17）
- PrimaryNav 优化（收窄到 48px 或合并到 Sidebar）
- 深色模式预留

### Phase 4 — DataGrid 自定义表格（核心升级）

> 完整设计：`dev-docs/DATA_TABLE_REDESIGN.md`

#### Phase 4a — DataGrid MVP
- **自定义 DataGrid 组件**替换 DataTab 中的 NDataTable（`src/components/datagrid/`）
- **表头过滤器行**：每列输入框 → 后端过滤（filter expression 构建）
- **单元格内联编辑**：双击编辑 + dirty 标记 + 批量保存
- **统一工具栏**：刷新 / 过滤器 toggle / 添加行 / 保存 / 导出
- **高级筛选面板**：可折叠式面板，固定在顶部不遮盖表格
- **底部状态栏**：分页 + 行数统计 + dirty 计数
- **列宽拖拽调整**

#### Phase 4b — DataGrid 增强
- 虚拟滚动（大数据集优化）
- 固定列（左/右 pin）
- 列头右键菜单（排序/隐藏/筛选）
- 行选择（多选 checkbox）
- 复制/粘贴单元格

#### Phase 4c — DataGrid 高级
- 可视化条件构建器（AND/OR 条件组）
- 列拖拽排序
- SearchView 也替换为 DataGrid
- 单元格条件格式化（样式规则）

### 持续性目标

- Sidebar 筛选（Local/S3/Remote）+ 全局导航
- ~~Explorer 补齐 DBeaver 风格能力（列宽/列固定/快捷筛选）~~ → 已规划到 Phase 4（DataGrid）
- Search 复用 DataGrid 组件（Phase 4c）
- 性能：启用 Arrow IPC（`format: "arrow"`）

---

## 17. IPC 调用统一模式

### 17.1 useCommand composable

提取统一的 IPC 调用 + 错误处理模式，消除各提交函数中重复的 try/catch：

```ts
const { execute, isLoading, error } = useCommand(
  async (args: Args) => {
    return unwrapEnvelope(await someIpcCall(args))
  },
  {
    onSuccess: (result) => setStatus("操作完成"),
    onError: (err) => setError(err),
  }
)
```

- `isLoading`：自动跟踪执行状态，绑定按钮 `:loading`
- `error`：自动捕获并暴露错误信息，绑定 NAlert
- 避免每个命令函数手写 `try { ... } catch (e) { errorMessage.value = ... }`

### 17.2 错误类型统一

- IPC 返回的 `Envelope` 已做统一解包（`unwrapEnvelope`）
- 前端错误分类：`NetworkError` / `ValidationError` / `ServerError`
- 各类错误对应不同的用户提示策略

---

## 18. 路由规范

### 18.1 Explorer 子路由

Explorer 精简为 4 个常驻 Tab，低频功能通过 Modal 访问（无对应路由）：

```
# 常驻 Tab 路由
/connections/:id/table/:name/schema
/connections/:id/table/:name/data        # 浏览 + 写入合并
/connections/:id/table/:name/indexes
/connections/:id/table/:name/versions

# 已移除的路由（功能改为 Modal 入口）：
# /connections/:id/table/:name/write          → 合并到 /data
# /connections/:id/table/:name/import-export  → Modal
# /connections/:id/table/:name/maintenance    → Modal
```

### 18.2 路由 meta

- `layout: 'dialog'`：弹窗类路由（新建/编辑连接），使用独立布局
- `requiresConnection: true`：需要活跃连接的路由，无连接时重定向到欢迎页
- `fullWidth: true`：数据浏览等需要全宽的页面

### 18.3 keep-alive 策略

- Explorer Tab 切换时保留各 Tab 组件状态（避免每次切换都重新加载数据）
- 切换**不同表**时销毁旧组件、重建新组件（通过 `:key="tableId"` 控制）
