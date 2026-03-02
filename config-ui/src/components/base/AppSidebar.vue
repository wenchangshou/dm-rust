<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../composables/useI18n'
import type { LocaleCode } from '../../locales/messages'
import type { PageKey } from '../../types/config'

const props = defineProps<{
  activePage: PageKey
  channelsCount: number
  nodesCount: number
  scenesCount: number
  saving: boolean
  loading: boolean
}>()

const emit = defineEmits<{
  (e: 'change-page', page: PageKey): void
  (e: 'save'): void
  (e: 'reload'): void
}>()

const { t, locale, setLocale, languageOptions } = useI18n()

const navItems = computed(() => [
  {
    key: 'channels' as const,
    label: t('sidebar.channels'),
    count: props.channelsCount,
    icon: 'Connection'
  },
  {
    key: 'nodes' as const,
    label: t('sidebar.nodes'),
    count: props.nodesCount,
    icon: 'Cpu'
  },
  {
    key: 'scenes' as const,
    label: t('sidebar.scenes'),
    count: props.scenesCount,
    icon: 'Film'
  }
])

const onLocaleChange = (value: string) => {
  const next = value as LocaleCode
  setLocale(next)
}

const onMenuSelect = (key: string) => {
  emit('change-page', key as PageKey)
}
</script>

<template>
  <aside class="enterprise-sidebar">
    <div class="brand-panel">
      <div class="brand-row">
        <div class="brand-logo">
          <el-icon><Operation /></el-icon>
        </div>
        <div>
          <div class="brand-title">{{ t('sidebar.title') }}</div>
          <div class="brand-subtitle">{{ t('app.subtitle') }}</div>
        </div>
      </div>

      <div class="language-box">
        <span>{{ t('sidebar.language') }}</span>
        <el-select :model-value="locale" class="full-width" @change="onLocaleChange">
          <el-option
            v-for="option in languageOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </div>
    </div>

    <el-menu :default-active="activePage" class="nav-menu" @select="onMenuSelect">
      <el-menu-item v-for="item in navItems" :key="item.key" :index="item.key">
        <el-icon>
          <component :is="item.icon" />
        </el-icon>
        <span>{{ item.label }}</span>
        <el-tag class="count-tag" size="small" effect="dark">{{ item.count }}</el-tag>
      </el-menu-item>
    </el-menu>

    <div class="ops-panel">
      <div class="ops-title">{{ t('sidebar.operations') }}</div>
      <el-button type="primary" :loading="saving" class="full-width" @click="emit('save')">
        {{ saving ? t('common.saving') : t('common.save') }}
      </el-button>
      <el-button :loading="loading" class="full-width" @click="emit('reload')">
        {{ loading ? t('common.loading') : t('common.reload') }}
      </el-button>
    </div>
  </aside>
</template>

<style scoped>
.enterprise-sidebar {
  height: 100%;
  display: grid;
  grid-template-rows: auto 1fr auto;
  background: linear-gradient(180deg, #101e38 0%, #121f34 55%, #0f1a2d 100%);
  color: #dbeafe;
}

.brand-panel {
  padding: 18px 14px 12px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.25);
  display: grid;
  gap: 12px;
}

.brand-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-logo {
  width: 34px;
  height: 34px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  background: linear-gradient(135deg, #2563eb, #0ea5e9);
  color: #eff6ff;
  font-size: 16px;
}

.brand-title {
  font-size: 14px;
  font-weight: 700;
  color: #f8fafc;
}

.brand-subtitle {
  margin-top: 2px;
  font-size: 11px;
  color: #93c5fd;
  line-height: 1.4;
}

.language-box {
  display: grid;
  gap: 6px;
}

.language-box span {
  font-size: 12px;
  color: #93c5fd;
}

.full-width {
  width: 100%;
}

.nav-menu {
  border-right: none;
  background: transparent;
  padding: 8px;
}

:deep(.nav-menu .el-menu-item) {
  color: #dbeafe;
  border-radius: 10px;
  margin-bottom: 6px;
}

:deep(.nav-menu .el-menu-item:hover) {
  background: rgba(37, 99, 235, 0.2);
}

:deep(.nav-menu .el-menu-item.is-active) {
  background: linear-gradient(90deg, #2563eb, #1d4ed8);
  color: #f8fafc;
}

.count-tag {
  margin-left: auto;
}

.ops-panel {
  padding: 12px;
  border-top: 1px solid rgba(148, 163, 184, 0.25);
  display: grid;
  gap: 8px;
}

.ops-title {
  font-size: 12px;
  color: #93c5fd;
}

:deep(.ops-panel .el-button:not(.el-button--primary)) {
  background: rgba(148, 163, 184, 0.2);
  border-color: rgba(148, 163, 184, 0.32);
  color: #e2e8f0;
}

@media (max-width: 960px) {
  .enterprise-sidebar {
    height: auto;
    grid-template-rows: auto auto auto;
  }

  .nav-menu {
    padding-top: 6px;
  }
}
</style>
