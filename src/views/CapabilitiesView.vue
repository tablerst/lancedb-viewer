<script setup lang="ts">
import { computed } from "vue"

const statusMap = {
	done: { label: "已实现", type: "success" },
	backend: { label: "后端已具备", type: "info" },
	todo: { label: "待接入", type: "warning" },
	plan: { label: "规划中", type: "default" },
} as const

type StatusKey = keyof typeof statusMap

type FeatureItem = {
	name: string
	status: StatusKey
	note?: string
}

type FeatureModule = {
	title: string
	description?: string
	items: FeatureItem[]
}

const modules = computed<FeatureModule[]>(() => [
	{
		title: "连接与库",
		description: "覆盖 Connection 与库级管理能力",
		items: [
			{ name: "连接/断开", status: "done", note: "connect 已实现" },
			{ name: "列出表", status: "done" },
			{ name: "创建表", status: "done" },
			{ name: "删除表", status: "done", note: "已接入 drop_table_v1" },
			{ name: "重命名表", status: "plan" },
		],
	},
	{
		title: "Schema 演化",
		description: "字段查看与演化操作",
		items: [
			{ name: "读取 Schema", status: "done" },
			{ name: "新增列", status: "done" },
			{ name: "修改列", status: "done" },
			{ name: "删除列", status: "done" },
		],
	},
	{
		title: "数据读写",
		description: "记录写入、更新与删除",
		items: [
			{ name: "Scan 浏览", status: "done" },
			{ name: "追加写入", status: "done" },
			{ name: "覆盖写入", status: "done" },
			{ name: "更新/删除", status: "done" },
		],
	},
	{
		title: "查询与检索",
		description: "过滤、向量与全文检索",
		items: [
			{ name: "过滤查询", status: "done" },
			{ name: "向量检索", status: "done" },
			{ name: "全文检索", status: "done" },
			{ name: "组合查询", status: "todo" },
		],
	},
	{
		title: "索引管理",
		description: "向量/全文索引创建与状态",
		items: [
			{ name: "索引列表", status: "done" },
			{ name: "创建向量索引", status: "done" },
			{ name: "创建全文索引", status: "done" },
			{ name: "删除索引", status: "done" },
		],
	},
	{
		title: "版本与时间旅行",
		description: "版本列表、回溯与克隆",
		items: [
			{ name: "版本列表", status: "todo" },
			{ name: "打开指定版本", status: "todo" },
			{ name: "克隆/分支", status: "todo" },
		],
	},
	{
		title: "导入导出与维护",
		description: "数据导入导出与维护操作",
		items: [
			{ name: "导入 (CSV/Parquet/JSONL)", status: "todo" },
			{ name: "导出", status: "todo" },
			{ name: "Compact/Vacuum", status: "todo" },
		],
	},
	{
		title: "安全与凭证",
		description: "凭证/授权与安全存储",
		items: [
			{ name: "Auth Descriptor 接入", status: "plan" },
			{ name: "Stronghold 存储", status: "plan" },
		],
	},
])

function getStatusMeta(status: StatusKey) {
	return statusMap[status]
}
</script>

<template>
	<div class="space-y-4">
		<NCard size="small" title="能力地图" class="shadow-sm">
			<div class="space-y-2 text-sm text-slate-600">
				<p>
					本页用于覆盖 LanceDB crate 能力的前端对齐情况，优先做功能扩展，性能与健壮性后续再优化。
				</p>
				<p class="text-xs text-slate-400">
					状态说明：已实现 / 后端已具备 / 待接入 / 规划中。
				</p>
			</div>
		</NCard>

		<div class="grid gap-4 lg:grid-cols-2">
			<NCard
				v-for="module in modules"
				:key="module.title"
				size="small"
				:title="module.title"
				class="shadow-sm"
			>
				<div v-if="module.description" class="mb-3 text-xs text-slate-500">
					{{ module.description }}
				</div>
				<div class="space-y-2">
					<div
						v-for="item in module.items"
						:key="item.name"
						class="flex items-center justify-between gap-3 rounded-md border border-slate-100 bg-slate-50/60 px-3 py-2"
					>
						<div class="min-w-0">
							<div class="truncate text-sm text-slate-700" :title="item.name">
								{{ item.name }}
							</div>
							<div v-if="item.note" class="text-[11px] text-slate-400">
								{{ item.note }}
							</div>
						</div>
						<NTag
							:size="'small'"
							:type="getStatusMeta(item.status).type"
						>
							{{ getStatusMeta(item.status).label }}
						</NTag>
					</div>
				</div>
			</NCard>
		</div>
	</div>
</template>
