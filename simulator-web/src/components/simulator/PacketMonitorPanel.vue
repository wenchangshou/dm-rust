<template>
  <el-card class="packet-monitor-panel">
    <template #header>
      <div class="panel-header">
        <div class="header-left">
          <span>报文监控</span>
          <el-tag :type="monitorEnabled ? 'success' : 'info'" size="small">
            {{ monitorEnabled ? '监控中' : '已暂停' }}
          </el-tag>
          <el-tag type="info" size="small">
            {{ packets.length }} 条记录
          </el-tag>
        </div>
        <div class="header-right">
          <el-switch
            v-model="monitorEnabled"
            active-text="监控"
            inactive-text="暂停"
            @change="handleMonitorToggle"
          />
          <el-button size="small" @click="handleRefresh" :loading="loading">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
          <el-button size="small" type="danger" @click="handleClear" :disabled="packets.length === 0">
            <el-icon><Delete /></el-icon>
            清空
          </el-button>
          <el-checkbox v-model="autoScroll" size="small">自动滚动</el-checkbox>
        </div>
      </div>
    </template>

    <!-- 报文列表 -->
    <div class="packet-list" ref="packetListRef">
      <div
        v-for="packet in packets"
        :key="packet.id"
        :class="['packet-item', `packet-${packet.direction}`]"
      >
        <div class="packet-header">
          <span class="packet-direction">
            <el-icon v-if="packet.direction === 'received'"><Download /></el-icon>
            <el-icon v-else><Upload /></el-icon>
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
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { Refresh, Delete, Download, Upload } from '@element-plus/icons-vue'
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
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .header-left {
      display: flex;
      align-items: center;
      gap: 10px;
    }

    .header-right {
      display: flex;
      align-items: center;
      gap: 10px;
    }
  }

  .packet-list {
    height: 400px;
    overflow-y: auto;
    background: #1e1e1e;
    border-radius: 4px;
    padding: 10px;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 12px;
  }

  .packet-item {
    margin-bottom: 8px;
    padding: 8px;
    border-radius: 4px;
    background: #2d2d2d;

    &.packet-received {
      border-left: 3px solid #67c23a;

      .packet-direction {
        color: #67c23a;
      }
    }

    &.packet-sent {
      border-left: 3px solid #409eff;

      .packet-direction {
        color: #409eff;
      }
    }

    .packet-header {
      display: flex;
      align-items: center;
      gap: 15px;
      margin-bottom: 6px;
      color: #909399;
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
        color: #e6a23c;
        word-break: break-all;
        line-height: 1.6;
      }
    }
  }
}
</style>
