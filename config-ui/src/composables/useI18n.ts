import { computed, ref, watch } from 'vue'
import { messages, type LocaleCode } from '../locales/messages'

const STORAGE_KEY = 'dm-rust-config-locale'

function resolveLocale(): LocaleCode {
  const saved = localStorage.getItem(STORAGE_KEY)
  if (saved === 'zh-CN' || saved === 'en-US') {
    return saved
  }
  return 'zh-CN'
}

const locale = ref<LocaleCode>(resolveLocale())

watch(
  locale,
  (value) => {
    localStorage.setItem(STORAGE_KEY, value)
    document.documentElement.lang = value
  },
  { immediate: true }
)

function readPath(path: string, source: Record<string, unknown>): unknown {
  return path.split('.').reduce<unknown>((acc, key) => {
    if (acc && typeof acc === 'object' && key in acc) {
      return (acc as Record<string, unknown>)[key]
    }
    return undefined
  }, source)
}

function formatTemplate(template: string, params?: Record<string, string | number>) {
  if (!params) {
    return template
  }

  return template.replace(/\{\{(\w+)\}\}/g, (_, key: string) => {
    if (key in params) {
      return String(params[key])
    }
    return `{{${key}}}`
  })
}

export function useI18n() {
  const t = (key: string, params?: Record<string, string | number>) => {
    const active = messages[locale.value] as Record<string, unknown>
    const fallback = messages['zh-CN'] as Record<string, unknown>
    const text = readPath(key, active) ?? readPath(key, fallback)

    if (typeof text === 'string') {
      return formatTemplate(text, params)
    }

    return key
  }

  const languageOptions = computed(() => [
    { value: 'zh-CN' as const, label: t('sidebar.chinese') },
    { value: 'en-US' as const, label: t('sidebar.english') }
  ])

  const setLocale = (next: LocaleCode) => {
    locale.value = next
  }

  return {
    locale,
    t,
    setLocale,
    languageOptions
  }
}
