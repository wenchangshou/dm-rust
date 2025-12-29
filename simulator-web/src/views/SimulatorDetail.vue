<template>
  <div class="simulator-detail" v-loading="store.loading">
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
            <StatusBadge :status="simulator.status" />
          </div>
        </div>
        <div class="header-right">
          <div class="action-group">
            <el-button type="primary" class="action-btn" @click="showSaveTemplateDialog = true">
              <el-icon>
                <Files />
              </el-icon>
              保存为模板
            </el-button>
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

      <!-- ... existing content ... -->
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
            <el-descriptions-item label="协议">
              <el-tag size="small" class="protocol-tag">{{ simulator.protocol }}</el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="监听地址">
              <code class="address-code">{{ simulator.bind_addr }}:{{ simulator.port }}</code>
            </el-descriptions-item>
            <el-descriptions-item label="状态">
              <StatusBadge :status="simulator.status" />
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
              <div class="mini-value">{{ simulator.state.stats.total_connections }}</div>
              <div class="mini-label">总连接数</div>
            </div>
            <div class="stats-mini-item">
              <div class="mini-value active">{{ simulator.state.stats.active_connections }}</div>
              <div class="mini-label">活动连接</div>
            </div>
            <div class="stats-mini-item">
              <div class="mini-value">{{ formatBytes(simulator.state.stats.bytes_received) }}</div>
              <div class="mini-label">接收字节</div>
            </div>
            <div class="stats-mini-item">
              <div class="mini-value">{{ formatBytes(simulator.state.stats.bytes_sent) }}</div>
              <div class="mini-label">发送字节</div>
            </div>
          </div>
          <div class="last-activity">
            <span class="label">最后活动：</span>
            <span class="value">{{ simulator.state.stats.last_activity || '无' }}</span>
          </div>
        </el-card>
      </div>

      <!-- 设备状态控制 -->
      <el-card class="control-card">
        <template #header>
          <div class="card-title">
            <el-icon>
              <Setting />
            </el-icon>
            <span>设备状态模拟</span>
          </div>
        </template>

        <div class="control-grid">
          <div class="control-item">
            <div class="control-label">在线状态</div>
            <el-switch v-model="isOnline" active-text="在线" inactive-text="离线" @change="handleOnlineChange" />
          </div>

          <div class="control-item">
            <div class="control-label">故障模拟</div>
            <div class="fault-control">
              <template v-if="simulator.state.fault">
                <el-tag type="danger" size="large" class="fault-tag">
                  {{ simulator.state.fault }}
                </el-tag>
                <el-button type="primary" size="small" @click="handleClearFault">
                  清除故障
                </el-button>
              </template>
              <template v-else>
                <el-select v-model="selectedFault" placeholder="选择故障类型" class="fault-select">
                  <el-option label="通信超时" value="timeout" />
                  <el-option label="协议错误" value="protocol_error" />
                  <el-option label="设备故障" value="device_fault" />
                  <el-option label="校验和错误" value="checksum_error" />
                </el-select>
                <el-button type="danger" :disabled="!selectedFault" @click="handleTriggerFault">
                  触发故障
                </el-button>
              </template>
            </div>
          </div>
        </div>
      </el-card>

      <!-- Modbus 模拟器专用面板 -->
      <ModbusSimulatorPanel v-if="isModbusProtocol" :simulator-id="id" class="modbus-panel" />

      <!-- 其他协议状态值 -->
      <el-card v-else class="state-card">
        <template #header>
          <div class="card-title">
            <el-icon>
              <List />
            </el-icon>
            <span>协议状态值</span>
          </div>
        </template>
        <el-table :data="stateValues">
          <el-table-column prop="key" label="键" width="200">
            <template #default="{ row }">
              <code class="key-code">{{ row.key }}</code>
            </template>
          </el-table-column>
          <el-table-column prop="value" label="值">
            <template #default="{ row }">
              <code class="value-code">{{ JSON.stringify(row.value) }}</code>
            </template>
          </el-table-column>
        </el-table>
        <el-empty v-if="stateValues.length === 0" description="暂无状态值" />
      </el-card>

      <!-- 报文监控（所有协议通用） -->
      <PacketMonitorPanel v-if="simulator.status === 'running'" :simulator-id="id" class="packet-panel" />

      <!-- 保存模板对话框 -->
      <el-dialog title="保存为模板" v-model="showSaveTemplateDialog" width="500px">
        <el-form label-width="80px">
          <el-form-item label="模板名称" required>
            <el-input v-model="templateForm.name" placeholder="请输入模板名称" />
          </el-form-item>
          <el-form-item label="描述">
            <el-input v-model="templateForm.description" type="textarea" placeholder="请输入模板描述" :rows="3" />
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="showSaveTemplateDialog = false">取消</el-button>
          <el-button type="primary" :loading="savingTemplate" @click="handleSaveTemplate">
            保存
          </el-button>
        </template>
      </el-dialog>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, VideoPlay, VideoPause, Refresh, InfoFilled, DataLine, Setting, List, Files } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useSimulatorStore } from '@/stores/simulator'
