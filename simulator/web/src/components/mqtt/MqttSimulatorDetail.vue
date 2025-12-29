<template>
    <div class="mqtt-simulator-detail">
        <!-- 基本信息 -->
        <el-descriptions :column="2" border class="info-section">
            <el-descriptions-item label="ID">
                <code>{{ simulator.id }}</code>
            </el-descriptions-item>
            <el-descriptions-item label="模式">
                <el-tag :type="simulator.mode === 'broker' ? 'primary' : 'warning'" size="small">
                    {{ simulator.mode === 'broker' ? 'Broker 模式' : '代理模式' }}
                </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="MQTT 版本">
                <div class="version-tags">
                    <el-tag v-for="version in simulator.mqtt_versions" :key="version" size="small" type="info">
                        {{ version.toUpperCase() }}
                    </el-tag>
                </div>
            </el-descriptions-item>
            <el-descriptions-item label="地址">
                <div class="address-info">
                    <template v-if="simulator.mqtt_versions.includes('v4') && simulator.mqtt_versions.includes('v5')">
                        <code>v4: {{ simulator.bind_addr }}:{{ simulator.port }}</code>
                        <code>v5: {{ simulator.bind_addr }}:{{ simulator.port + 1 }}</code>
                    </template>
                    <template v-else>
                        <code>{{ simulator.bind_addr }}:{{ simulator.port }}</code>
                    </template>
                </div>
            </el-descriptions-item>
            <el-descriptions-item label="状态">
                <el-tag :type="getStatusType(simulator.status)" size="small">
                    {{ getStatusText(simulator.status) }}
                </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="活动连接">
                <span class="stat-highlight">{{ simulator.state?.stats?.active_connections || 0 }}</span>
            </el-descriptions-item>
            <el-descriptions-item label="消息接收">
                <span class="stat-value">{{ simulator.state?.stats?.messages_received || 0 }}</span>
            </el-descriptions-item>
        </el-descriptions>

        <!-- 规则管理 -->
        <div class="section">
            <div class="section-header">
                <h4>
                    <el-icon>
                        <Setting />
                    </el-icon>
                    规则管理
                </h4>
                <el-button type="primary" size="small" @click="showAddRuleDialog = true">
                    <el-icon>
                        <Plus />
                    </el-icon>
                    添加规则
                </el-button>
            </div>
            <el-table :data="rules" v-loading="loadingRules" stripe size="small">
                <el-table-column prop="name" label="名称" width="150" />
                <el-table-column prop="topic_pattern" label="Topic 匹配" min-width="200">
                    <template #default="{ row }">
                        <code class="topic-pattern">{{ row.topic_pattern }}</code>
                    </template>
                </el-table-column>
                <el-table-column prop="action" label="动作" width="120">
                    <template #default="{ row }">
                        <el-tag size="small" :type="getActionType(row.action)">
                            {{ getActionText(row.action) }}
                        </el-tag>
                    </template>
                </el-table-column>
                <el-table-column label="启用" width="80" align="center">
                    <template #default="{ row }">
                        <el-tag :type="row.enabled ? 'success' : 'info'" size="small">
                            {{ row.enabled ? '是' : '否' }}
                        </el-tag>
                    </template>
                </el-table-column>
                <el-table-column label="操作" width="80">
                    <template #default="{ row }">
                        <el-button type="danger" size="small" link @click="handleDeleteRule(row)">
                            删除
                        </el-button>
                    </template>
                </el-table-column>
            </el-table>
        </div>

        <!-- 报文监控 -->
        <div class="section">
            <div class="section-header">
                <h4>
                    <el-icon>
                        <Document />
                    </el-icon>
                    报文监控
                </h4>
                <div class="header-actions">
                    <el-button size="small" @click="fetchPackets">
                        <el-icon>
                            <Refresh />
                        </el-icon>
                        刷新
                    </el-button>
                    <el-button size="small" type="danger" @click="handleClearPackets">
                        清空
                    </el-button>
                </div>
            </div>
            <el-table :data="packets" v-loading="loadingPackets" stripe size="small" max-height="300">
                <el-table-column prop="timestamp" label="时间" width="180">
                    <template #default="{ row }">
                        <span class="timestamp">{{ formatTime(row.timestamp) }}</span>
                    </template>
                </el-table-column>
                <el-table-column prop="direction" label="方向" width="80">
                    <template #default="{ row }">
                        <el-tag
                            :type="row.direction === 'received' ? 'success' : row.direction === 'sent' ? 'primary' : 'warning'"
                            size="small">
                            {{ getDirectionText(row.direction) }}
                        </el-tag>
                    </template>
                </el-table-column>
                <el-table-column prop="topic" label="Topic" min-width="200">
                    <template #default="{ row }">
                        <code v-if="row.topic" class="topic">{{ row.topic }}</code>
                        <span v-else class="no-data">-</span>
                    </template>
                </el-table-column>
                <el-table-column prop="payload" label="Payload" min-width="200">
                    <template #default="{ row }">
                        <el-tooltip v-if="row.payload" :content="row.payload" placement="top" :show-after="500">
                            <span class="payload-preview">{{ truncate(row.payload, 50) }}</span>
                        </el-tooltip>
                        <span v-else class="no-data">-</span>
                    </template>
                </el-table-column>
                <el-table-column prop="qos" label="QoS" width="60" align="center" />
            </el-table>
        </div>

        <!-- 添加规则对话框 -->
        <el-dialog v-model="showAddRuleDialog" title="添加规则" width="500px">
            <el-form :model="ruleForm" label-width="100px">
                <el-form-item label="规则名称" required>
                    <el-input v-model="ruleForm.name" placeholder="例如：温度响应规则" />
                </el-form-item>
                <el-form-item label="Topic 匹配" required>
                    <el-input v-model="ruleForm.topic_pattern" placeholder="例如：sensor/+/temperature 或 device/#" />
                    <div class="form-tip">支持 MQTT 通配符：+ 匹配单层，# 匹配多层</div>
                </el-form-item>
                <el-form-item label="动作类型" required>
                    <el-select v-model="ruleForm.actionType" style="width: 100%">
                        <el-option label="记录日志" value="log" />
                        <el-option label="发布响应" value="respond" />
                        <el-option label="静默" value="silence" />
                    </el-select>
                </el-form-item>
                <template v-if="ruleForm.actionType === 'respond'">
                    <el-form-item label="响应 Topic" required>
                        <el-input v-model="ruleForm.respondTopic" placeholder="例如：response/temperature" />
                    </el-form-item>
                    <el-form-item label="响应 Payload" required>
                        <el-input v-model="ruleForm.respondPayload" type="textarea" :rows="3" placeholder="响应内容" />
                    </el-form-item>
                </template>
                <template v-if="ruleForm.actionType === 'log'">
                    <el-form-item label="日志消息">
                        <el-input v-model="ruleForm.logMessage" placeholder="可选的日志消息" />
                    </el-form-item>
                </template>
            </el-form>
            <template #footer>
                <el-button @click="showAddRuleDialog = false">取消</el-button>
                <el-button type="primary" :loading="addingRule" @click="handleAddRule">添加</el-button>
            </template>
        </el-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue'
