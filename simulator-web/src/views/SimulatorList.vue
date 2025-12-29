<template>
  <div class="simulator-list">
    <!-- 页面头部 -->
    <div class="page-header">
      <h2>模拟器管理</h2>
      <el-button class="create-btn" @click="showCreateDialog = true">
        <el-icon>
          <Plus />
        </el-icon>
        创建模拟器
      </el-button>
    </div>

    <el-tabs v-model="activeTab" class="main-tabs">
      <el-tab-pane label="模拟器列表" name="simulators">
        <!-- 统计卡片 -->
        <div class="stats-grid">
          <div class="stat-card-wrapper">
            <div class="stat-card">
              <div class="stat-icon total">
                <el-icon :size="24">
                  <Grid />
                </el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-value">{{ store.simulators.length }}</div>
                <div class="stat-label">模拟器总数</div>
              </div>
            </div>
          </div>

          <div class="stat-card-wrapper">
            <div class="stat-card">
              <div class="stat-icon running">
                <el-icon :size="24">
                  <VideoPlay />
                </el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-value running">{{ store.runningCount }}</div>
                <div class="stat-label">运行中</div>
              </div>
            </div>
          </div>

          <div class="stat-card-wrapper">
            <div class="stat-card">
              <div class="stat-icon stopped">
                <el-icon :size="24">
                  <VideoPause />
                </el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-value stopped">{{ store.stoppedCount }}</div>
                <div class="stat-label">已停止</div>
              </div>
            </div>
          </div>

          <div class="stat-card-wrapper">
            <div class="stat-card">
              <div class="stat-icon connections">
                <el-icon :size="24">
                  <Connection />
                </el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-value connections">{{ store.totalConnections }}</div>
                <div class="stat-label">活动连接</div>
              </div>
            </div>
          </div>
        </div>

        <!-- 模拟器表格 -->
        <el-card class="table-card">
          <template #header>
            <div class="table-header">
              <span class="header-title">模拟器列表</span>
              <span class="header-count">共 {{ store.simulators.length }} 个</span>
            </div>
          </template>
          <el-table :data="store.simulators" v-loading="store.loading">
            <el-table-column prop="name" label="名称" min-width="150">
              <template #default="{ row }">
                <router-link :to="`/simulator/${row.id}`" class="simulator-link">
                  <span class="link-text">{{ row.name }}</span>
                  <el-icon class="link-arrow">
                    <ArrowRight />
                  </el-icon>
                </router-link>
              </template>
            </el-table-column>
            <el-table-column prop="protocol" label="协议" width="120">
              <template #default="{ row }">
                <el-tag size="small" class="protocol-tag">{{ row.protocol }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="地址" width="180">
              <template #default="{ row }">
                <code class="address-code">{{ row.bind_addr }}:{{ row.port }}</code>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="100">
              <template #default="{ row }">
                <StatusBadge :status="row.status" />
              </template>
            </el-table-column>
            <el-table-column label="设备状态" width="140">
              <template #default="{ row }">
                <div class="device-status">
                  <span :class="['status-dot', row.state.online ? 'online' : 'offline']"></span>
                  <span class="status-text">{{ row.state.online ? '在线' : '离线' }}</span>
                  <el-tag v-if="row.state.fault" type="danger" size="small" class="fault-tag">
                    故障
                  </el-tag>
                </div>
              </template>
            </el-table-column>
            <el-table-column label="连接" width="100" align="center">
              <template #default="{ row }">
                <el-badge v-if="row.state.stats.active_connections > 0" :value="row.state.stats.active_connections"
                  class="connection-badge clickable" @click="handleShowConnections(row)">
                  <el-button size="small" type="primary" plain class="connection-btn">
                    <el-icon>
                      <Connection />
                    </el-icon>
                    查看
                  </el-button>
                </el-badge>
                <span v-else class="connection-count zero">
                  <el-icon>
                    <Connection />
                  </el-icon>
                </span>
              </template>
            </el-table-column>
            <el-table-column label="Debug" width="140" align="center">
              <template #default="{ row }">
                <div class="debug-actions" v-if="row.status === 'running'">
                  <el-switch :model-value="row.state?.packet_monitor?.debug_mode || false" size="small"
                    @change="(val: boolean) => handleToggleDebug(row, val)" />
                  <el-button v-if="row.state?.packet_monitor?.debug_mode" type="success" size="small" link
                    @click="handleDownloadLog(row)">
                    <el-icon>
                      <Download />
                    </el-icon>
                    下载
                  </el-button>
                </div>
                <span v-else class="debug-disabled">-</span>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="160">
              <template #default="{ row }">
                <div class="action-buttons">
                  <el-button v-if="row.status === 'stopped'" type="success" size="small" class="action-btn"
                    @click="handleStart(row)">
                    <el-icon>
                      <VideoPlay />
                    </el-icon>
                    启动
                  </el-button>
                  <el-button v-else type="warning" size="small" class="action-btn" @click="handleStop(row)">
                    <el-icon>
                      <VideoPause />
                    </el-icon>
                    停止
                  </el-button>
                  <el-button type="danger" size="small" class="action-btn" @click="handleDelete(row)">
                    <el-icon>
                      <Delete />
                    </el-icon>
                  </el-button>
                </div>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-tab-pane>

      <el-tab-pane label="模板列表" name="templates">
        <TemplateList ref="templateListRef" />
      </el-tab-pane>
    </el-tabs>

    <!-- 创建对话框 -->
    <CreateSimulatorDialog v-model:visible="showCreateDialog" :protocols="store.protocols" @created="handleCreated" />

    <!-- 客户端连接对话框 -->
    <el-dialog v-model="showConnectionsDialog" :title="`客户端连接 - ${selectedSimulator?.name || ''}`" width="600px"
      :close-on-click-modal="true" class="connections-dialog">
      <ClientConnectionsPanel v-if="selectedSimulator" :simulator-id="selectedSimulator.id" />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, VideoPlay, VideoPause, Delete, Grid, Connection, ArrowRight, Download } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useSimulatorStore } from '@/stores/simulator'
