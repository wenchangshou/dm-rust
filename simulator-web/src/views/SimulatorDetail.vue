<template>
  <div class="simulator-detail" v-loading="store.loading">
    <template v-if="simulator">
      <!-- 页面头部 -->
      <div class="page-header">
        <div class="header-left">
          <el-button text @click="router.back()">
            <el-icon><ArrowLeft /></el-icon>
            返回
          </el-button>
          <h2>{{ simulator.name }}</h2>
          <StatusBadge :status="simulator.status" />
        </div>
        <div class="header-right">
          <el-button-group>
            <el-button
              v-if="simulator.status === 'stopped'"
              type="success"
              @click="handleStart"
            >
              <el-icon><VideoPlay /></el-icon>
              启动
            </el-button>
            <el-button
              v-else
              type="warning"
              @click="handleStop"
            >
              <el-icon><VideoPause /></el-icon>
              停止
            </el-button>
            <el-button @click="refresh">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </el-button-group>
        </div>
      </div>

      <el-row :gutter="20">
        <!-- 基本信息 -->
        <el-col :span="12">
          <el-card>
            <template #header>
              <span>基本信息</span>
            </template>
            <el-descriptions :column="1" border>
              <el-descriptions-item label="ID">{{ simulator.id }}</el-descriptions-item>
              <el-descriptions-item label="协议">{{ simulator.protocol }}</el-descriptions-item>
              <el-descriptions-item label="监听地址">
                {{ simulator.bind_addr }}:{{ simulator.port }}
              </el-descriptions-item>
              <el-descriptions-item label="状态">
                <StatusBadge :status="simulator.status" />
              </el-descriptions-item>
            </el-descriptions>
          </el-card>
        </el-col>

        <!-- 连接统计 -->
        <el-col :span="12">
          <el-card>
            <template #header>
              <span>连接统计</span>
            </template>
            <el-descriptions :column="2" border>
              <el-descriptions-item label="总连接数">
                {{ simulator.state.stats.total_connections }}
              </el-descriptions-item>
              <el-descriptions-item label="活动连接">
                {{ simulator.state.stats.active_connections }}
              </el-descriptions-item>
              <el-descriptions-item label="接收字节">
                {{ formatBytes(simulator.state.stats.bytes_received) }}
              </el-descriptions-item>
              <el-descriptions-item label="发送字节">
                {{ formatBytes(simulator.state.stats.bytes_sent) }}
              </el-descriptions-item>
              <el-descriptions-item label="最后活动" :span="2">
                {{ simulator.state.stats.last_activity || '无' }}
              </el-descriptions-item>
            </el-descriptions>
          </el-card>
        </el-col>
      </el-row>

      <!-- 设备状态控制 -->
      <el-card style="margin-top: 20px">
        <template #header>
          <span>设备状态模拟</span>
        </template>

        <el-form label-width="100px">
          <el-form-item label="在线状态">
            <el-switch
              v-model="isOnline"
              active-text="在线"
              inactive-text="离线"
              @change="handleOnlineChange"
            />
          </el-form-item>

          <el-form-item label="故障模拟">
            <template v-if="simulator.state.fault">
              <el-tag type="danger" size="large">
                {{ simulator.state.fault }}
              </el-tag>
              <el-button type="primary" size="small" style="margin-left: 10px" @click="handleClearFault">
                清除故障
              </el-button>
            </template>
            <template v-else>
              <el-select
                v-model="selectedFault"
                placeholder="选择故障类型"
                style="width: 200px"
              >
                <el-option label="通信超时" value="timeout" />
                <el-option label="协议错误" value="protocol_error" />
                <el-option label="设备故障" value="device_fault" />
                <el-option label="校验和错误" value="checksum_error" />
              </el-select>
              <el-button
                type="danger"
                style="margin-left: 10px"
                :disabled="!selectedFault"
                @click="handleTriggerFault"
              >
                触发故障
              </el-button>
            </template>
          </el-form-item>
        </el-form>
      </el-card>

      <!-- Modbus 模拟器专用面板 -->
      <ModbusSimulatorPanel
        v-if="isModbusProtocol"
        :simulator-id="id"
        style="margin-top: 20px"
      />

      <!-- 其他协议状态值 -->
      <el-card v-else style="margin-top: 20px">
        <template #header>
          <span>协议状态值</span>
        </template>
        <el-table :data="stateValues" stripe>
          <el-table-column prop="key" label="键" width="200" />
          <el-table-column prop="value" label="值">
            <template #default="{ row }">
              <code>{{ JSON.stringify(row.value) }}</code>
            </template>
          </el-table-column>
        </el-table>
        <el-empty v-if="stateValues.length === 0" description="暂无状态值" />
      </el-card>

      <!-- 报文监控（所有协议通用） -->
      <PacketMonitorPanel
        v-if="simulator.status === 'running'"
        :simulator-id="id"
        style="margin-top: 20px"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, VideoPlay, VideoPause, Refresh } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useSimulatorStore } from '@/stores/simulator'
import StatusBadge from '@/components/common/StatusBadge.vue'
import ModbusSimulatorPanel from '@/components/simulator/ModbusSimulatorPanel.vue'
import PacketMonitorPanel from '@/components/simulator/PacketMonitorPanel.vue'

const props = defineProps<{
  id: string
}>()

const router = useRouter()
const store = useSimulatorStore()

const simulator = computed(() => store.currentSimulator)
const isOnline = ref(true)
const selectedFault = ref('')

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
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;

    .header-left {
      display: flex;
      align-items: center;
      gap: 10px;

      h2 {
        margin: 0;
      }
    }
  }

  code {
    background: #f5f7fa;
    padding: 2px 6px;
    border-radius: 4px;
    font-family: monospace;
  }
}
</style>
