import { describe, it, expect } from 'vitest'
import { ChannelSchema } from '../channel.schema'

describe('ChannelSchema', () => {
  it('valide un channel bien formé', () => {
    const raw = {
      id: 'uuid-1',
      name: 'Alice',
      last_message: 'Salut',
      last_message_time: 1_700_000_000,
      unread_count: 2,
      is_group: false,
    }
    const result = ChannelSchema.safeParse(raw)
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.data.unread_count).toBe(2)
    }
  })

  it('valide last_message null', () => {
    const raw = {
      id: 'uuid-2',
      name: 'Bob',
      last_message: null,
      last_message_time: null,
      unread_count: 0,
      is_group: true,
    }
    expect(ChannelSchema.safeParse(raw).success).toBe(true)
  })

  it('rejette un unread_count négatif', () => {
    const raw = {
      id: 'uuid-3',
      name: 'Eve',
      last_message: null,
      last_message_time: null,
      unread_count: -1,
      is_group: false,
    }
    expect(ChannelSchema.safeParse(raw).success).toBe(false)
  })

  it('rejette un id manquant', () => {
    const raw = { name: 'X', last_message: null, last_message_time: null, unread_count: 0, is_group: false }
    expect(ChannelSchema.safeParse(raw).success).toBe(false)
  })
})
