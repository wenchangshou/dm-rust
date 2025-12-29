<template>
    <div class="mqtt-simulator-list">
        <!-- 头部 -->
        <div class="list-header">
            <h3>MQTT 模拟器</h3>
            <el-button type="primary" @click="showCreateDialog = true">
                <el-icon>
                    <Plus />
                </el-icon>
                创建
            </el-button>
        </div>

        <!-- 列表 -->
        <el-table :data="simulators" v-loading="loading" stripe>
            <el-table-column prop="name" label="名称" min-width="150">
                <template #default="{ row }">
                    <span class="name-link" @click="handleDetail(row)">{{ row.name }}</span>
                </template>
            </el-table-column>
            <el-table-column prop="mode" label="模式" width="100">
                <template #default="{ row }">
                    <el-tag :type="row.mode === 'broker' ? 'primary' : 'warning'" size="small">
                        {{ row.mode === 'broker' ? 'Broker' : '代理' }}
                    </el-tag>
                </template>
            </el-table-column>
            <el-table-column label="地址" width="200">
                <template #default="{ row }">
                    <div class="address-info">
                        <template v-if="row.mqtt_versions.includes('v4') && row.mqtt_versions.includes('v5')">
                            <code class="address-code">v4: {{ row.bind_addr }}:{{ row.port }}</code>
                            <code class="address-code">v5: {{ row.bind_addr }}:{{ row.port + 1 }}</code>
                        </template>
                        <template v-else>
                            <code class="address-code">{{ row.bind_addr }}:{{ row.port }}</code>
                        </template>
                    </div>
                </template>
            </el-table-column>
            <el-table-column label="MQTT 版本" width="120">
                <template #default="{ row }">
                    <div class="version-tags">
                        <el-tag v-for="version in row.mqtt_versions" :key="version" size="small" type="info">
                            {{ version.toUpperCase() }}
                        </el-tag>
                    </div>
                </template>
            </el-table-column>
            <el-table-column label="状态" width="100">
                <template #default="{ row }">
                    <el-tag :type="getStatusType(row.status)" size="small">
                        {{ getStatusText(row.status) }}
                    </el-tag>
                </template>
            </el-table-column>
            <el-table-column label="连接数" width="80" align="center">
                <template #default="{ row }">
                    <span class="stat-value">{{ row.state?.stats?.active_connections || 0 }}</span>
                </template>
            </el-table-column>
            <el-table-column label="消息数" width="100" align="center">
                <template #default="{ row }">
                    <span class="stat-value">{{ row.state?.stats?.messages_received || 0 }}</span>
                </template>
            </el-table-column>
            <el-table-column label="操作" width="240">
                <template #default="{ row }">
                    <div class="action-buttons">
                        <el-button v-if="row.status === 'stopped'" type="success" size="small"
                            @click="handleStart(row)">
                            <el-icon>
                                <VideoPlay />
                            </el-icon>
                            启动
                        </el-button>
                        <el-button v-else type="warning" size="small" @click="handleStop(row)">
                            <el-icon>
                                <VideoPause />
                            </el-icon>
                            停止
                        </el-button>
                        <el-button type="primary" size="small" @click="handleDetail(row)">
                            <el-icon>
                                <View />
                            </el-icon>
                            详情
                        </el-button>
                        <el-button type="danger" size="small" @click="handleDelete(row)">
                            <el-icon>
                                <Delete />
                            </el-icon>
                            删除
                        </el-button>
                    </div>
                </template>
            </el-table-column>
        </el-table>

        <!-- 创建对话框 -->
        <el-dialog v-model="showCreateDialog" title="创建 MQTT 模拟器" width="500px">
            <el-form :model="createForm" label-width="100px">
                <el-form-item label="名称" required>
                    <el-input v-model="createForm.name" placeholder="请输入名称" />
                </el-form-item>
                <el-form-item label="描述">
                    <el-input v-model="createForm.description" type="textarea" :rows="2" placeholder="描述（可选）" />
                </el-form-item>
                <el-form-item label="模式" required>
                    <el-radio-group v-model="createForm.mode">
                        <el-radio value="broker">Broker 模式</el-radio>
                        <el-radio value="proxy">代理模式</el-radio>
                    </el-radio-group>
                </el-form-item>
                <el-form-item label="端口" required>
                    <el-input-number v-model="createForm.port" :min="1" :max="65535" />
                </el-form-item>
                <el-form-item label="MQTT 版本">
                    <el-checkbox-group v-model="createForm.mqtt_versions">
                        <el-checkbox value="v4">MQTT v3.1/v3.1.1 (v4)</el-checkbox>
                        <el-checkbox value="v5">MQTT v5.0</el-checkbox>
                    </el-checkbox-group>
                    <div class="form-tip">
                        <el-icon><InfoFilled /></el-icon>
                        <span v-if="createForm.mqtt_versions.length === 2">
                            同时启用时：v4 使用端口 {{ createForm.port }}，v5 使用端口 {{ createForm.port + 1 }}
                        </span>
                        <span v-else>
                            可同时支持多个版本，默认仅支持 v4
                        </span>
                    </div>
                </el-form-item>
                <el-form-item label="绑定地址">
                    <el-input v-model="createForm.bind_addr" placeholder="0.0.0.0" />
                </el-form-item>

                <!-- 代理模式配置 -->
                <template v-if="createForm.mode === 'proxy'">
                    <el-divider>上游 Broker 配置</el-divider>
                    <el-form-item label="主机地址" required>
                        <el-input v-model="proxyConfig.upstream_host" placeholder="broker.example.com" />
                    </el-form-item>
                    <el-form-item label="端口" required>
                        <el-input-number v-model="proxyConfig.upstream_port" :min="1" :max="65535" />
                    </el-form-item>
                    <el-form-item label="用户名">
                        <el-input v-model="proxyConfig.upstream_username" placeholder="可选" />
                    </el-form-item>
                    <el-form-item label="密码">
                        <el-input v-model="proxyConfig.upstream_password" type="password" placeholder="可选" />
                    </el-form-item>
                </template>

                <el-form-item label="自动启动">
                    <el-switch v-model="createForm.auto_start" />
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button @click="showCreateDialog = false">取消</el-button>
                <el-button type="primary" :loading="creating" @click="handleCreate">创建</el-button>
            </template>
        </el-dialog>


    </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { Plus, VideoPlay, VideoPause, View, Delete, InfoFilled } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
    listMqttSimulators,
    createMqttSimulator,
    startMqttSimulator,
    stopMqttSimulator,
    deleteMqttSimulator,
    type MqttSimulatorInfo,
    type MqttMode,
    type MqttVersion,
    type MqttSimulatorStatus,
    type ProxyConfig,
} from '@/api/mqtt'
import { useRouter } from 'vue-router'

