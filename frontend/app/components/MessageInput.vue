<script setup lang="ts">
import { nextTick, ref } from 'vue'

const props = defineProps<{ disabled?: boolean }>()
const emit = defineEmits<{ send: [text: string] }>()

const text = ref('')
const textareaRef = ref<HTMLTextAreaElement | null>(null)

function autoResize() {
  const el = textareaRef.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = `${Math.min(el.scrollHeight, 160)}px`
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    submit()
  }
}

function submit() {
  const trimmed = text.value.trim()
  if (!trimmed || props.disabled) return
  emit('send', trimmed)
  text.value = ''
  nextTick(autoResize)
}
</script>

<template>
  <div class="border-t border-gray-200 dark:border-gray-800 px-4 py-3 bg-white dark:bg-gray-900">
    <div class="flex items-end gap-2">
      <textarea
        ref="textareaRef"
        v-model="text"
        class="flex-1 resize-none rounded-xl border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 px-3 py-2 text-sm leading-relaxed focus:outline-none focus:ring-2 focus:ring-primary-500 transition-all"
        :class="{ 'opacity-50 cursor-not-allowed': disabled }"
        rows="1"
        placeholder="Message… (Enter to send, Shift+Enter for new line)"
        :disabled="disabled"
        @input="autoResize"
        @keydown="handleKeydown"
      />
      <UButton
        icon="i-heroicons-paper-airplane"
        :disabled="!text.trim() || disabled"
        @click="submit"
      />
    </div>
  </div>
</template>
