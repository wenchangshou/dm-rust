<template>
  <el-config-provider :locale="zhCn">
    <el-container class="app-container">
      <el-header class="app-header">
        <div class="header-left">
          <el-icon :size="24"><Connection /></el-icon>
          <span class="app-title">TCP 模拟器管理</span>
        </div>
        <div class="header-right">
          <el-button text @click="refreshAll">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </div>
      </el-header>
      <el-main class="app-main">
        <router-view />
      </el-main>
    </el-container>
  </el-config-provider>
</template>

<script setup lang="ts">
import { Connection, Refresh } from '@element-plus/icons-vue'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'
import { useSimulatorStore } from '@/stores/simulator'

const simulatorStore = useSimulatorStore()

const refreshAll = () => {
  simulatorStore.fetchSimulators()
  simulatorStore.fetchProtocols()
}
</script>

<style lang="scss" scoped>
.app-container {
  height: 100vh;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #409eff;
  color: white;
  padding: 0 20px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.app-title {
  font-size: 18px;
  font-weight: 600;
}

.header-right {
  .el-button {
    color: white;
  }
}

.app-main {
  background: #f5f7fa;
  padding: 20px;
}
</style>
