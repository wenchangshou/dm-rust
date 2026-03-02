export type LogLevel = 'debug' | 'info' | 'warn' | 'error'

const STORAGE_KEY = 'config-ui-debug'
const PREFIX = '[ConfigUI]'

function hasWindow() {
  return typeof window !== 'undefined'
}

function queryFlagEnabled() {
  if (!hasWindow()) {
    return false
  }

  const search = new URLSearchParams(window.location.search)
  return search.get('debugConfigUi') === '1'
}

function storageFlagEnabled() {
  if (!hasWindow()) {
    return false
  }

  return window.localStorage.getItem(STORAGE_KEY) === '1'
}

function shouldLog() {
  return import.meta.env.DEV || queryFlagEnabled() || storageFlagEnabled()
}

function print(level: LogLevel, scope: string, message: string, payload?: unknown) {
  if (!shouldLog()) {
    return
  }

  const head = `${PREFIX}[${scope}] ${message}`
  if (payload === undefined) {
    console[level](head)
    return
  }

  console[level](head, payload)
}

export const logger = {
  debug(scope: string, message: string, payload?: unknown) {
    print('debug', scope, message, payload)
  },
  info(scope: string, message: string, payload?: unknown) {
    print('info', scope, message, payload)
  },
  warn(scope: string, message: string, payload?: unknown) {
    print('warn', scope, message, payload)
  },
  error(scope: string, message: string, payload?: unknown) {
    print('error', scope, message, payload)
  }
}
