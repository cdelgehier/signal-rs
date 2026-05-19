<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from 'vue'
import type { Message } from '../domains/message/entities/Message'

const props = defineProps<{
  messages: Message[]
  loading: boolean
}>()

const listRef = ref<HTMLElement | null>(null)

async function scrollToBottom() {
  await nextTick()
  if (listRef.value) {
    listRef.value.scrollTop = listRef.value.scrollHeight
  }
}

watch(() => props.messages.length, scrollToBottom)
onMounted(scrollToBottom)
</script>

<template>
  <div
    ref="listRef"
    class="flex-1 overflow-y-auto px-4"
  >
    <div v-if="loading" class="h-full flex items-center justify-center">
      <UIcon name="i-heroicons-arrow-path" class="animate-spin size-5 text-gray-400" />
    </div>

    <div v-else-if="messages.length === 0" class="h-full flex items-center justify-center">
      <p class="text-sm text-gray-500 dark:text-gray-500">
        No messages yet. Say hello!
      </p>
    </div>

    <!-- flex-col with justify-end pushes messages to the bottom -->
    <div v-else class="min-h-full flex flex-col justify-end py-4 space-y-1">
      <MessageBubble
        v-for="msg in messages"
        :key="msg.id"
        :message="msg"
      />
    </div>
  </div>
</template>
