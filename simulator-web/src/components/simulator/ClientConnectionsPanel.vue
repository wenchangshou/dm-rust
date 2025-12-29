<template>
    <el-card class="client-connections-panel">
        <template #header>
            <div class="panel-header">
                <div class="header-left">
                    <el-icon class="header-icon">
                        <Connection />
                    </el-icon>
                    <span class="header-title">客户端连接</span>
                    <el-badge :value="clients.length" :hidden="clients.length === 0" class="client-count" />
                </div>
                <div class="header-right">
                    <el-button size="small" circle @click="handleRefresh" :loading="loading">
                        <el-icon>
                            <Refresh />
                        </el-icon>
                    </el-button>
                </div>
            </div>
        </template>

        <div class="panel-content">
            <div v-if="clients.length === 0" class="empty-state">
                <el-icon class="empty-icon">
                    <Connection />
                </el-icon>
                <p class="empty-text">暂无客户端连接</p>
            </div>

            <div v-else class="client-list">
                <div v-for="client in clients" :key="client.id" class="client-item">
                    <div class="client-info">
                        <div class="client-addr">
                            <el-icon>
                                <Monitor />
                            </el-icon>
                            <span class="addr-text">{{ client.peer_addr }}</span>
                        </div>
                        <div class="client-meta">
                            <span class="meta-item">
                                <el-icon>
                                    <Clock />
                                </el-icon>
                                {{ formatDuration(client.connected_at) }}
                            </span>
                            <span class="meta-item">
                                <el-icon>
                                    <Download />
                                </el-icon>
                                {{ formatBytes(client.bytes_received) }}
                            </span>
                            <span class="meta-item">
                                <el-icon>
                                    <Upload />
                                </el-icon>
                                {{ formatBytes(client.bytes_sent) }}
                            </span>
                        </div>
                    </div>
                    <div class="client-actions">
                        <el-popconfirm title="确定断开此连接？" confirm-button-text="断开" cancel-button-text="取消"
                            @confirm="handleDisconnect(client.id)">
                            <template #reference>
                                <el-button size="small" type="danger" plain :loading="disconnecting === client.id">
                                    断开
                                </el-button>
                            </template>
                        </el-popconfirm>
                    </div>
                </div>
            </div>
        </div>
    </el-card>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { Connection, Refresh, Monitor, Clock, Download, Upload } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { listClients, disconnectClient } from '@/api/simulator'
import type { ClientConnection } from '@/types/simulator'

const props = defineProps<{
    simulatorId: string
}>()

const clients = ref<ClientConnection[]>([])
const loading = ref(false)
const disconnecting = ref<string | null>(null)
let refreshInterval: ReturnType<typeof setInterval> | null = null

// 获取客户端列表
async function fetchClients() {
    loading.value = true
    try {
        clients.value = await listClients(props.simulatorId)
    } catch (error) {
        console.error('获取客户端列表失败:', error)
    } finally {
        loading.value = false
    }
}

// 刷新
function handleRefresh() {
    fetchClients()
}

// 断开连接
async function handleDisconnect(clientId: string) {
    disconnecting.value = clientId
    try {
        await disconnectClient(props.simulatorId, clientId)
        ElMessage.success('已断开连接')
        await fetchClients()
    } catch (error) {
        ElMessage.error('断开连接失败')
        console.error('断开连接失败:', error)
    } finally {
        disconnecting.value = null
    }
}

// 格式化字节数
function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

// 格式化时长
function formatDuration(isoDate: string): string {
    const start = new Date(isoDate)
    const now = new Date()
    const seconds = Math.floor((now.getTime() - start.getTime()) / 1000)

    if (seconds < 60) return `${seconds}秒`
    if (seconds < 3600) return `${Math.floor(seconds / 60)}分钟`
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}小时`
    return `${Math.floor(seconds / 86400)}天`
}

onMounted(() => {
    fetchClients()
    // 每 3 秒刷新一次
    refreshInterval = setInterval(fetchClients, 3000)
})

onBeforeUnmount(() => {
    if (refreshInterval) {
        clearInterval(refreshInterval)
    }
})
</script>

<style lang="scss" scoped>
.client-connections-panel {
    background: var(--bg-card);
    border: 1px solid var(--border-glass);
    border-radius: 16px;

    :deep(.el-card__header) {
        padding: 16px 20px;
        border-bottom: 1px solid var(--border-glass);
    }

    :deep(.el-card__body) {
        padding: 0;
    }
}

.panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;

    .header-left {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .header-icon {
        font-size: 18px;
        color: var(--primary);
    }

    .header-title {
        font-weight: 600;
        color: var(--text-primary);
    }

    .client-count {
        margin-left: 4px;
    }
}

.panel-content {
    min-height: 120px;
}

.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 32px;
    color: var(--text-muted);

    .empty-icon {
        font-size: 36px;
        margin-bottom: 12px;
        opacity: 0.5;
    }

    .empty-text {
        margin: 0;
        font-size: 14px;
    }
}

.client-list {
    padding: 8px 0;
}

.client-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    border-bottom: 1px solid var(--border-glass);
    transition: background 0.2s;

    &:last-child {
        border-bottom: none;
    }

    &:hover {
        background: rgba(var(--primary-rgb), 0.05);
    }
}

.client-info {
    flex: 1;
}

.client-addr {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 500;
    color: var(--text-primary);
    margin-bottom: 4px;

    .el-icon {
        color: var(--primary);
    }

    .addr-text {
        font-family: 'JetBrains Mono', monospace;
    }
}

.client-meta {
    display: flex;
    align-items: center;
    gap: 16px;
    font-size: 12px;
    color: var(--text-secondary);

    .meta-item {
        display: flex;
        align-items: center;
        gap: 4px;

        .el-icon {
            font-size: 12px;
        }
    }
}

.client-actions {
    margin-left: 16px;
}
</style>