import type { SimulatorInfo } from '@/types/simulator'
import StatusBadge from '@/components/common/StatusBadge.vue'
import CreateSimulatorDialog from '@/components/simulator/CreateSimulatorDialog.vue'
import ClientConnectionsPanel from '@/components/simulator/ClientConnectionsPanel.vue'
import TemplateList from '@/components/simulator/TemplateList.vue'
import * as simulatorApi from '@/api/simulator'

const store = useSimulatorStore()
const showCreateDialog = ref(false)
const showConnectionsDialog = ref(false)
const selectedSimulator = ref<SimulatorInfo | null>(null)
const activeTab = ref('simulators')
const templateListRef = ref<InstanceType<typeof TemplateList>>()

onMounted(() => {
  store.fetchSimulators()
  store.fetchProtocols()
})

async function handleStart(simulator: SimulatorInfo) {
  try {
    await store.startSimulator(simulator.id)
    ElMessage.success('模拟器已启动')
  } catch {
    // 错误已在拦截器中处理
  }
}

async function handleStop(simulator: SimulatorInfo) {
  try {
    await store.stopSimulator(simulator.id)
    ElMessage.success('模拟器已停止')
  } catch {
    // 错误已在拦截器中处理
  }
}

async function handleDelete(simulator: SimulatorInfo) {
  try {
    await ElMessageBox.confirm(
      `确定要删除模拟器 "${simulator.name}" 吗？`,
      '确认删除',
      { type: 'warning' }
    )
    await store.deleteSimulator(simulator.id)
    ElMessage.success('模拟器已删除')
  } catch {
    // 用户取消或错误
  }
}

function handleCreated() {
  showCreateDialog.value = false
  ElMessage.success('模拟器创建成功')
}

function handleShowConnections(simulator: SimulatorInfo) {
  selectedSimulator.value = simulator
  showConnectionsDialog.value = true
}

// Debug 模式控制
async function handleToggleDebug(simulator: SimulatorInfo, enabled: boolean) {
  try {
    await simulatorApi.setDebugMode(simulator.id, enabled)
    ElMessage.success(enabled ? 'Debug 模式已开启' : 'Debug 模式已关闭')
    // 刷新列表以更新状态
    await store.fetchSimulators()
  } catch {
    // 错误已在拦截器中处理
  }
}

function handleDownloadLog(simulator: SimulatorInfo) {
  const url = simulatorApi.getDebugLogUrl(simulator.id)
  window.open(url, '_blank')
}
</script>

<style lang="scss" scoped>
.simulator-list {
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

  h2 {
    font-size: 28px;
    font-weight: 700;
    background: linear-gradient(135deg, var(--text-primary) 0%, var(--text-secondary) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
}

.create-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  padding: 12px 24px;
  font-size: 14px;
  font-weight: 600;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
  transition: all 0.3s ease;

  &:hover {
    transform: translateY(-3px);
    box-shadow: 0 8px 25px rgba(102, 126, 234, 0.5);
  }
}

// 统计卡片网格
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 20px;
  margin-bottom: 28px;
}

