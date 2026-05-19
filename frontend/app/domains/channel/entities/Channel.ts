export interface Channel {
  id: string
  name: string
  lastMessage: string | null
  lastMessageTime: number | null
  unreadCount: number
  isGroup: boolean
}
