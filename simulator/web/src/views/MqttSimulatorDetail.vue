<template>
  <div class="mqtt-simulator-detail" v-loading="loading">
    <template v-if="simulator">
      <!-- 页面头部 -->
      <div class="page-header">
        <div class="header-left">
          <el-button class="back-btn" @click="router.back()">
            <el-icon>
              <ArrowLeft />
            </el-icon>
            <span>返回</span>
          </el-button>
          <div class="title-group">
            <h2>{{ simulator.name }}</h2>
            <el-tag :type="getStatusType(simulator.status)" effect="dark">
              {{ getStatusText(simulator.status) }}
            </el-tag>
          </div>
        </div>
        <div class="header-right">
          <div class="action-group">
            <el-button v-if="simulator.status === 'stopped'" type="success" class="action-btn" @click="handleStart">
              <el-icon>
                <VideoPlay />
              </el-icon>
              启动
            </el-button>
            <el-button v-else type="warning" class="action-btn" @click="handleStop">
              <el-icon>
                <VideoPause />
              </el-icon>
              停止
            </el-button>
            <el-button class="refresh-action-btn" @click="refresh">
              <el-icon>
                <Refresh />
              </el-icon>
              刷新
            </el-button>
          </div>
        </div>
      </div>

      <div class="info-grid">
        <!-- 基本信息 -->
        <el-card class="info-card">
          <template #header>
            <div class="card-title">
              <el-icon>
                <InfoFilled />
              </el-icon>
              <span>基本信息</span>
            </div>
          </template>
          <el-descriptions :column="1" border>
            <el-descriptions-item label="ID">
              <code class="id-code">{{ simulator.id }}</code>
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
                  <div class="address-row">v4: <code class="address-code">{{ simulator.bind_addr }}:{{ simulator.port }}</code></div>
                  <div class="address-row">v5: <code class="address-code">{{ simulator.bind_addr }}:{{ simulator.port + 1 }}</code></div>
                </template>
                <template v-else>
                  <code class="address-code">{{ simulator.bind_addr }}:{{ simulator.port }}</code>
                </template>
              </div>
            </el-descriptions-item>
          </el-descriptions>
        </el-card>

        <!-- 连接统计 -->
        <el-card class="info-card">
          <template #header>
            <div class="card-title">
              <el-icon>
                <DataLine />
              </el-icon>
              <span>连接统计</span>
            </div>
          </template>
          <div class="stats-mini-grid">
            <div class="stats-mini-item">
              <div class="mini-value">{{ simulator.state?.stats?.active_connections || 0 }}</div>
              <div class="mini-label">活动连接</div>
            </div>
            <div class="stats-mini-item">
              <div class="mini-value">{{ simulator.state?.stats?.messages_received || 0 }}</div>
              <div class="mini-label">消息接收</div>
            </div>
             <div class="stats-mini-item">
              <div class="mini-value">{{ simulator.state?.stats?.messages_sent || 0 }}</div>
              <div class="mini-label">消息发送</div>
            </div>
             <div class="stats-mini-item">
              <div class="mini-value">{{ simulator.state?.stats?.total_connections || 0 }}</div>
              <div class="mini-label">总连接数</div>
            </div>
          </div>
        </el-card>
      </div>

      <!-- 规则管理 -->
      <el-card class="section-card rule-panel">
        <template #header>
          <div class="card-header-flex">
            <div class="card-title">
              <el-icon>
                <Setting />
              </el-icon>
              <span>规则管理</span>
            </div>
            <el-button type="primary" size="small" @click="showAddRuleDialog = true">
              <el-icon>
                <Plus />
              </el-icon>
              添加规则
            </el-button>
          </div>
        </template>
        <el-table :data="rules" v-loading="loadingRules" stripe>
          <el-table-column prop="name" label="名称" width="180" />
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
            <el-table-column label="操作" width="100" align="center">
                <template #default="{ row }">
                    <el-button type="danger" size="small" link @click="handleDeleteRule(row)">
                        删除
                    </el-button>
                </template>
            </el-table-column>
        </el-table>
      </el-card>

      <!-- 报文监控 -->
      <el-card class="section-card packet-panel">
        <template #header>
            <div class="card-header-flex">
                <div class="card-title">
                    <el-icon>
                        <Document />
                    </el-icon>
                    <span>报文监控</span>
                </div>
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
        </template>
        <el-table :data="packets" v-loading="loadingPackets" stripe size="small" max-height="500">
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
      </el-card>

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
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, VideoPlay, VideoPause, Refresh, InfoFilled, DataLine, Setting, Document, Plus } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  getMqttSimulator,
  startMqttSimulator,
  stopMqttSimulator,
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
  id: string
}>()

const router = useRouter()
const simulator = ref<MqttSimulatorInfo | null>(null)
const loading = ref(false)
const refreshTimer = ref<number | null>(null)

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
  refresh()
  // Auto refresh packets every 5 seconds if running
  refreshTimer.value = window.setInterval(() => {
    if (simulator.value?.status === 'running') {
      fetchPackets()
      fetchSimulator(false) // silent refresh
    }
  }, 5000)
})

onUnmounted(() => {
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
  }
})