.stat-card-wrapper {
  .stat-card {
    background: var(--bg-card);
    backdrop-filter: blur(20px);
    border: 1px solid var(--border-glass);
    border-radius: 16px;
    padding: 20px;
    display: flex;
    align-items: center;
    gap: 16px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;

    &::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      height: 2px;
      background: linear-gradient(90deg, var(--primary-start), var(--primary-end));
      opacity: 0;
      transition: opacity 0.3s ease;
    }

    &:hover {
      transform: translateY(-4px);
      border-color: var(--border-glass-hover);
      box-shadow: var(--shadow-glass), var(--shadow-glow);

      &::before {
        opacity: 1;
      }
    }
  }
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;

  &.total {
    background: linear-gradient(135deg, #667eea, #764ba2);
  }

  &.running {
    background: linear-gradient(135deg, var(--success), #4ade80);
  }

  &.stopped {
    background: linear-gradient(135deg, #64748b, #94a3b8);
  }

  &.connections {
    background: linear-gradient(135deg, var(--info), #0984e3);
  }
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  background: linear-gradient(135deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  line-height: 1.2;

  &.running {
    background: linear-gradient(135deg, var(--success), #4ade80);
    -webkit-background-clip: text;
    background-clip: text;
  }

  &.stopped {
    background: linear-gradient(135deg, #94a3b8, #64748b);
    -webkit-background-clip: text;
    background-clip: text;
  }

  &.connections {
    background: linear-gradient(135deg, var(--info), #0984e3);
    -webkit-background-clip: text;
    background-clip: text;
  }
}

.stat-label {
  font-size: 13px;
  color: var(--text-muted);
  margin-top: 4px;
  font-weight: 500;
}

// 表格区域
.table-card {
  border-radius: 16px;

  .table-header {
    display: flex;
    align-items: center;
    gap: 12px;

    .header-title {
      font-weight: 600;
    }

    .header-count {
      font-size: 13px;
      color: var(--text-muted);
      background: var(--bg-input);
      padding: 4px 10px;
      border-radius: 20px;
    }
  }
}

.simulator-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  text-decoration: none;
  color: inherit;
  transition: all 0.3s ease;

  .link-text {
    font-weight: 500;
    background: linear-gradient(135deg, #667eea, #764ba2);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .link-arrow {
    font-size: 14px;
    color: #667eea;
    opacity: 0;
    transform: translateX(-4px);
    transition: all 0.3s ease;
  }

  &:hover {
    .link-arrow {
      opacity: 1;
      transform: translateX(0);
    }
  }
}

.protocol-tag {
  background: rgba(102, 126, 234, 0.15);
  border-color: rgba(102, 126, 234, 0.3);
  color: #667eea;
  font-weight: 500;
}

.address-code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  background: var(--bg-input);
  padding: 4px 8px;
  border-radius: 6px;
  color: var(--text-secondary);
}

.device-status {
  display: flex;
  align-items: center;
  gap: 8px;

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;

    &.online {
      background: var(--success);
      box-shadow: 0 0 10px rgba(var(--success-rgb), 0.5);
      animation: pulse 2s ease-in-out infinite;
    }

    &.offline {
      background: #64748b;
    }
  }

  .status-text {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .fault-tag {
    margin-left: 4px;
  }
}

@keyframes pulse {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.5;
  }
}

.connection-count {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  font-family: 'JetBrains Mono', monospace;
  font-weight: 600;
  color: var(--info);

  &.zero {
    color: var(--text-muted);
    opacity: 0.5;
  }
}

.connection-badge {
  &.clickable {
    cursor: pointer;

    .connection-btn {
      transition: all 0.2s;
    }

    &:hover .connection-btn {
      transform: scale(1.05);
    }
  }
}

.connection-btn {
  font-size: 12px;

  .el-icon {
    margin-right: 4px;
  }
}

.action-buttons {
  display: flex;
  gap: 8px;

  .action-btn {
    padding: 6px 12px;
    font-size: 12px;
  }
}

// 对话框样式
:deep(.connections-dialog) {
  .el-dialog__body {
    padding: 0;
  }
}

// Debug 控制列
.debug-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.debug-disabled {
  color: var(--text-muted);
  opacity: 0.5;
}

// 响应式
@media (max-width: 1200px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }

  .page-header {
    flex-direction: column;
    gap: 16px;
    align-items: flex-start;
  }
}
</style>
