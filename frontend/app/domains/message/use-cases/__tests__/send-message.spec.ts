import { describe, it, expect, vi } from 'vitest'
import { SendMessage } from '../SendMessage'
import type { IMessageRepository } from '../../repositories/IMessageRepository'

function mockRepo(overrides: Partial<IMessageRepository> = {}): IMessageRepository {
  return {
    getByChannel: vi.fn().mockResolvedValue([]),
    send: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  }
}

describe('SendMessage', () => {
  it('appelle repo.send avec le texte trimé', async () => {
    const repo = mockRepo()
    const useCase = new SendMessage(repo)
    await useCase.execute('channel-1', '  Bonjour  ')
    expect(repo.send).toHaveBeenCalledWith('channel-1', 'Bonjour')
  })

  it('lève une erreur si le texte est vide', async () => {
    const repo = mockRepo()
    const useCase = new SendMessage(repo)
    await expect(useCase.execute('channel-1', '   ')).rejects.toThrow('empty')
  })

  it('lève une erreur si le texte ne contient que des espaces', async () => {
    const repo = mockRepo()
    const useCase = new SendMessage(repo)
    await expect(useCase.execute('channel-1', '\t\n')).rejects.toThrow()
  })
})
