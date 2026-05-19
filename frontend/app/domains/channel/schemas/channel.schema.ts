import { z } from 'zod'

export const ChannelSchema = z.object({
  id: z.string(),
  name: z.string(),
  last_message: z.string().nullable(),
  last_message_time: z.number().nullable(),
  unread_count: z.number().int().nonnegative(),
  is_group: z.boolean(),
})

export type ChannelRaw = z.infer<typeof ChannelSchema>