import { Plus, Setting, Document, Refresh } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
    getMqttRules,
    addMqttRule,
    removeMqttRule,
    getMqttPackets,
    clearMqttPackets,
    type MqttSimulatorInfo,
    type MqttSimulatorStatus,
    type MqttRule,
    type MqttRuleAction,
    type MqttPacketRecord,
} from '@/api/mqtt'

const props = defineProps<{
    simulator: MqttSimulatorInfo
}>()

const emit = defineEmits<{
    (e: 'refresh'): void
}>()

const rules = ref<MqttRule[]>([])
const packets = ref<MqttPacketRecord[]>([])
const loadingRules = ref(false)
const loadingPackets = ref(false)
const showAddRuleDialog = ref(false)
const addingRule = ref(false)

const ruleForm = reactive({
    name: '',
    topic_pattern: '',
    actionType: 'log' as 'log' | 'respond' | 'silence',
    respondTopic: '',
    respondPayload: '',
    logMessage: '',
})

onMounted(() => {
    fetchRules()
    fetchPackets()
})

watch(() => props.simulator.id, () => {
    fetchRules()
    fetchPackets()
})

async function fetchRules() {
    loadingRules.value = true
    try {
        rules.value = await getMqttRules(props.simulator.id)
    } catch (e) {
        console.error(e)
    } finally {
        loadingRules.value = false
    }
}

