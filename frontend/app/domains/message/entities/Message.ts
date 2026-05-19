export type ReceiptStatus = 'sent' | 'delivered' | 'read'

export interface Message {
  id: number
  senderId: string
  senderName: string
  text: string | null
  timestamp: number
  isOutgoing: boolean
  receipt: ReceiptStatus | null
}
