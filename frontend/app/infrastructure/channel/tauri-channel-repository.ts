import { invoke } from '@tauri-apps/api/core'
import type { IChannelRepository } from '../../domains/channel/repositories/IChannelRepository'
import type { Channel } from '../../domains/channel/entities/Channel'
import { ChannelSchema } from '../../domains/channel/schemas/channel.schema'
import { z } from 'zod'

export class TauriChannelRepository implements IChannelRepository {
  async getAll(): Promise<Channel[]> {
    const raw = await invoke<unknown[]>('get_channels')
    return z.array(ChannelSchema).parse(raw).map(r => ({
      id: r.id,
      name: r.name,
      lastMessage: r.last_message,
      lastMessageTime: r.last_message_time,
      unreadCount: r.unread_count,
      isGroup: r.is_group,
    }))
  }
}
