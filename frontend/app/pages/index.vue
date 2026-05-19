<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ChannelService } from '../application/channel/channel-service'
import { MessageService } from '../application/message/message-service'
import { TauriChannelRepository } from '../infrastructure/channel/tauri-channel-repository'
import { TauriMessageRepository } from '../infrastructure/message/tauri-message-repository'
import type { Channel } from '../domains/channel/entities/Channel'
import type { Message } from '../domains/message/entities/Message'

const router = useRouter()
const channelService = new ChannelService(new TauriChannelRepository())
const messageService = new MessageService(new TauriMessageRepository())

const channels = ref<Channel[]>([])
const activeChannelId = ref<string | null>(null)
const messages = ref<Message[]>([])
const channelsLoading = ref(true)
const messagesLoading = ref(false)
const sendDisabled = ref(false)

onMounted(async () => {
  try {
    const status = await invoke<string>('get_pairing_status')
    if (status === 'pending') { router.replace('/onboarding'); return }
  }
  catch { router.replace('/onboarding'); return }

  // Register listeners BEFORE loading to avoid race condition with Rust events
  await listen('contacts-updated', () => loadChannels())

  // Device was deassociated from phone → back to QR pairing
  await listen('deassociated', () => router.replace('/onboarding'))

  // payload = [channelId, message]
  await listen<[string, Message]>('message-received', ({ payload }) => {
    const [channelId, msg] = payload
    if (channelId === activeChannelId.value) {
      messages.value.push(msg)
    }
    const ch = channels.value.find(c => c.id === channelId)
    if (ch) {
      ch.lastMessage = msg.text
      ch.lastMessageTime = msg.timestamp
    }
    else {
      // Unknown channel — reload the full list
      loadChannels()
    }
  })

  await loadChannels()
})

async function loadChannels() {
  channelsLoading.value = true
  try { channels.value = await channelService.listChannels() }
  catch (e) { console.error('Failed to load channels', e) }
  finally { channelsLoading.value = false }
}

async function selectChannel(id: string) {
  activeChannelId.value = id
  messagesLoading.value = true
  try { messages.value = await messageService.getMessages(id) }
  catch (e) { console.error('Failed to load messages', e) }
  finally { messagesLoading.value = false }
}

async function handleSend(text: string) {
  if (!activeChannelId.value) return
  sendDisabled.value = true
  try {
    await messageService.send(activeChannelId.value, text)
    const now = Date.now()
    messages.value.push({
      id: now, senderId: 'me', senderName: 'Me',
      text, timestamp: now, isOutgoing: true, receipt: 'sent',
    })
    // Update sidebar last message immediately
    const ch = channels.value.find(c => c.id === activeChannelId.value)
    if (ch) { ch.lastMessage = text; ch.lastMessageTime = now }
  }
  catch (e) { console.error('Failed to send', e) }
  finally { sendDisabled.value = false }
}

const activeChannel = computed(() =>
  channels.value.find(c => c.id === activeChannelId.value) ?? null,
)
</script>

<template>
  <div class="h-screen flex overflow-hidden bg-gray-50 dark:bg-gray-950">
    <AppSidebar
      :channels="channels"
      :active-channel-id="activeChannelId"
      :loading="channelsLoading"
      @select="selectChannel"
    />

    <!-- Chat area -->
    <div class="flex-1 flex flex-col min-w-0 bg-white dark:bg-gray-900">
      <!-- Header -->
      <div class="h-14 px-4 flex items-center justify-between border-b border-gray-200 dark:border-gray-800 flex-shrink-0">
        <div v-if="activeChannel" class="flex items-center gap-3">
          <UAvatar :text="activeChannel.name.slice(0, 2).toUpperCase()" size="sm" />
          <div>
            <p class="font-semibold text-sm text-gray-900 dark:text-white">
              {{ activeChannel.name }}
            </p>
          </div>
        </div>
        <p v-else class="text-sm text-gray-400">
          Select a conversation
        </p>
        <div v-if="activeChannel" class="flex items-center gap-1">
          <UButton icon="i-heroicons-magnifying-glass" variant="ghost" color="neutral" size="sm" />
          <UButton icon="i-heroicons-ellipsis-vertical" variant="ghost" color="neutral" size="sm" />
        </div>
      </div>

      <!-- Messages -->
      <MessageList :messages="messages" :loading="messagesLoading" />

      <!-- Input -->
      <MessageInput :disabled="!activeChannelId || sendDisabled" @send="handleSend" />
    </div>
  </div>
</template>
