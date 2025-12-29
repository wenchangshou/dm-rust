<template>
    <div class="template-list">
        <el-card class="table-card">
            <template #header>
                <div class="table-header">
                    <span class="header-title">模板列表</span>
                    <span class="header-count">共 {{ templates.length }} 个</span>
                    <div class="header-actions">
                        <el-button type="primary" size="small" @click="fetchTemplates">
                            <el-icon>
                                <Refresh />
                            </el-icon> 刷新
                        </el-button>
                    </div>
                </div>
            </template>

            <el-table :data="templates" v-loading="loading" style="width: 100%">
                <el-table-column prop="name" label="名称" width="200">
                    <template #default="{ row }">
                        <span class="template-name">{{ row.name }}</span>
                    </template>
                </el-table-column>
                <el-table-column prop="protocol" label="协议" width="120">
                    <template #default="{ row }">
                        <el-tag size="small" class="protocol-tag">{{ row.protocol }}</el-tag>
                    </template>
                </el-table-column>
                <el-table-column prop="description" label="描述" min-width="250" show-overflow-tooltip />
                <el-table-column prop="created_at" label="创建时间" width="180">
                    <template #default="{ row }">
                        <span class="time-text">{{ formatDate(row.created_at) }}</span>
                    </template>
                </el-table-column>
                <el-table-column label="操作" width="150" fixed="right" align="center">
                    <template #default="{ row }">
                        <el-button type="danger" size="small" class="action-btn" @click="handleDelete(row)">
                            <el-icon>
                                <Delete />
                            </el-icon> 删除
                        </el-button>
                    </template>
                </el-table-column>
            </el-table>
        </el-card>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Delete, Refresh } from '@element-plus/icons-vue'
import { ElMessageBox, ElMessage } from 'element-plus'
import * as simulatorApi from '@/api/simulator'
import type { SimulatorTemplate } from '@/types/simulator'

const templates = ref<SimulatorTemplate[]>([])
const loading = ref(false)

onMounted(() => {
    fetchTemplates()
})

async function fetchTemplates() {
    loading.value = true
    try {
        templates.value = await simulatorApi.listTemplates()
    } catch (e: any) {
        ElMessage.error(e.message || '获取模板列表失败')
    } finally {
        loading.value = false
    }
}

async function handleDelete(template: SimulatorTemplate) {
    try {
        await ElMessageBox.confirm(
            `确定要删除模板 "${template.name}" 吗？`,
            '确认删除',
            { type: 'warning' }
        )
        await simulatorApi.deleteTemplate(template.id)
        ElMessage.success('模板已删除')
        fetchTemplates()
    } catch {
        // cancelled
    }
}

function formatDate(dateStr: string) {
    try {
        return new Date(dateStr).toLocaleString()
    } catch {
        return dateStr
    }
}

defineExpose({
    refresh: fetchTemplates
})
</script>

<style lang="scss" scoped>
.table-card {
    border-radius: 16px;

    .table-header {
        display: flex;
        align-items: center;
        gap: 12px;

        .header-title {
            font-weight: 600;
            color: var(--text-primary);
        }

        .header-count {
            font-size: 13px;
            color: var(--text-muted);
            background: var(--bg-input);
            padding: 4px 10px;
            border-radius: 20px;
        }

        .header-actions {
            margin-left: auto;
        }
    }
}

.template-name {
    font-weight: 600;
    color: #667eea;
}

.protocol-tag {
    background: rgba(102, 126, 234, 0.15);
    border-color: rgba(102, 126, 234, 0.3);
    color: #667eea;
    font-weight: 500;
}

.time-text {
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    color: var(--text-secondary);
}

.action-btn {
    padding: 6px 12px;
}
</style>
