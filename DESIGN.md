# DESIGN.md

本文件定义 **LanceDB Studio** 前端的标准 UI 规范与信息架构，用于：

- 指导后续页面/组件的实现与重构
- 保证交互一致性（侧边栏、连接卡片、表格操作）
- 为动效（GSAP）与图标（Lucide）提供统一约束

> 技术栈：Vue 3 + TypeScript + Naive UI + Tailwind + GSAP + Lucide

---

## 1. 设计目标与原则

### 1.1 产品目标

- 以 **连接（Connections）→ 表（Tables）→ 数据/检索（Data/Search）** 为主线
- 支持 **多连接并行**：多个连接同时保持在线，互不影响
- 大列表与大表数据要“不卡”：连接列表/表列表 **虚拟化**，数据表格先做 **分页/片段渲染**

### 1.2 交互原则

- **稳定画布**：右侧正文尽量不因侧边栏交互产生大幅 reflow
- **就地反馈**：成功/失败状态统一在右侧“连接状态”卡片中呈现（`NAlert`）
- **低认知负担**：相同动作在不同页面/组件保持同样入口和同样文案

### 1.3 MVP 约束

- 排序：仅做 **前端排序**（对“当前已加载数据片段”排序），不要求后端 scan 支持 order by
- 检索页：后端命令已存在，UI 后续按工作台方式接入（见 Roadmap）

---

## 2. 信息架构（IA）

整体布局采用“**左侧可收缩侧边栏 + 右侧正文工作区**”。

- 左侧 Sidebar：以连接管理为主（含表树）；全局导航后续补齐（Explorer/Search）
- 右侧正文：按当前路由展示 Explorer / Search 等工作台

当前落地文件：

- Layout Shell：`src/App.vue`
- Sidebar：`src/components/sidebar/Sidebar.vue`
- 连接项卡片：`src/components/sidebar/ConnectionItem.vue`
- Explorer：`src/views/ExplorerView.vue`

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
- 阴影：轻量（`shadow-sm`），避免重阴影造成“浮夸”

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

一致性要求：

- 任何“连接/刷新/打开表”的动作都必须 **先选中该连接**，确保右侧正文同步。

### 5.3 表列表（树形/展开）与虚拟化

- 当前为“表列表”最小形态（后续可扩展为树形）
- 展开高度：最多 `200px`，内部使用 `NVirtualList`（itemSize `32px`）

---

## 6. 右侧正文（Content）规范

### 6.1 顶部连接状态卡

`src/App.vue` 负责在右侧顶部固定渲染“连接状态”卡（`NCard`）。

内容：

- 成功消息：`statusMessage`（`NAlert type=success`）
- 错误消息：`errorMessage`（`NAlert type=error`）
- 当前活跃连接名 + 类型 Tag + 表数量

原则：

- 状态与错误信息统一入口，避免 toast 到处飞
- 文案要可行动：尽量提示“选择连接/点击连接按钮”等下一步

---

## 7. Explorer（表结构 + 数据浏览）规范

文件：`src/views/ExplorerView.vue`

### 7.1 Schema Tab

- 表结构表格：`NDataTable`
- 字段信息：name / dataType / nullable

### 7.2 数据浏览 Tab

提供：

- Limit / Offset
- Filter 表达式（目前以字符串输入）
- 列投影（`NSelect multiple`）
- 查询按钮

### 7.3 排序（UI-only）

- 仅对当前已加载数据片段排序（Naive DataTable `sorter`）
- 排序比较规则（现状）：
  - 可转数字则按数值
  - 否则按字符串比较（`localeCompare`）
  - `null/undefined` 排序靠前

> 说明：此策略保证 MVP 能用，但不等价于“全表排序”。当用户需要全表排序时，应升级后端能力或明确提示。

---

## 8. 动效（GSAP）规范

### 8.1 使用范围

- ✅ 允许：
  - Sidebar 宽度折叠/展开
  - 连接项“表列表”展开/收起（高度动画）
  - 小范围淡入淡出（例如过滤面板弹出）
- ❌ 避免：
  - 对右侧大表格区域做大幅动画（容易卡顿/重排）

### 8.2 参数建议（统一手感）

- Sidebar 宽度：`duration 0.25`，`ease power2.out`
- 展开/收起：`duration 0.2`，`ease power2.out`

### 8.3 可维护性要求

- 动画触发点尽量集中在组件内部
- 任何依赖 DOM 尺寸的动画需 `nextTick()` 后再启动
- 后续如引入更多动画，建议使用 `gsap.context()` 以便卸载清理

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
- 大改动前后都需要跑 `npm run lint`（Biome）与 `npm run build`

---

## 12. 状态 / 空态 / 加载态规范

### 12.1 空态（Empty）

- 无连接档案：使用 `NEmpty`，文案引导用户“新建连接/导入连接”
- 未连接：连接项卡片显示灰色状态灯，并在表列表区域显示“未连接”
- 未选择表：Explorer 详情卡片显示“选择表以查看详情”

### 12.2 加载态（Loading）

- 连接中：连接按钮 `loading`；避免重复触发
- 刷新表列表：刷新按钮 `loading`；避免重复触发
- 打开表：表按钮（或行）显示 loading；Schema 清空后再加载

### 12.3 错误与成功反馈

- 全局反馈统一汇聚到右侧“连接状态”卡片：
  - 成功：`NAlert type=success`
  - 失败：`NAlert type=error`
- 错误文案要求：
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
- [ ] 通过 `npm run lint` 与 `npm run build`

---

## 16. Roadmap（UI 视角）

- Sidebar：补齐筛选（Local/S3/Remote）+ 新建连接 Modal + 全局导航（Explorer/Search）
- Explorer：补齐常用 DBeaver 风格能力（搜索框、列宽/列固定、快捷筛选）
- Search：接入 `query_filter_v1 / vector_search_v1 / fts_search_v1`，复用结果表格
- 性能：启用 Arrow IPC（`format: "arrow"`）以提升大数据场景
