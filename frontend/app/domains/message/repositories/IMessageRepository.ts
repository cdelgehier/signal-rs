import type { Message } from '../entities/Message'

export interface IMessageRepository {
  getByChannel(channelId: string): Promise<Message[]>
  send(channelId: string, text: string): Promise<void>
}