import StatusBadge from '@/components/common/StatusBadge.vue'
import ModbusSimulatorPanel from '@/components/simulator/ModbusSimulatorPanel.vue'
import PacketMonitorPanel from '@/components/simulator/PacketMonitorPanel.vue'
import * as simulatorApi from '@/api/simulator'

const props = defineProps<{
  id: string
}>()

const router = useRouter()
const store = useSimulatorStore()

const simulator = computed(() => store.currentSimulator)
const isOnline = ref(true)
const selectedFault = ref('')

// 模板保存相关
const showSaveTemplateDialog = ref(false)
const savingTemplate = ref(false)
const templateForm = reactive({
  name: '',
  description: ''
})

const isModbusProtocol = computed(() => {
  const protocol = simulator.value?.protocol?.toLowerCase() || ''
  return protocol.includes('modbus')
})

const stateValues = computed(() => {
  if (!simulator.value) return []
  return Object.entries(simulator.value.state.values).map(([key, value]) => ({
    key,
    value,
  }))
})

watch(simulator, (val) => {
  if (val) {
    isOnline.value = val.state.online
    // 预填模板名称
    if (!templateForm.name) {
      templateForm.name = `${val.name}_模板`
    }
  }
})

onMounted(() => {
  refresh()
})

function refresh() {
  store.fetchSimulator(props.id)
}

async function handleStart() {
  try {
    await store.startSimulator(props.id)
    ElMessage.success('模拟器已启动')
  } catch {
    // 错误已处理
  }
}

async function handleStop() {
  try {
    await store.stopSimulator(props.id)
    ElMessage.success('模拟器已停止')
  } catch {
    // 错误已处理
  }
}

async function handleOnlineChange(online: boolean) {
  try {
    await store.setOnline(props.id, online)
    ElMessage.success(online ? '设备已上线' : '设备已下线')
  } catch {
    isOnline.value = !online
  }
}

async function handleTriggerFault() {
  if (!selectedFault.value) return
  try {
    await store.triggerFault(props.id, selectedFault.value)
    ElMessage.success('故障已触发')
    selectedFault.value = ''
  } catch {
    // 错误已处理
  }
}

async function handleClearFault() {
  try {
    await store.clearFault(props.id)
    ElMessage.success('故障已清除')
  } catch {
    // 错误已处理
  }
}

async function handleSaveTemplate() {
  if (!templateForm.name) {
    ElMessage.warning('请输入模板名称')
    return
  }

  savingTemplate.value = true
  try {
    await simulatorApi.saveAsTemplate(props.id, {
      name: templateForm.name,
      description: templateForm.description
    })
    ElMessage.success('模板保存成功')
    showSaveTemplateDialog.value = false
  } catch (e: any) {
    ElMessage.error(e.message || '模板保存失败')
  } finally {
    savingTemplate.value = false
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}
</script>
<style lang="scss" scoped>
.simulator-detail {
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
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
    background: linear-gradient(135deg, var(--text-primary) 0%, var(--text-secondary) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    margin: 0;
  }
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

// 信息卡片网格
.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
  margin-bottom: 20px;
}

.info-card,
.control-card,
.state-card {
  border-radius: 16px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: var(--text-primary);

  .el-icon {
    color: #667eea;
  }
}

.id-code,
.address-code,
.key-code,
.value-code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  background: var(--bg-input);
  padding: 4px 8px;
  border-radius: 6px;
  color: var(--text-secondary);
}

.protocol-tag {
  background: rgba(102, 126, 234, 0.15);
  border-color: rgba(102, 126, 234, 0.3);
  color: #667eea;
  font-weight: 500;
}

// 迷你统计
.stats-mini-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 16px;
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

  &.active {
    color: var(--success);
  }
}

.mini-label {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
}

.last-activity {
  padding: 12px;
  background: var(--bg-input);
  border-radius: 10px;
  font-size: 13px;

  .label {
    color: var(--text-muted);
  }

  .value {
    color: var(--text-primary);
    font-family: 'JetBrains Mono', monospace;
  }
}

// 控制面板
.control-card {
  margin-bottom: 20px;
}

.control-grid {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.control-item {
  display: flex;
  align-items: center;
  gap: 16px;
}

.control-label {
  min-width: 100px;
  font-weight: 500;
  color: var(--text-secondary);
}

.fault-control {
  display: flex;
  align-items: center;
  gap: 12px;
}

.fault-tag {
  font-size: 14px;
}

.fault-select {
  width: 180px;
}

// 面板
.modbus-panel,
.state-card,
.packet-panel {
  margin-top: 24px;
}

// 响应式
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

  .stats-mini-grid {
    grid-template-columns: 1fr;
  }

  .control-item {
    flex-direction: column;
    align-items: flex-start;
  }

  .fault-control {
    flex-wrap: wrap;
  }
}
</style>
