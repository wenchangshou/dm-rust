<template>
  <el-card class="packet-monitor-panel">
    <template #header>
      <div class="panel-header">
        <div class="header-left">
          <span class="panel-title">报文监控</span>
          <el-tag :type="monitorEnabled ? 'success' : 'info'" size="small">
            {{ monitorEnabled ? '监控中' : '已暂停' }}
          </el-tag>
          <el-tag type="info" size="small" class="count-tag">
            {{ packets.length }} 条记录
          </el-tag>
          <el-tag v-if="debugMode" type="warning" size="small" class="debug-tag">
            <el-icon>
              <Aim />
            </el-icon>
            Debug
          </el-tag>
        </div>
        <div class="header-right">
          <el-tooltip content="Debug 模式：持久化所有报文到文件" placement="top">
            <el-switch v-model="debugMode" active-text="Debug" inactive-text="" @change="handleDebugToggle"
              :loading="debugLoading" class="debug-switch" />
          </el-tooltip>
          <el-button v-if="debugMode" size="small" type="success" @click="handleDownloadLog">
            <el-icon>
              <Download />
            </el-icon>
            下载日志
          </el-button>
          <el-divider direction="vertical" />
          <el-switch v-model="monitorEnabled" active-text="监控" inactive-text="暂停" @change="handleMonitorToggle" />
          <el-button size="small" @click="handleRefresh" :loading="loading">
            <el-icon>
              <Refresh />
            </el-icon>
            刷新
          </el-button>
          <el-button size="small" type="danger" @click="handleClear" :disabled="packets.length === 0">
            <el-icon>
              <Delete />
            </el-icon>
            清空
          </el-button>
          <el-checkbox v-model="autoScroll" size="small">自动滚动</el-checkbox>
        </div>
      </div>
    </template>

    <!-- 报文列表 -->
    <div class="packet-list" ref="packetListRef">
      <div v-for="packet in packets" :key="packet.id" :class="['packet-item', `packet-${packet.direction}`]">
        <div class="packet-header">
          <span class="packet-direction">
            <el-icon v-if="packet.direction === 'received'">
              <Download />
            </el-icon>
            <el-icon v-else>
              <Upload />
            </el-icon>
            {{ packet.direction === 'received' ? 'RX' : 'TX' }}
          </span>
          <span class="packet-time">{{ formatTime(packet.timestamp) }}</span>
          <span class="packet-addr">{{ packet.peer_addr }}</span>
          <span class="packet-size">{{ packet.size }} bytes</span>
        </div>
        <div class="packet-data">
          <code>{{ formatHex(packet.hex_data) }}</code>
        </div>
      </div>

      <el-empty v-if="packets.length === 0" description="暂无报文记录" />
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { Refresh, Delete, Download, Upload, Aim } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import * as simulatorApi from '@/api/simulator'
import type { PacketRecord } from '@/types/simulator'

interface Props {
  simulatorId: string
  enabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  enabled: true,
})

const loading = ref(false)
const packets = ref<PacketRecord[]>([])
const monitorEnabled = ref(props.enabled)
const autoScroll = ref(true)
const packetListRef = ref<HTMLElement | null>(null)
const lastPacketId = ref(0)
let pollTimer: ReturnType<typeof setInterval> | null = null

// Debug 模式
const debugMode = ref(false)
const debugLoading = ref(false)

// 格式化时间
function formatTime(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3,
  })
}

// 格式化十六进制数据（每两个字符加空格）
function formatHex(hex: string): string {
  return hex.toUpperCase().replace(/(.{2})/g, '$1 ').trim()
}

// 加载报文
async function loadPackets(incremental = false) {
  loading.value = true
  try {
    const afterId = incremental ? lastPacketId.value : undefined
    const response = await simulatorApi.getPackets(props.simulatorId, afterId)

    if (incremental && response.packets.length > 0) {
      // 增量添加
      packets.value.push(...response.packets)
    } else if (!incremental) {
      // 全量加载
      packets.value = response.packets
    }

    // 更新最后的 packet ID
    if (packets.value.length > 0) {
      lastPacketId.value = packets.value[packets.value.length - 1].id
    }

    // 自动滚动到底部
    if (autoScroll.value && response.packets.length > 0) {
      nextTick(() => {
        if (packetListRef.value) {
          packetListRef.value.scrollTop = packetListRef.value.scrollHeight
        }
      })
    }
  } catch {
    // 错误已处理
  } finally {
    loading.value = false
  }
}

