import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { api } from '../../../api/client.ts'
import type { Chat } from '../../../api/generated/Chat.ts'
import type { ChatPage } from '../../../api/generated/ChatPage.ts'
import type { OnEvent } from '../../live/hooks/useBroadcastEvents.ts'

/** `useBroadcastEvents`가 돌려준 값. WebSocket은 화면당 하나라 같은 연결에 구독한다. */
export type ChatEvents = { onEvent: OnEvent; connected: boolean }
export type ChatApi = ReturnType<typeof useChat>

const NONE: Chat[] = []

/** 다시 조회한 최신 페이지에, 이미 불러온 더 오래된 기록을 이어 붙인다. */
function mergeLatest(prev: ChatPage, fresh: ChatPage): ChatPage {
  const firstId = fresh.items[0]?.id
  const older = firstId === undefined ? [] : prev.items.filter((c) => c.id < firstId)
  return { items: [...older, ...fresh.items], hasMore: older.length > 0 ? prev.hasMore : fresh.hasMore }
}

/**
 * 한 방송의 채팅 목록(최근 50개 + 위로 더 불러오기)과 실시간 반영. 작성·수정·삭제는 서버 에러를 그대로 던진다.
 * shortcut: 이전 기록 불러오기 실패는 재시도 버튼만 있고 자동 재시도는 없다.
 */
export function useChat(broadcastId: number | undefined, { onEvent, connected }: ChatEvents) {
  const qc = useQueryClient()
  const key = useMemo(() => ['chats', broadcastId], [broadcastId])
  const [loadingMore, setLoadingMore] = useState(false)
  const [moreFailed, setMoreFailed] = useState(false)

  const query = useQuery({
    queryKey: key,
    enabled: broadcastId !== undefined,
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    queryFn: async () => {
      const fresh = await api<ChatPage>(`/broadcasts/${broadcastId}/chats`)
      const prev = qc.getQueryData<ChatPage>(key)
      return prev ? mergeLatest(prev, fresh) : fresh
    },
  })

  const setPage = useCallback((f: (d: ChatPage) => ChatPage) => qc.setQueryData<ChatPage>(key, (d) => (d ? f(d) : d)), [qc, key])
  const upsert = useCallback(
    (c: Chat) =>
      setPage((d) => ({ ...d, items: d.items.some((x) => x.id === c.id) ? d.items.map((x) => (x.id === c.id ? c : x)) : [...d.items, c] })),
    [setPage],
  )
  const markDeleted = useCallback(
    (id: number) => setPage((d) => ({ ...d, items: d.items.map((x) => (x.id === id ? { ...x, body: null, deleted: true } : x)) })),
    [setPage],
  )

  useEffect(() => {
    const offs = [
      onEvent('chat.created', upsert),
      onEvent('chat.updated', upsert),
      onEvent('chat.deleted', ({ id }) => markDeleted(id)),
      onEvent('resync', () => void qc.invalidateQueries({ queryKey: key })),
    ]
    return () => offs.forEach((off) => off())
  }, [onEvent, upsert, markDeleted, qc, key])

  // 끊겼다가 다시 연결되면 그 사이 놓친 채팅을 보충한다(첫 연결은 처음 조회가 맡는다)
  const wasConnected = useRef(false)
  useEffect(() => {
    if (connected && wasConnected.current) void qc.invalidateQueries({ queryKey: key })
    if (connected) wasConnected.current = true
  }, [connected, qc, key])

  const data = query.data
  const loadMore = async () => {
    const first = data?.items[0]
    if (!data?.hasMore || !first || loadingMore) return
    setLoadingMore(true)
    setMoreFailed(false)
    try {
      const page = await api<ChatPage>(`/broadcasts/${broadcastId}/chats?before=${first.id}`)
      setPage((d) => ({ items: [...page.items.filter((c) => c.id < (d.items[0]?.id ?? Infinity)), ...d.items], hasMore: page.hasMore }))
    } catch {
      setMoreFailed(true)
    } finally {
      setLoadingMore(false)
    }
  }

  return {
    items: data?.items ?? NONE,
    hasMore: data?.hasMore ?? false,
    loading: query.isPending,
    error: query.error,
    refetch: query.refetch,
    loadMore,
    loadingMore,
    moreFailed,
    send: async (body: string) => {
      const chat = await api<Chat>(`/broadcasts/${broadcastId}/chats`, { method: 'POST', json: { body } })
      upsert(chat)
      return chat
    },
    edit: async (id: number, body: string) => {
      upsert(await api<Chat>(`/chats/${id}`, { method: 'PATCH', json: { body } }))
    },
    remove: async (id: number) => {
      await api<void>(`/chats/${id}`, { method: 'DELETE' })
      markDeleted(id)
    },
  }
}
