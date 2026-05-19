import { invoke } from '@tauri-apps/api/core'
import type { IMessageRepository } from '../../domains/message/repositories/IMessageRepository'
import type { Message } from '../../domains/message/entities/Message'
import { MessageSchema } from '../../domains/message/schemas/message.schema'
import { z } from 'zod'

export class TauriMessageRepository implements IMessageRepository {
  async getByChannel(channelId: string): Promise<Message[]> {
    const raw = await invoke<unknown[]>('get_messages', { channelId })
    return z.array(MessageSchema).parse(raw).map(r => ({
      id: r.id,
      senderId: r.sender_id,
      senderName: r.sender_name,
      text: r.text,
      timestamp: r.timestamp,
      isOutgoing: r.is_outgoing,
      receipt: r.receipt,
    }))
  }

  async send(channelId: string, text: string): Promise<void> {
    await invoke('send_message', { channelId, text })
  }
}
