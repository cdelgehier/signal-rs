<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits<{ paired: [] }>()

const qrBase64 = ref<string | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)

const steps = [
  { icon: 'i-heroicons-device-phone-mobile', label: 'Open Signal on your phone' },
  { icon: 'i-heroicons-link', label: 'Go to Linked devices' },
  { icon: 'i-heroicons-qr-code', label: 'Tap "Link a device" and scan' },
]

async function loadQr() {
  loading.value = true
  error.value = null
  qrBase64.value = null
  try {
    const b64 = await invoke<string>('generate_qr_code')
    if (!b64) {
      emit('paired')
      return
    }
    qrBase64.value = b64
  }
  catch (e) {
    error.value = String(e)
  }
  finally {
    loading.value = false
  }
}

onMounted(async () => {
  await loadQr()

  const interval = setInterval(async () => {
    try {
      const status = await invoke<string>('get_pairing_status')
      if (status === 'linked') {
        clearInterval(interval)
        emit('paired')
      }
    }
    catch { /* ignore */ }
  }, 2000)

  onUnmounted(() => clearInterval(interval))
})
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-950 dark:to-gray-900 p-6">
    <div class="w-full max-w-md space-y-6">
      <!-- Header -->
      <div class="text-center space-y-2">
        <div class="inline-flex items-center justify-center size-14 rounded-2xl bg-primary-500 shadow-lg mb-2">
          <UIcon name="i-heroicons-chat-bubble-left-right" class="size-7 text-white" />
        </div>
        <h1 class="text-2xl font-bold tracking-tight">
          Link Signal RS
        </h1>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Scan the QR code with your phone to get started
        </p>
      </div>

      <!-- QR card -->
      <UCard class="overflow-hidden">
        <!-- Loading -->
        <div v-if="loading" class="flex flex-col items-center justify-center py-12 gap-3">
          <UIcon name="i-heroicons-arrow-path" class="animate-spin size-8 text-primary-500" />
          <p class="text-sm text-gray-500">Connecting to Signal…</p>
        </div>

        <!-- Error -->
        <div v-else-if="error" class="p-4">
          <UAlert
            color="error"
            icon="i-heroicons-exclamation-triangle"
            title="Connection failed"
            :description="error"
          />
          <UButton
            class="mt-4 w-full"
            variant="outline"
            icon="i-heroicons-arrow-path"
            @click="loadQr"
          >
            Try again
          </UButton>
        </div>

        <!-- QR code -->
        <div v-else-if="qrBase64" class="flex flex-col items-center gap-4 p-6">
          <div class="rounded-xl overflow-hidden border-4 border-white shadow-md">
            <img
              :src="`data:image/png;base64,${qrBase64}`"
              alt="Signal QR Code"
              class="size-52 block"
            >
          </div>
          <div class="flex items-center gap-1.5 text-xs text-gray-400">
            <UIcon name="i-heroicons-clock" class="size-3.5" />
            <span>Expires in 30 seconds</span>
          </div>
          <UButton
            size="sm"
            variant="ghost"
            icon="i-heroicons-arrow-path"
            @click="loadQr"
          >
            Refresh QR code
          </UButton>
        </div>
      </UCard>

      <!-- Steps -->
      <UCard>
        <ol class="space-y-3">
          <li
            v-for="(step, i) in steps"
            :key="i"
            class="flex items-center gap-3"
          >
            <span class="flex-shrink-0 flex items-center justify-center size-7 rounded-full bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 text-xs font-semibold">
              {{ i + 1 }}
            </span>
            <div class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
              <UIcon :name="step.icon" class="size-4 text-gray-400" />
              {{ step.label }}
            </div>
          </li>
        </ol>
      </UCard>
    </div>
  </div>
</template>
