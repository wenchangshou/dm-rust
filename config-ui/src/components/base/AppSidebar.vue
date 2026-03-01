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
  { key: 'channels' as const, icon: '📡', label: t('sidebar.channels'), count: props.channelsCount },
  { key: 'nodes' as const, icon: '🔌', label: t('sidebar.nodes'), count: props.nodesCount },
  { key: 'scenes' as const, icon: '🎬', label: t('sidebar.scenes'), count: props.scenesCount }
])

const onLocaleChange = (event: Event) => {
  const next = (event.target as HTMLSelectElement).value as LocaleCode
  setLocale(next)
}
</script>

<template>
  <aside class="sidebar">
    <div class="brand">
      <span class="brand-icon">⚙</span>
      <div>
        <div class="brand-title">{{ t('sidebar.title') }}</div>
      </div>
    </div>

    <div class="language-switcher">
      <label>{{ t('sidebar.language') }}</label>
      <select :value="locale" @change="onLocaleChange">
        <option v-for="option in languageOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </div>

    <ul class="nav-list">
      <li
        v-for="item in navItems"
        :key="item.key"
        :class="{ active: activePage === item.key }"
        @click="emit('change-page', item.key)"
      >
        <span class="icon">{{ item.icon }}</span>
        <span>{{ item.label }}</span>
        <span class="count">{{ item.count }}</span>
      </li>
    </ul>

    <div class="actions">
      <button class="btn btn-primary" :disabled="saving" @click="emit('save')">
        {{ saving ? t('common.saving') : t('common.save') }}
      </button>
      <button class="btn btn-muted" :disabled="loading" @click="emit('reload')">
        {{ loading ? t('common.loading') : t('common.reload') }}
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 270px;
  min-width: 270px;
  background: linear-gradient(170deg, #0f172a, #111827 62%, #1e293b);
  color: #e2e8f0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid rgba(148, 163, 184, 0.2);
}

.brand {
  padding: 18px 16px 12px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-icon {
  font-size: 20px;
}

.brand-title {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.language-switcher {
  padding: 0 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.language-switcher label {
  font-size: 12px;
  color: #94a3b8;
}

.language-switcher select {
  height: 34px;
  border-radius: 8px;
  border: 1px solid rgba(148, 163, 184, 0.28);
  background: rgba(15, 23, 42, 0.55);
  color: #e2e8f0;
  padding: 0 10px;
}

.nav-list {
  list-style: none;
  margin: 4px 0 0;
  padding: 0 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.nav-list li {
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 8px;
  min-height: 38px;
  padding: 0 10px;
  color: #cbd5e1;
  cursor: pointer;
  transition: all 0.2s ease;
}

.nav-list li:hover {
  background: rgba(148, 163, 184, 0.16);
}

.nav-list li.active {
  background: linear-gradient(90deg, #2563eb, #1d4ed8);
  color: #f8fafc;
}

.icon {
  font-size: 15px;
}

.count {
  margin-left: auto;
  background: rgba(255, 255, 255, 0.12);
  border-radius: 999px;
  min-width: 24px;
  height: 20px;
  padding: 0 7px;
  display: inline-flex;
  justify-content: center;
  align-items: center;
  font-size: 11px;
}

.actions {
  padding: 14px;
  display: grid;
  gap: 8px;
  border-top: 1px solid rgba(148, 163, 184, 0.2);
}

.btn {
  height: 36px;
  border-radius: 9px;
  border: none;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.btn-primary {
  background: linear-gradient(90deg, #3b82f6, #2563eb);
  color: #eff6ff;
}

.btn-muted {
  background: rgba(148, 163, 184, 0.15);
  color: #e2e8f0;
  border: 1px solid rgba(148, 163, 184, 0.26);
}

@media (max-width: 960px) {
  .sidebar {
    width: 100%;
    min-width: 100%;
    border-right: none;
    border-bottom: 1px solid rgba(148, 163, 184, 0.2);
  }

  .actions {
    grid-template-columns: 1fr 1fr;
  }
}
</style>
