<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Channel } from '../domains/channel/entities/Channel'

const props = defineProps<{
  channels: Channel[]
  activeChannelId: string | null
  loading: boolean
}>()

const emit = defineEmits<{
  select: [channelId: string]
}>()

const search = ref('')

const filtered = computed(() =>
  props.channels.filter(c =>
    c.name.toLowerCase().includes(search.value.toLowerCase()),
  ),
)

function initials(name: string) {
  return name.split(' ').slice(0, 2).map(w => w[0]?.toUpperCase() ?? '').join('')
}

function formatTime(ts: number | null) {
  if (!ts) return ''
  const d = new Date(ts)
  const now = new Date()
  if (d.toDateString() === now.toDateString())
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  return d.toLocaleDateString([], { weekday: 'short' })
}
</script>

<template>
  <aside class="w-80 flex-shrink-0 flex flex-col border-r border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950">
    <!-- Header -->
    <div class="px-4 pt-5 pb-3 flex items-center justify-between">
      <h1 class="text-xl font-bold text-gray-900 dark:text-white">
        Chats
      </h1>
      <div class="flex items-center gap-1">
        <UButton icon="i-heroicons-pencil-square" variant="ghost" color="neutral" size="sm" />
        <UButton icon="i-heroicons-ellipsis-horizontal" variant="ghost" color="neutral" size="sm" />
      </div>
    </div>

    <!-- Search -->
    <div class="px-3 pb-3">
      <UInput
        v-model="search"
        icon="i-heroicons-magnifying-glass"
        placeholder="Search"
        size="sm"
        class="w-full"
      />
    </div>

    <USeparator />

    <!-- Loading -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <UIcon name="i-heroicons-arrow-path" class="animate-spin size-5 text-gray-400" />
    </div>

    <!-- Empty -->
    <div v-else-if="filtered.length === 0" class="flex-1 flex flex-col items-center justify-center gap-2 p-6 text-center">
      <UIcon name="i-heroicons-chat-bubble-left-ellipsis" class="size-10 text-gray-300 dark:text-gray-700" />
      <p class="text-sm text-gray-500 dark:text-gray-400">
        {{ search ? 'No results' : 'No conversations yet' }}
      </p>
    </div>

    <!-- List -->
    <ul v-else class="flex-1 overflow-y-auto">
      <li
        v-for="channel in filtered"
        :key="channel.id"
        class="flex items-center gap-3 px-4 py-3 cursor-pointer transition-colors"
        :class="channel.id === activeChannelId
          ? 'bg-primary-50 dark:bg-primary-900/20'
          : 'hover:bg-gray-50 dark:hover:bg-gray-900'"
        @click="emit('select', channel.id)"
      >
        <UAvatar :text="initials(channel.name)" size="md" />

        <div class="flex-1 min-w-0">
          <div class="flex items-baseline justify-between gap-2">
            <span class="font-medium text-sm text-gray-900 dark:text-white truncate">
              {{ channel.name }}
            </span>
            <span class="text-xs text-gray-400 flex-shrink-0">
              {{ formatTime(channel.lastMessageTime) }}
            </span>
          </div>
          <div class="flex items-center justify-between gap-2">
            <p class="text-xs text-gray-500 dark:text-gray-400 truncate">
              {{ channel.lastMessage ?? 'No messages' }}
            </p>
            <UBadge
              v-if="channel.unreadCount > 0"
              :label="String(channel.unreadCount)"
              color="primary"
              size="xs"
              class="flex-shrink-0"
            />
          </div>
        </div>
      </li>
    </ul>
  </aside>
</template>
