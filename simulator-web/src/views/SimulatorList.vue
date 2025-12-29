<template>
  <div class="simulator-list">
    <!-- 页面头部 -->
    <div class="page-header">
      <h2>模拟器列表</h2>
      <el-button type="primary" @click="showCreateDialog = true">
        <el-icon><Plus /></el-icon>
        创建模拟器
      </el-button>
    </div>

    <!-- 统计卡片 -->
    <el-row :gutter="20" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-value">{{ store.simulators.length }}</div>
          <div class="stat-label">总数</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-value status-running">{{ store.runningCount }}</div>
          <div class="stat-label">运行中</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-value status-stopped">{{ store.stoppedCount }}</div>
          <div class="stat-label">已停止</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-value">{{ store.totalConnections }}</div>
          <div class="stat-label">活动连接</div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 模拟器表格 -->
    <el-card>
      <el-table :data="store.simulators" v-loading="store.loading" stripe>
        <el-table-column prop="name" label="名称" min-width="150">
          <template #default="{ row }">
            <router-link :to="`/simulator/${row.id}`" class="simulator-link">
              {{ row.name }}
            </router-link>
          </template>
        </el-table-column>
        <el-table-column prop="protocol" label="协议" width="120" />
        <el-table-column label="地址" width="180">
          <template #default="{ row }">
            {{ row.bind_addr }}:{{ row.port }}
          </template>
        </el-table-column>
        <el-table-column label="状态" width="100">
          <template #default="{ row }">
            <StatusBadge :status="row.status" />
          </template>
        </el-table-column>
        <el-table-column label="设备状态" width="120">
          <template #default="{ row }">
            <el-tag :type="row.state.online ? 'success' : 'info'" size="small">
              {{ row.state.online ? '在线' : '离线' }}
            </el-tag>
            <el-tag v-if="row.state.fault" type="danger" size="small" style="margin-left: 4px">
              故障
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="连接" width="80">
          <template #default="{ row }">
            {{ row.state.stats.active_connections }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button-group>
              <el-button
                v-if="row.status === 'stopped'"
                type="success"
                size="small"
                @click="handleStart(row)"
              >
                启动
              </el-button>
              <el-button
                v-else
                type="warning"
                size="small"
                @click="handleStop(row)"
              >
                停止
              </el-button>
              <el-button
                type="danger"
                size="small"
                @click="handleDelete(row)"
              >
                删除
              </el-button>
            </el-button-group>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 创建对话框 -->
    <CreateSimulatorDialog
      v-model:visible="showCreateDialog"
      :protocols="store.protocols"
      @created="handleCreated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useSimulatorStore } from '@/stores/simulator'
import type { SimulatorInfo } from '@/types/simulator'
import StatusBadge from '@/components/common/StatusBadge.vue'
import CreateSimulatorDialog from '@/components/simulator/CreateSimulatorDialog.vue'

const store = useSimulatorStore()
const showCreateDialog = ref(false)

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
</script>

<style lang="scss" scoped>
.simulator-list {
  .stats-row {
    margin-bottom: 20px;
  }

  .simulator-link {
    color: #409eff;
    text-decoration: none;
    font-weight: 500;

    &:hover {
      text-decoration: underline;
    }
  }
}
</style>