async function refresh() {
  await fetchSimulator(true)
  if (simulator.value) {
    fetchRules()
    fetchPackets()
  }
}

async function fetchSimulator(showLoading = false) {
  if (showLoading) loading.value = true
  try {
    simulator.value = await getMqttSimulator(props.id)
  } catch (e: any) {
    ElMessage.error(e.message || '获取模拟器信息失败')
  } finally {
    if (showLoading) loading.value = false
  }
}

async function handleStart() {
  try {
    await startMqttSimulator(props.id)
    ElMessage.success('模拟器已启动')
    refresh()
  } catch (e: any) {
    ElMessage.error(e.message || '启动失败')
  }
}

async function handleStop() {
  try {
    await stopMqttSimulator(props.id)
    ElMessage.success('模拟器已停止')
    refresh()
  } catch (e: any) {
    ElMessage.error(e.message || '停止失败')
  }
}

// Rules Logic
async function fetchRules() {
    loadingRules.value = true
    try {
        rules.value = await getMqttRules(props.id)
    } catch (e) {
        console.error(e)
    } finally {
        loadingRules.value = false
    }
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
        await addMqttRule(props.id, {
            name: ruleForm.name,
            topic_pattern: ruleForm.topic_pattern,
            action,
        })
        ElMessage.success('规则添加成功')
        showAddRuleDialog.value = false
        await fetchRules()
        // Reset form
        ruleForm.name = ''
        ruleForm.topic_pattern = ''
        ruleForm.actionType = 'log'
        ruleForm.respondTopic = ''
        ruleForm.respondPayload = ''
        ruleForm.logMessage = ''
    } catch (e: any) {
        ElMessage.error(e.message || '添加失败')
    } finally {
        addingRule.value = false
    }
}

async function handleDeleteRule(rule: MqttRule) {
    try {
        await ElMessageBox.confirm(`确定删除规则 "${rule.name}"？`, '确认')
        await removeMqttRule(props.id, rule.id)
        ElMessage.success('删除成功')
        await fetchRules()
    } catch (e: any) {
        if (e !== 'cancel') {
            ElMessage.error(e.message || '删除失败')
        }
    }
}

// Packets Logic
async function fetchPackets() {
    loadingPackets.value = true
    try {
        packets.value = await getMqttPackets(props.id, 100)
    } catch (e) {
        console.error(e)
    } finally {
        loadingPackets.value = false
    }
}

async function handleClearPackets() {
    try {
        await ElMessageBox.confirm('确定清空所有报文记录？', '确认')
        await clearMqttPackets(props.id)
        ElMessage.success('已清空')
        packets.value = []
    } catch (e: any) {
        if (e !== 'cancel') {
            ElMessage.error(e.message || '清空失败')
        }
    }
}

// Helpers
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
</script>

<style lang="scss" scoped>
.mqtt-simulator-detail {
  animation: fadeIn 0.5s ease;
  padding: 20px;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 28px;
  flex-wrap: wrap;
  gap: 16px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.back-btn {
  background: var(--bg-input);
  border: 1px solid var(--border-glass);
  color: var(--text-secondary);
  padding: 10px 16px;
  border-radius: 10px;
  transition: all 0.3s ease;

  &:hover {
    background: var(--bg-hover);
    border-color: var(--primary-start);
    color: var(--text-primary);
    transform: translateX(-4px);
  }
}

.title-group {
  display: flex;
  align-items: center;
  gap: 12px;

  h2 {
    font-size: 24px;
    font-weight: 700;
    margin: 0;
    background: linear-gradient(135deg, var(--text-primary) 0%, var(--text-secondary) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
}

.header-right {
  display: flex;
  align-items: center;
}

.action-group {
  display: flex;
  gap: 10px;
}

.action-btn {
  padding: 10px 20px;
  font-weight: 600;
}

.refresh-action-btn {
  background: var(--bg-input);
  border: 1px solid var(--border-glass);
  color: var(--text-secondary);
  padding: 10px 16px;
  &:hover {
    background: var(--bg-hover);
    border-color: var(--primary-start);
    color: var(--text-primary);
  }
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
  margin-bottom: 20px;
}

.info-card, .section-card {
  border-radius: 16px;
  margin-bottom: 20px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: var(--text-primary);
  .el-icon { color: #667eea; }
}

.card-header-flex {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.id-code, .address-code, .topic-pattern, .topic {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  background: var(--bg-input);
  padding: 4px 8px;
  border-radius: 6px;
  color: var(--text-secondary);
}

.address-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.address-row {
    font-size: 13px;
    color: var(--el-text-color-regular);
}

.version-tags {
    display: flex;
    gap: 4px;
}

.stats-mini-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.stats-mini-item {
  text-align: center;
  padding: 12px;
  background: var(--bg-input);
  border-radius: 10px;
  border: 1px solid var(--border-glass);
}

.mini-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  font-family: 'JetBrains Mono', monospace;
}

.mini-label {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
}

.header-actions {
    display: flex;
    gap: 8px;
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

@media (max-width: 1024px) {
  .info-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .page-header {
    flex-direction: column;
    align-items: flex-start;
  }
  .header-left {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
