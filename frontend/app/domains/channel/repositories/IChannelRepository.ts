import type { Channel } from '../entities/Channel'

export interface IChannelRepository {
  getAll(): Promise<Channel[]>
}