const router = useRouter()
const loading = ref(false)
const simulators = ref<MqttSimulatorInfo[]>([])
const showCreateDialog = ref(false)
const creating = ref(false)

const createForm = reactive({
    name: '',
    description: '',
    mode: 'broker' as MqttMode,
    port: 1883,
    bind_addr: '0.0.0.0',
    mqtt_versions: ['v4'] as MqttVersion[],
    auto_start: false,
})

const proxyConfig = reactive<ProxyConfig>({
    upstream_host: '',
    upstream_port: 1883,
    upstream_username: undefined,
    upstream_password: undefined,
    client_id_prefix: 'proxy_',
})

onMounted(() => {
    fetchSimulators()
})

async function fetchSimulators() {
    loading.value = true
    try {
        simulators.value = await listMqttSimulators()
    } catch (e) {
        console.error(e)
    } finally {
        loading.value = false
    }
}

function getStatusType(status: MqttSimulatorStatus): string {
    if (status === 'running') return 'success'
    if (status === 'stopped') return 'info'
    return 'danger'
}

function getStatusText(status: MqttSimulatorStatus): string {
    if (status === 'running') return '运行中'
    if (status === 'stopped') return '已停止'
    if (typeof status === 'object' && status.error) return '错误'
    return '未知'
}

async function handleCreate() {
    if (!createForm.name) {
        ElMessage.warning('请输入名称')
        return
    }

    creating.value = true
    try {
        const request = {
            ...createForm,
            proxy_config: createForm.mode === 'proxy' ? proxyConfig : undefined,
        }
        await createMqttSimulator(request)
        ElMessage.success('创建成功')
        showCreateDialog.value = false
        await fetchSimulators()
        // 重置表单
        createForm.name = ''
        createForm.description = ''
        createForm.mode = 'broker'
        createForm.port = 1883
        createForm.mqtt_versions = ['v4']
        createForm.auto_start = false
    } catch (e: any) {
        ElMessage.error(e.message || '创建失败')
    } finally {
        creating.value = false
    }
}

async function handleStart(simulator: MqttSimulatorInfo) {
    try {
        await startMqttSimulator(simulator.id)
        ElMessage.success('启动成功')
        await fetchSimulators()
    } catch (e: any) {
        ElMessage.error(e.message || '启动失败')
    }
}

async function handleStop(simulator: MqttSimulatorInfo) {
    try {
        await stopMqttSimulator(simulator.id)
        ElMessage.success('停止成功')
        await fetchSimulators()
    } catch (e: any) {
        ElMessage.error(e.message || '停止失败')
    }
}

async function handleDelete(simulator: MqttSimulatorInfo) {
    try {
        await ElMessageBox.confirm(`确定要删除 "${simulator.name}" 吗？`, '确认删除', { type: 'warning' })
        await deleteMqttSimulator(simulator.id)
        ElMessage.success('删除成功')
        await fetchSimulators()
    } catch (e: any) {
        if (e !== 'cancel') {
            ElMessage.error(e.message || '删除失败')
        }
    }
}

function handleDetail(simulator: MqttSimulatorInfo) {
    router.push(`/mqtt-simulator/${simulator.id}`)
}
</script>

<style lang="scss" scoped>
.mqtt-simulator-list {
    .list-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 16px;

        h3 {
            margin: 0;
            font-size: 16px;
            font-weight: 600;
        }
    }

    .name-link {
        color: var(--el-color-primary);
        cursor: pointer;
        font-weight: 500;

        &:hover {
            text-decoration: underline;
        }
    }

    .address-info {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .address-code {
        font-family: 'JetBrains Mono', monospace;
        font-size: 12px;
        background: var(--el-fill-color-light);
        padding: 2px 6px;
        border-radius: 4px;
        display: inline-block;
    }

    .stat-value {
        font-family: 'JetBrains Mono', monospace;
        font-weight: 600;
    }

    .version-tags {
        display: flex;
        gap: 4px;
        flex-wrap: wrap;
    }

    .action-buttons {
        display: flex;
        gap: 8px;
    }

    .form-tip {
        margin-top: 8px;
        font-size: 12px;
        color: var(--el-text-color-secondary);
        display: flex;
        align-items: center;
        gap: 4px;

        .el-icon {
            font-size: 14px;
        }
    }
}
</style>
