<script setup lang="ts">
import { computed } from 'vue'
import type { Message } from '../domains/message/entities/Message'

const props = defineProps<{ message: Message }>()

const time = computed(() =>
  new Date(props.message.timestamp).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
  }),
)

const receiptIcon = computed(() => {
  if (!props.message.isOutgoing) return null
  return props.message.receipt === 'read'
    ? 'i-heroicons-check-circle'
    : 'i-heroicons-check'
})
</script>

<template>
  <div
    class="flex mb-1.5"
    :class="message.isOutgoing ? 'justify-end' : 'justify-start'"
  >
    <UAvatar
      v-if="!message.isOutgoing"
      :text="message.senderName.slice(0, 2).toUpperCase()"
      size="xs"
      class="mr-2 mt-auto flex-shrink-0"
    />

    <div
      class="max-w-sm lg:max-w-md px-3.5 py-2 shadow-sm"
      :class="message.isOutgoing
        ? 'bg-[#3b82f6] text-white rounded-2xl rounded-br-sm'
        : 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-2xl rounded-bl-sm border border-gray-100 dark:border-gray-600'"
    >
      <p
        v-if="!message.isOutgoing"
        class="text-xs font-semibold mb-0.5"
        :class="message.isOutgoing ? 'text-blue-100' : 'text-blue-500'"
      >
        {{ message.senderName }}
      </p>

      <!-- Attachment placeholder -->
      <div
        v-if="message.text === null && !message.text"
        class="flex items-center gap-2 text-sm opacity-70 italic"
      >
        <UIcon name="i-heroicons-photo" class="size-4 flex-shrink-0" />
        <span>Attachment (not yet supported)</span>
      </div>

      <p v-else class="text-sm whitespace-pre-wrap break-words leading-relaxed">
        {{ message.text }}
      </p>

      <div class="flex items-center gap-1 justify-end mt-0.5">
        <span class="text-xs" :class="message.isOutgoing ? 'text-blue-200' : 'text-gray-400'">
          {{ time }}
        </span>
        <UIcon
          v-if="receiptIcon"
          :name="receiptIcon"
          class="size-3"
          :class="message.receipt === 'read' ? 'text-blue-300' : 'text-blue-200'"
        />
      </div>
    </div>
  </div>
</template>
