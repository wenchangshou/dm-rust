import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'system'

export const useThemeStore = defineStore('theme', () => {
    // 当前主题模式 (light/dark/system)
    const mode = ref<ThemeMode>(getInitialMode())

    // 实际应用的主题 (light/dark)
    const resolvedTheme = ref<'light' | 'dark'>(getSystemTheme())

    // 获取初始主题模式
    function getInitialMode(): ThemeMode {
        const saved = localStorage.getItem('theme-mode')
        if (saved && ['light', 'dark', 'system'].includes(saved)) {
            return saved as ThemeMode
        }
        return 'system'
    }

    // 获取系统主题
    function getSystemTheme(): 'light' | 'dark' {
        if (typeof window !== 'undefined' && window.matchMedia) {
            return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
        }
        return 'dark'
    }

    // 更新解析后的主题
    function updateResolvedTheme() {
        if (mode.value === 'system') {
            resolvedTheme.value = getSystemTheme()
        } else {
            resolvedTheme.value = mode.value
        }
        applyTheme()
    }

    // 应用主题到 DOM
    function applyTheme() {
        const root = document.documentElement
        root.setAttribute('data-theme', resolvedTheme.value)

        // 同时更新 meta theme-color
        const metaThemeColor = document.querySelector('meta[name="theme-color"]')
        if (metaThemeColor) {
            metaThemeColor.setAttribute(
                'content',
                resolvedTheme.value === 'dark' ? '#0f0f23' : '#f8fafc'
            )
        }
    }

    // 设置主题模式
    function setMode(newMode: ThemeMode) {
        mode.value = newMode
        localStorage.setItem('theme-mode', newMode)
        updateResolvedTheme()
    }

    // 切换主题
    function toggleTheme() {
        if (mode.value === 'dark') {
            setMode('light')
        } else if (mode.value === 'light') {
            setMode('system')
        } else {
            setMode('dark')
        }
    }

    // 监听系统主题变化
    if (typeof window !== 'undefined' && window.matchMedia) {
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
        mediaQuery.addEventListener('change', () => {
            if (mode.value === 'system') {
                updateResolvedTheme()
            }
        })
    }

    // 初始化时应用主题
    updateResolvedTheme()

    // 监听模式变化
    watch(mode, updateResolvedTheme)

    return {
        mode,
        resolvedTheme,
        setMode,
        toggleTheme,
    }
})