// 刷新
function handleRefresh() {
  loadPackets(false)
}

// 清空
async function handleClear() {
  try {
    await ElMessageBox.confirm('确定要清空所有报文记录吗？', '确认', { type: 'warning' })
    await simulatorApi.clearPackets(props.simulatorId)
    packets.value = []
    lastPacketId.value = 0
    ElMessage.success('报文已清空')
  } catch {
    // 取消或错误
  }
}

// 切换监控状态
async function handleMonitorToggle(enabled: boolean) {
  try {
    await simulatorApi.setPacketMonitorSettings(props.simulatorId, { enabled })
    ElMessage.success(enabled ? '监控已开启' : '监控已暂停')
  } catch {
    monitorEnabled.value = !enabled
  }
}

// 切换 Debug 模式
async function handleDebugToggle(enabled: boolean) {
  debugLoading.value = true
  try {
    await simulatorApi.setDebugMode(props.simulatorId, enabled)
    ElMessage.success(enabled ? 'Debug 模式已开启，报文将持久化到文件' : 'Debug 模式已关闭')
  } catch {
    debugMode.value = !enabled
  } finally {
    debugLoading.value = false
  }
}

// 下载 Debug 日志
function handleDownloadLog() {
  const url = simulatorApi.getDebugLogUrl(props.simulatorId)
  window.open(url, '_blank')
}

// 开始轮询
function startPolling() {
  if (pollTimer) return
  pollTimer = setInterval(() => {
    if (monitorEnabled.value) {
      loadPackets(true)
    }
  }, 1000)
}

// 停止轮询
function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

onMounted(() => {
  loadPackets(false)
  startPolling()
})

onUnmounted(() => {
  stopPolling()
})

// 监控 simulatorId 变化
watch(() => props.simulatorId, () => {
  packets.value = []
  lastPacketId.value = 0
  loadPackets(false)
})
</script>

<style lang="scss" scoped>
.packet-monitor-panel {
  border-radius: 16px;

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;

    .header-left {
      display: flex;
      align-items: center;
      gap: 10px;
    }

    .panel-title {
      font-weight: 600;
    }

    .count-tag {
      font-family: 'JetBrains Mono', monospace;
    }

    .debug-tag {
      display: flex;
      align-items: center;
      gap: 4px;
    }

    .debug-switch {
      margin-right: 4px;
    }

    .header-right {
      display: flex;
      align-items: center;
      gap: 10px;

      .el-divider--vertical {
        height: 20px;
      }
    }
  }

  .packet-list {
    height: 400px;
    overflow-y: auto;
    background: #1a1a2e;
    border-radius: 10px;
    padding: 12px;
    font-family: 'JetBrains Mono', 'Consolas', 'Monaco', monospace;
    font-size: 12px;
  }

  .packet-item {
    margin-bottom: 8px;
    padding: 10px 12px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    transition: all 0.2s ease;

    &:hover {
      background: rgba(255, 255, 255, 0.05);
    }

    &.packet-received {
      border-left: 3px solid var(--success);

      .packet-direction {
        color: var(--success);
      }
    }

    &.packet-sent {
      border-left: 3px solid var(--info);

      .packet-direction {
        color: var(--info);
      }
    }

    .packet-header {
      display: flex;
      align-items: center;
      gap: 15px;
      margin-bottom: 6px;
      color: rgba(255, 255, 255, 0.5);
      font-size: 11px;

      .packet-direction {
        display: flex;
        align-items: center;
        gap: 4px;
        font-weight: bold;
      }

      .packet-size {
        margin-left: auto;
      }
    }

    .packet-data {
      code {
        color: #feca57;
        word-break: break-all;
        line-height: 1.6;
      }
    }
  }
}

// 浅色主题下保持深色终端风格
[data-theme="light"] .packet-monitor-panel {
  .packet-list {
    background: #1e293b;
  }
}
</style>