async function fetchPackets() {
    loadingPackets.value = true
    try {
        packets.value = await getMqttPackets(props.simulator.id, 100)
    } catch (e) {
        console.error(e)
    } finally {
        loadingPackets.value = false
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
    return '错误'
}

function getActionType(action: MqttRuleAction): string {
    if (action.type === 'respond') return 'primary'
    if (action.type === 'log') return 'info'
    if (action.type === 'silence') return 'warning'
    return ''
}

function getActionText(action: MqttRuleAction): string {
    switch (action.type) {
        case 'respond': return '响应'
        case 'log': return '日志'
        case 'silence': return '静默'
        case 'forward': return '转发'
        case 'transform': return '转换'
        default: return '未知'
    }
}

function getDirectionText(direction: string): string {
    switch (direction) {
        case 'received': return '接收'
        case 'sent': return '发送'
        case 'forwarded': return '转发'
        default: return direction
    }
}

function formatTime(timestamp: string): string {
    return new Date(timestamp).toLocaleString('zh-CN')
}

function truncate(str: string, len: number): string {
    return str.length > len ? str.slice(0, len) + '...' : str
}

async function handleAddRule() {
    if (!ruleForm.name || !ruleForm.topic_pattern) {
        ElMessage.warning('请填写必填项')
        return
    }

    let action: MqttRuleAction
    switch (ruleForm.actionType) {
        case 'respond':
            if (!ruleForm.respondTopic || !ruleForm.respondPayload) {
                ElMessage.warning('请填写响应配置')
                return
            }
            action = {
                type: 'respond',
                topic: ruleForm.respondTopic,
                payload: ruleForm.respondPayload,
                use_topic_vars: false,
            }
            break
        case 'log':
            action = { type: 'log', message: ruleForm.logMessage || undefined }
            break
        case 'silence':
        default:
            action = { type: 'silence' }
    }

    addingRule.value = true
    try {
        await addMqttRule(props.simulator.id, {
            name: ruleForm.name,
            topic_pattern: ruleForm.topic_pattern,
            action,
        })
        ElMessage.success('规则添加成功')
        showAddRuleDialog.value = false
        await fetchRules()
        // 重置表单
        ruleForm.name = ''
        ruleForm.topic_pattern = ''
        ruleForm.actionType = 'log'
    } catch (e: any) {
        ElMessage.error(e.message || '添加失败')
    } finally {
        addingRule.value = false
    }
}

async function handleDeleteRule(rule: MqttRule) {
    try {
        await ElMessageBox.confirm(`确定删除规则 "${rule.name}"？`, '确认')
        await removeMqttRule(props.simulator.id, rule.id)
        ElMessage.success('删除成功')
        await fetchRules()
    } catch (e: any) {
        if (e !== 'cancel') {
            ElMessage.error(e.message || '删除失败')
        }
    }
}

async function handleClearPackets() {
    try {
        await ElMessageBox.confirm('确定清空所有报文记录？', '确认')
        await clearMqttPackets(props.simulator.id)
        ElMessage.success('已清空')
        packets.value = []
    } catch (e: any) {
        if (e !== 'cancel') {
            ElMessage.error(e.message || '清空失败')
        }
    }
}
</script>

<style lang="scss" scoped>
.mqtt-simulator-detail {
    .info-section {
        margin-bottom: 20px;
    }

    .section {
        margin-bottom: 20px;

        .section-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 12px;

            h4 {
                margin: 0;
                font-size: 14px;
                font-weight: 600;
                display: flex;
                align-items: center;
                gap: 6px;
            }

            .header-actions {
                display: flex;
                gap: 8px;
            }
        }
    }

    .stat-highlight {
        font-size: 18px;
        font-weight: 700;
        color: var(--el-color-primary);
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

    .address-info {
        display: flex;
        flex-direction: column;
        gap: 4px;

        code {
            font-family: 'JetBrains Mono', monospace;
            font-size: 12px;
            background: var(--el-fill-color-light);
            padding: 2px 6px;
            border-radius: 4px;
            display: inline-block;
        }
    }

    .topic-pattern,
    .topic {
        font-family: 'JetBrains Mono', monospace;
        font-size: 12px;
        background: var(--el-fill-color-light);
        padding: 2px 6px;
        border-radius: 4px;
    }

    .timestamp {
        font-size: 12px;
        color: var(--el-text-color-secondary);
    }

    .payload-preview {
        font-size: 12px;
        cursor: pointer;
    }

    .no-data {
        color: var(--el-text-color-placeholder);
    }

    .form-tip {
        font-size: 12px;
        color: var(--el-text-color-secondary);
        margin-top: 4px;
    }
}
</style>
