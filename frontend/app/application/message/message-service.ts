import type { IMessageRepository } from '../../domains/message/repositories/IMessageRepository'
import { SendMessage } from '../../domains/message/use-cases/SendMessage'
import type { Message } from '../../domains/message/entities/Message'

export class MessageService {
  private sendMessage: SendMessage

  constructor(private readonly repo: IMessageRepository) {
    this.sendMessage = new SendMessage(repo)
  }

  async getMessages(channelId: string): Promise<Message[]> {
    return this.repo.getByChannel(channelId)
  }

  async send(channelId: string, text: string): Promise<void> {
    return this.sendMessage.execute(channelId, text)
  }
}
