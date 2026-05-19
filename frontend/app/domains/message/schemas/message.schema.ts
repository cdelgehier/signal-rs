import { z } from 'zod'

export const ReceiptStatusSchema = z.enum(['sent', 'delivered', 'read']).nullable()

export const MessageSchema = z.object({
  id: z.number(),
  sender_id: z.string(),
  sender_name: z.string(),
  text: z.string().nullable(),
  timestamp: z.number(),
  is_outgoing: z.boolean(),
  receipt: ReceiptStatusSchema,
})

export type MessageRaw = z.infer<typeof MessageSchema>
