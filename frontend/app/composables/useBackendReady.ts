import { invoke } from '@tauri-apps/api/core'

export function useBackendReady() {
  const ready = ref(false)
  const error = ref<string | null>(null)

  onMounted(async () => {
    const deadline = Date.now() + 30_000
    while (Date.now() < deadline) {
      try {
        await invoke('get_pairing_status')
        ready.value = true
        return
      }
      catch {
        await new Promise(r => setTimeout(r, 300))
      }
    }
    error.value = 'Timeout waiting for backend'
  })

  return { ready, error }
}
