import type { IChannelRepository } from '../../domains/channel/repositories/IChannelRepository'
import { GetChannels } from '../../domains/channel/use-cases/GetChannels'
import type { Channel } from '../../domains/channel/entities/Channel'

export class ChannelService {
  private getChannels: GetChannels

  constructor(repo: IChannelRepository) {
    this.getChannels = new GetChannels(repo)
  }

  async listChannels(): Promise<Channel[]> {
    return this.getChannels.execute()
  }
}
