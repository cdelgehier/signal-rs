import { describe, it, expect, vi } from 'vitest'
import { GetChannels } from '../GetChannels'
import type { IChannelRepository } from '../../repositories/IChannelRepository'
import type { Channel } from '../../entities/Channel'

const fakeChannels: Channel[] = [
  { id: '1', name: 'Alice', lastMessage: 'Hey', lastMessageTime: 1_700_000_000, unreadCount: 1, isGroup: false },
  { id: '2', name: 'Groupe', lastMessage: null, lastMessageTime: null, unreadCount: 0, isGroup: true },
]

describe('GetChannels', () => {
  it('retourne la liste des channels depuis le repo', async () => {
    const repo: IChannelRepository = { getAll: vi.fn().mockResolvedValue(fakeChannels) }
    const useCase = new GetChannels(repo)
    const result = await useCase.execute()
    expect(result).toHaveLength(2)
    expect(result[0].name).toBe('Alice')
  })

  it('retourne une liste vide si le repo est vide', async () => {
    const repo: IChannelRepository = { getAll: vi.fn().mockResolvedValue([]) }
    const useCase = new GetChannels(repo)
    const result = await useCase.execute()
    expect(result).toEqual([])
  })
})
