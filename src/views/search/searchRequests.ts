import type {
	CombinedSearchRequestV1,
	FtsSearchRequestV1,
	QueryFilterRequestV1,
	VectorSearchRequestV1,
} from "../../ipc/v1"

type RequestResult<T> = { ok: true; request: T } | { ok: false; message: string }

function optionalArray<T>(value: T[]): T[] | undefined {
	return value.length ? value : undefined
}

function optionalText(value: string): string | undefined {
	const cleaned = value.trim()
	return cleaned ? cleaned : undefined
}

export function parseVectorInputValue(value: string): number[] | null {
	const cleaned = value.replace(/[[\]]/g, " ")
	const parts = cleaned.split(/[,\s]+/).filter(Boolean)
	const numbers = parts.map((part) => Number(part))
	if (!numbers.length || numbers.some((value) => !Number.isFinite(value))) {
		return null
	}
	return numbers
}

export function buildFilterQueryRequest(input: {
	tableId: string
	filter: string
	projection: string[]
	limit: number
	offset: number
}): RequestResult<QueryFilterRequestV1> {
	const filter = optionalText(input.filter)
	if (!filter) {
		return { ok: false, message: "请输入过滤表达式" }
	}
	return {
		ok: true,
		request: {
			tableId: input.tableId,
			filter,
			projection: optionalArray(input.projection),
			limit: input.limit,
			offset: input.offset,
		},
	}
}

export function buildVectorSearchRequest(input: {
	tableId: string
	vectorText: string
	column: string | null
	topK: number
	offset: number
	projection: string[]
	filter: string
	nprobes: number | null
	refineFactor: number | null
}): RequestResult<VectorSearchRequestV1> {
	const vector = parseVectorInputValue(input.vectorText)
	if (!vector) {
		return { ok: false, message: "请输入有效向量（例如：0.1, 0.2, 0.3）" }
	}
	return {
		ok: true,
		request: {
			tableId: input.tableId,
			vector,
			column: input.column ?? undefined,
			topK: input.topK,
			offset: input.offset,
			projection: optionalArray(input.projection),
			filter: optionalText(input.filter),
			nprobes: input.nprobes ?? undefined,
			refineFactor: input.refineFactor ?? undefined,
		},
	}
}

export function buildFtsSearchRequest(input: {
	tableId: string
	query: string
	columns: string[]
	limit: number
	offset: number
	projection: string[]
	filter: string
}): RequestResult<FtsSearchRequestV1> {
	const query = optionalText(input.query)
	if (!query) {
		return { ok: false, message: "请输入查询文本" }
	}
	return {
		ok: true,
		request: {
			tableId: input.tableId,
			query,
			columns: optionalArray(input.columns),
			limit: input.limit,
			offset: input.offset,
			projection: optionalArray(input.projection),
			filter: optionalText(input.filter),
		},
	}
}

export function buildCombinedSearchRequest(input: {
	tableId: string
	query: string
	vectorText: string
	vectorColumn: string | null
	columns: string[]
	limit: number
	offset: number
	projection: string[]
	filter: string
	nprobes: number | null
	refineFactor: number | null
}): RequestResult<CombinedSearchRequestV1> {
	const query = optionalText(input.query)
	const vectorText = optionalText(input.vectorText)
	const vector = vectorText ? parseVectorInputValue(vectorText) : null
	if (!query || !vectorText) {
		return { ok: false, message: "混合检索需要同时输入查询文本和向量" }
	}
	if (!vector) {
		return { ok: false, message: "请输入有效向量（例如：0.1, 0.2, 0.3）" }
	}
	return {
		ok: true,
		request: {
			tableId: input.tableId,
			vector: vector ?? undefined,
			vectorColumn: input.vectorColumn ?? undefined,
			query,
			columns: optionalArray(input.columns),
			projection: optionalArray(input.projection),
			filter: optionalText(input.filter),
			limit: input.limit,
			offset: input.offset,
			nprobes: input.nprobes ?? undefined,
			refineFactor: input.refineFactor ?? undefined,
		},
	}
}
