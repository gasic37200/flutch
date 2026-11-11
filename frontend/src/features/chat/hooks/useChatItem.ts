import { useState } from 'react'
import { ApiRequestError } from '../../../api/client.ts'
import type { Chat } from '../../../api/generated/Chat.ts'
import { useAuth } from '../../auth/hooks/useAuth.ts'
import type { ChatApi } from './useChat.ts'

const EDIT_WINDOW_MS = 5 * 60_000

const message = (e: unknown) => (e instanceof ApiRequestError ? e.message : '네트워크 연결을 확인하고 다시 시도해 주세요.')

/** 메시지 한 줄의 수정·삭제 상태. 권한은 서버가 다시 검사하고, 여기서는 메뉴에 보일지만 정한다. */
export function useChatItem(chat: Pick<ChatApi, 'edit' | 'remove'>, item: Chat, { isOwner, live }: { isOwner: boolean; live: boolean }) {
  const { me } = useAuth()
  const mine = !!me && me.id === item.author.id
  const [menu, setMenu] = useState(false)
  const [editable, setEditable] = useState(false) // 메뉴를 연 시점 기준 5분 이내
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState('')
  const [busy, setBusy] = useState(false)
  const [confirming, setConfirming] = useState(false)
  const [error, setError] = useState<string>()
  const [popup, setPopup] = useState(false) // 닉네임을 눌러 연 시청자 창

  // 욕설은 서버가 `***`로 가려 내려 준다(원문 비공개). 그대로 수정창에 넣고 저장하면 원문이 `***`로 덮이므로 빈 칸에서 시작한다.
  const masked = !!item.body?.includes('***')
  const canDelete = !item.deleted && (mine || isOwner)
  const hasMenu = !item.deleted && (mine || isOwner)

  const run = async (f: () => Promise<void>, done: () => void) => {
    setBusy(true)
    setError(undefined)
    try {
      await f()
      done()
    } catch (e) {
      setError(message(e))
      setConfirming(false)
    } finally {
      setBusy(false)
    }
  }

  return {
    popup,
    openPopup: () => setPopup(true),
    closePopup: () => setPopup(false),
    /** 방송 소유자 본인 id(차단 메뉴를 열 수 있을 때만) */
    ownerId: isOwner && me ? me.id : undefined,
    hasMenu,
    menu,
    toggleMenu: () => {
      setEditable(mine && live && Date.now() - Date.parse(item.createdAt) < EDIT_WINDOW_MS)
      setMenu((m) => !m)
    },
    canEdit: editable,
    canDelete,
    editing,
    masked,
    draft,
    setDraft,
    startEdit: () => {
      setDraft(masked ? '' : (item.body ?? ''))
      setError(undefined)
      setMenu(false)
      setEditing(true)
    },
    cancelEdit: () => {
      setEditing(false)
      setError(undefined)
    },
    saveEdit: () => {
      const body = draft.trim()
      if (!body || busy) return
      if (body === item.body) return setEditing(false)
      void run(() => chat.edit(item.id, body), () => setEditing(false))
    },
    busy,
    error,
    confirming,
    askDelete: () => {
      setMenu(false)
      setConfirming(true)
    },
    cancelDelete: () => setConfirming(false),
    confirmDelete: () => void run(() => chat.remove(item.id), () => setConfirming(false)),
  }
}
