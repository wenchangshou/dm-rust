<template>
  <el-config-provider :locale="zhCn">
    <div class="app-wrapper">
      <header class="app-header">
        <div class="header-content">
          <div class="header-left">
            <div class="logo-wrapper">
              <el-icon :size="22"><Connection /></el-icon>
            </div>
            <span class="app-title">TCP 模拟器管理</span>
          </div>
          <div class="header-right">
            <!-- 主题切换按钮 -->
            <el-tooltip :content="themeTooltip" placement="bottom">
              <button class="theme-btn" @click="themeStore.toggleTheme">
                <el-icon :size="18" class="theme-icon" :class="themeStore.resolvedTheme">
                  <Sunny v-if="themeStore.resolvedTheme === 'light'" />
                  <Moon v-else />
                </el-icon>
                <span class="theme-indicator" v-if="themeStore.mode === 'system'">
                  <el-icon :size="10"><Monitor /></el-icon>
                </span>
              </button>
            </el-tooltip>
            
            <el-button class="refresh-btn" @click="refreshAll">
              <el-icon class="refresh-icon"><Refresh /></el-icon>
              <span>刷新</span>
            </el-button>
          </div>
        </div>
      </header>
      <main class="app-main">
        <router-view />
      </main>
    </div>
  </el-config-provider>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Connection, Refresh, Sunny, Moon, Monitor } from '@element-plus/icons-vue'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'
import { useSimulatorStore } from '@/stores/simulator'
import { useThemeStore } from '@/stores/theme'

const simulatorStore = useSimulatorStore()
const themeStore = useThemeStore()

const themeTooltip = computed(() => {
  const modeLabels = {
    dark: '深色模式 (点击切换为浅色)',
    light: '浅色模式 (点击切换为跟随系统)',
    system: `跟随系统 (当前: ${themeStore.resolvedTheme === 'dark' ? '深色' : '浅色'}, 点击切换为深色)`
  }
  return modeLabels[themeStore.mode]
})

const refreshAll = () => {
  simulatorStore.fetchSimulators()
  simulatorStore.fetchProtocols()
}
</script>

<style lang="scss" scoped>
.app-wrapper {
  min-height: 100vh;
  background: var(--bg-base);
  position: relative;
  transition: background-color 0.3s ease;

  // 背景装饰
  &::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: 
      radial-gradient(circle at 20% 20%, rgba(102, 126, 234, 0.08) 0%, transparent 40%),
      radial-gradient(circle at 80% 80%, rgba(118, 75, 162, 0.08) 0%, transparent 40%);
    pointer-events: none;
    z-index: 0;
  }
}

[data-theme="light"] .app-wrapper::before {
  background: 
    radial-gradient(circle at 20% 20%, rgba(102, 126, 234, 0.05) 0%, transparent 40%),
    radial-gradient(circle at 80% 80%, rgba(118, 75, 162, 0.05) 0%, transparent 40%);
}

.app-header {
  position: sticky;
  top: 0;
  z-index: 100;
  background: rgba(15, 15, 35, 0.85);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--border-glass);
  box-shadow: 0 4px 30px rgba(0, 0, 0, 0.3);
  transition: background-color 0.3s ease, box-shadow 0.3s ease;
}

[data-theme="light"] .app-header {
  background: rgba(255, 255, 255, 0.9);
  box-shadow: 0 4px 30px rgba(0, 0, 0, 0.08);
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  height: 64px;
  max-width: 1600px;
  margin: 0 auto;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 10px;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
  color: white;
  transition: all 0.3s ease;

  &:hover {
    transform: rotate(-5deg) scale(1.05);
    box-shadow: 0 6px 20px rgba(102, 126, 234, 0.5);
  }
}

.app-title {
  font-size: 20px;
  font-weight: 600;
  background: linear-gradient(135deg, var(--text-primary) 0%, var(--text-secondary) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  letter-spacing: 0.5px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

// 主题切换按钮
.theme-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: var(--bg-input);
  border: 1px solid var(--border-glass);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.3s ease;

  &:hover {
    background: var(--bg-hover);
    border-color: var(--primary-start);
    transform: translateY(-2px);
  }

  .theme-icon {
    color: var(--text-secondary);
    transition: all 0.3s ease;

    &.dark {
      color: #feca57;
    }

    &.light {
      color: #f59e0b;
    }
  }

  .theme-indicator {
    position: absolute;
    bottom: -2px;
    right: -2px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    background: var(--primary-gradient);
    border-radius: 50%;
    color: white;
  }
}

.refresh-btn {
  background: var(--bg-input);
  border: 1px solid var(--border-glass);
  color: var(--text-secondary);
  padding: 10px 18px;
  border-radius: 10px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.3s ease;

  &:hover {
    background: var(--bg-hover);
    border-color: var(--primary-start);
    color: var(--text-primary);
    transform: translateY(-2px);

    .refresh-icon {
      transform: rotate(180deg);
    }
  }

  .refresh-icon {
    transition: transform 0.5s ease;
  }
}

.app-main {
  position: relative;
  z-index: 1;
  padding: 24px;
  max-width: 1600px;
  margin: 0 auto;
}
</style>
