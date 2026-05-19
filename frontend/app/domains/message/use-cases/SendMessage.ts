import type { IMessageRepository } from '../repositories/IMessageRepository'

export class SendMessage {
  constructor(private readonly repo: IMessageRepository) {}

  async execute(channelId: string, text: string): Promise<void> {
    const trimmed = text.trim()
    if (!trimmed) throw new Error('Cannot send an empty message')
    return this.repo.send(channelId, trimmed)
  }
}
