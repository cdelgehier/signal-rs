import { describe, it, expect } from 'vitest'
import { MessageSchema } from '../message.schema'

describe('MessageSchema', () => {
  it('valide un message entrant', () => {
    const raw = {
      id: 42,
      sender_id: 'uuid-alice',
      sender_name: 'Alice',
      text: 'Bonjour !',
      timestamp: 1_700_000_001_000,
      is_outgoing: false,
      receipt: null,
    }
    const result = MessageSchema.safeParse(raw)
    expect(result.success).toBe(true)
  })

  it('valide un message sortant avec receipt read', () => {
    const raw = {
      id: 43,
      sender_id: 'me',
      sender_name: 'Moi',
      text: 'Salut',
      timestamp: 1_700_000_002_000,
      is_outgoing: true,
      receipt: 'read',
    }
    const result = MessageSchema.safeParse(raw)
    expect(result.success).toBe(true)
    if (result.success) expect(result.data.receipt).toBe('read')
  })

  it('rejette un receipt invalide', () => {
    const raw = {
      id: 44,
      sender_id: 'me',
      sender_name: 'Moi',
      text: null,
      timestamp: 1_700_000_003_000,
      is_outgoing: true,
      receipt: 'unknown_status',
    }
    expect(MessageSchema.safeParse(raw).success).toBe(false)
  })
})
