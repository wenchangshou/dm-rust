import { onBeforeUnmount, ref } from 'vue'
import type { ToastType } from '../types/config'

export function useToast() {
  const message = ref('')
  const type = ref<ToastType>('success')
  let timer: ReturnType<typeof setTimeout> | null = null

  const clear = () => {
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
  }

  const show = (nextMessage: string, nextType: ToastType = 'success') => {
    message.value = nextMessage
    type.value = nextType
    clear()
    timer = setTimeout(() => {
      message.value = ''
      timer = null
    }, 3500)
  }

  onBeforeUnmount(clear)

  return {
    message,
    type,
    show,
    clear
  }
}
