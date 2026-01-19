<script setup lang="ts">
import { computed, onMounted, ref } from "vue";

import type { SchemaDefinition, TableInfo } from "./ipc/v1";
import type { StoredProfile } from "./models/profile";
import { toConnectProfile } from "./models/profile";
import { connectV1, getSchemaV1, listTablesV1, openTableV1, unwrapEnvelope } from "./lib/tauriClient";
import { createProfile, loadProfileState, saveProfileState } from "./stores/profiles";

const profiles = ref<StoredProfile[]>([]);
const activeProfileId = ref<string | null>(null);
const connectionId = ref<string | null>(null);
const tables = ref<TableInfo[]>([]);
const activeTableId = ref<string | null>(null);
const activeTableName = ref<string | null>(null);
const schema = ref<SchemaDefinition | null>(null);
const statusMessage = ref("");
const errorMessage = ref("");

const profileForm = ref({
  name: "",
  uri: "",
  storageOptionsJson: "{}",
});

const activeProfile = computed(() =>
  profiles.value.find((profile) => profile.id === activeProfileId.value) ?? null,
);

onMounted(async () => {
  const state = await loadProfileState();
  profiles.value = state.profiles;
  activeProfileId.value = state.activeProfileId;
});

function parseStorageOptions(raw: string): Record<string, string> {
  if (!raw.trim()) {
    return {};
  }
  const parsed = JSON.parse(raw) as Record<string, unknown>;
  if (parsed === null || Array.isArray(parsed) || typeof parsed !== "object") {
    throw new Error("storageOptions 必须是 JSON 对象");
  }
  return Object.fromEntries(
    Object.entries(parsed).map(([key, value]) => [key, String(value)]),
  );
}

async function persistProfiles() {
  await saveProfileState({
    profiles: profiles.value,
    activeProfileId: activeProfileId.value,
  });
}

async function addProfile() {
  errorMessage.value = "";
  statusMessage.value = "";
  const name = profileForm.value.name.trim();
  const uri = profileForm.value.uri.trim();
  if (!name || !uri) {
    errorMessage.value = "请填写连接名称与 URI";
    return;
  }

  try {
    const storageOptions = parseStorageOptions(profileForm.value.storageOptionsJson);
    const profile = createProfile({
      name,
      uri,
      storageOptions,
      auth: { type: "none" },
    });
    profiles.value = [...profiles.value, profile];
    activeProfileId.value = profile.id;
    await persistProfiles();
    profileForm.value = { name: "", uri: "", storageOptionsJson: "{}" };
    statusMessage.value = "连接档案已保存";
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "解析 storageOptions 失败";
  }
}

async function selectProfile(profileId: string) {
  activeProfileId.value = profileId;
  await persistProfiles();
}

async function connectActiveProfile() {
  errorMessage.value = "";
  statusMessage.value = "";
  const profile = activeProfile.value;
  if (!profile) {
    errorMessage.value = "请先选择连接档案";
    return;
  }

  try {
    const connectProfile = toConnectProfile(profile);
    connectProfile.auth ??= { type: "none" };
    const response = unwrapEnvelope(await connectV1(connectProfile));
    connectionId.value = response.connectionId;
    statusMessage.value = `已连接：${response.name}`;
    await refreshTables();
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "连接失败";
  }
}

async function refreshTables() {
  const id = connectionId.value;
  if (!id) {
    return;
  }
  try {
    const response = unwrapEnvelope(await listTablesV1(id));
    tables.value = response.tables;
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "拉取表列表失败";
  }
}

