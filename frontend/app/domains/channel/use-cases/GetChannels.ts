import type { IChannelRepository } from '../repositories/IChannelRepository'
import type { Channel } from '../entities/Channel'

export class GetChannels {
  constructor(private readonly repo: IChannelRepository) {}

  async execute(): Promise<Channel[]> {
    return this.repo.getAll()
  }
}
