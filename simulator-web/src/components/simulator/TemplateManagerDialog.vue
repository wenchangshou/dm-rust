<template>
    <el-dialog title="模板管理" :model-value="visible" @update:model-value="emit('update:visible', $event)" width="800px"
        class="template-manager-dialog">
        <el-table :data="templates" v-loading="loading" style="width: 100%">
            <el-table-column prop="name" label="名称" width="180">
                <template #default="{ row }">
                    <span class="template-name">{{ row.name }}</span>
                </template>
            </el-table-column>
            <el-table-column prop="protocol" label="协议" width="100">
                <template #default="{ row }">
                    <el-tag size="small">{{ row.protocol }}</el-tag>
                </template>
            </el-table-column>
            <el-table-column prop="description" label="描述" min-width="200" show-overflow-tooltip />
            <el-table-column prop="created_at" label="创建时间" width="180">
                <template #default="{ row }">
                    {{ formatDate(row.created_at) }}
                </template>
            </el-table-column>
            <el-table-column label="操作" width="100" fixed="right">
                <template #default="{ row }">
                    <el-button type="danger" link size="small" @click="handleDelete(row)">
                        <el-icon>
                            <Delete />
                        </el-icon> 删除
                    </el-button>
                </template>
            </el-table-column>
        </el-table>

        <template #footer>
            <el-button @click="emit('update:visible', false)">关闭</el-button>
        </template>
    </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Delete } from '@element-plus/icons-vue'
import { ElMessageBox, ElMessage } from 'element-plus'
import * as simulatorApi from '@/api/simulator'
import type { SimulatorTemplate } from '@/types/simulator'

const props = defineProps<{
    visible: boolean
}>()

const emit = defineEmits<{
    'update:visible': [value: boolean]
    'changed': [] // 模板列表变化时触发
}>()

const templates = ref<SimulatorTemplate[]>([])
const loading = ref(false)

watch(() => props.visible, (val) => {
    if (val) {
        fetchTemplates()
    }
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
        emit('changed')
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
</script>

<style lang="scss" scoped>
.template-name {
    font-weight: 600;
    color: var(--el-text-color-primary);
}
</style>