async function openTable(name: string) {
  const id = connectionId.value;
  if (!id) {
    return;
  }

  try {
    const handle = unwrapEnvelope(await openTableV1(id, name));
    activeTableId.value = handle.tableId;
    activeTableName.value = name;
    schema.value = unwrapEnvelope(await getSchemaV1(handle.tableId));
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "打开表失败";
  }
}
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <header class="panel-header">
        <h1>LanceDB Studio</h1>
        <p>JSON-first IPC · 可扩展连接抽象</p>
      </header>

      <section class="panel">
        <h2>连接档案</h2>
        <ul class="profile-list">
          <li v-for="profile in profiles" :key="profile.id">
            <button
              class="profile-button"
              :class="{ active: profile.id === activeProfileId }"
              @click="selectProfile(profile.id)"
            >
              <span>{{ profile.name }}</span>
              <small>{{ profile.uri }}</small>
            </button>
          </li>
        </ul>
        <button class="primary" @click="connectActiveProfile">连接当前档案</button>
      </section>

      <section class="panel">
        <h2>新增档案</h2>
        <div class="field">
          <label>名称</label>
          <input v-model="profileForm.name" placeholder="本地 LanceDB" />
        </div>
        <div class="field">
          <label>URI</label>
          <input v-model="profileForm.uri" placeholder="/path/to/db 或 s3://bucket/db" />
        </div>
        <div class="field">
          <label>storageOptions (JSON)</label>
          <textarea v-model="profileForm.storageOptionsJson" rows="4" />
        </div>
        <button class="secondary" @click="addProfile">保存档案</button>
      </section>
    </aside>

    <main class="content">
      <section class="panel">
        <h2>连接状态</h2>
        <p v-if="statusMessage" class="status">{{ statusMessage }}</p>
        <p v-if="errorMessage" class="error">{{ errorMessage }}</p>
        <p v-if="!connectionId" class="muted">尚未连接数据库</p>
      </section>

      <section class="panel">
        <div class="panel-row">
          <h2>表列表</h2>
          <button class="ghost" @click="refreshTables" :disabled="!connectionId">
            刷新
          </button>
        </div>
        <ul class="table-list">
          <li v-for="table in tables" :key="table.name">
            <button
              class="table-button"
              :class="{ active: table.name === activeTableName }"
              @click="openTable(table.name)"
            >
              {{ table.name }}
            </button>
          </li>
        </ul>
      </section>

      <section class="panel">
        <h2>Schema</h2>
        <div v-if="!schema" class="muted">选择表以查看 schema</div>
        <table v-else class="schema-table">
          <thead>
            <tr>
              <th>字段</th>
              <th>类型</th>
              <th>Nullable</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="field in schema.fields" :key="field.name">
              <td>{{ field.name }}</td>
              <td>{{ field.dataType }}</td>
              <td>{{ field.nullable ? "是" : "否" }}</td>
            </tr>
          </tbody>
        </table>
      </section>
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  display: grid;
  grid-template-columns: 320px 1fr;
  min-height: 100vh;
  background: #f6f7fb;
  color: #1f2328;
  font-family: "Inter", system-ui, sans-serif;
}

.sidebar {
  background: #0f172a;
  color: #e2e8f0;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.panel-header h1 {
  margin: 0 0 4px;
  font-size: 20px;
}

.panel-header p {
  margin: 0;
  font-size: 12px;
  color: #94a3b8;
}

.panel {
  background: #ffffff;
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 12px 28px rgba(15, 23, 42, 0.08);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.sidebar .panel {
  background: #1e293b;
  color: #e2e8f0;
  box-shadow: none;
}

.panel h2 {
  margin: 0;
  font-size: 14px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.profile-list,
.table-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.profile-button,
.table-button {
  border: none;
  background: rgba(148, 163, 184, 0.15);
  color: inherit;
  padding: 10px 12px;
  border-radius: 10px;
  text-align: left;
  width: 100%;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.profile-button small {
  color: #94a3b8;
}

.profile-button.active,
.table-button.active {
  background: #38bdf8;
  color: #0f172a;
}

.primary,
.secondary,
.ghost {
  border: none;
  border-radius: 10px;
  padding: 10px 12px;
  font-weight: 600;
  cursor: pointer;
}

.primary {
  background: #38bdf8;
  color: #0f172a;
}

.secondary {
  background: #334155;
  color: #e2e8f0;
}

.ghost {
  background: transparent;
  color: #475569;
}

.content {
  padding: 32px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.panel-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

input,
textarea {
  border-radius: 8px;
  border: 1px solid #cbd5f5;
  padding: 8px 10px;
  font-size: 14px;
  background: #ffffff;
}

.sidebar input,
.sidebar textarea {
  background: #0f172a;
  color: #e2e8f0;
  border: 1px solid #334155;
}

.status {
  color: #0f766e;
  font-weight: 600;
}

.error {
  color: #ef4444;
  font-weight: 600;
}

.muted {
  color: #64748b;
}

.schema-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}

.schema-table th,
.schema-table td {
  text-align: left;
  padding: 8px 10px;
  border-bottom: 1px solid #e2e8f0;
}

@media (max-width: 960px) {
  .app-shell {
    grid-template-columns: 1fr;
  }

  .sidebar {
    order: 2;
  }
}
</style>