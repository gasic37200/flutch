import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { api } from '../../../api/client.ts'
import type { Chat } from '../../../api/generated/Chat.ts'
import type { ChatPage } from '../../../api/generated/ChatPage.ts'
import type { Donation } from '../../../api/generated/Donation.ts'
import { useAuth } from '../../auth/hooks/useAuth.ts'
import type { OnEvent } from '../../live/hooks/useBroadcastEvents.ts'

/** `useBroadcastEvents`가 돌려준 값. WebSocket은 화면당 하나라 같은 연결에 구독한다. */
export type ChatEvents = { onEvent: OnEvent; connected: boolean }
export type ChatApi = ReturnType<typeof useChat>

const NONE: Chat[] = []

/** 차단 알림 줄. 저장되지 않는 실시간 전용이라 이 화면 상태로만 들고, `afterId` 채팅 바로 뒤에 그린다(없으면 맨 앞). */
export type SystemLine = { key: number; afterId: number | null; text: string; donation?: Donation }
let nextSystemKey = 1

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
  const meId = useAuth().me?.id
  const key = useMemo(() => ['chats', broadcastId], [broadcastId])
  const [systems, setSystems] = useState<SystemLine[]>([])
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
      onEvent('chat.system', ({ kind, user, until }) => {
        const mins = Math.max(1, Math.round((Date.parse(until ?? '') - Date.now()) / 60_000))
        const text = kind === 'ban' ? `${user.nickname}님이 차단되었어요` : `${user.nickname}님의 채팅이 ${mins}분간 제한되었어요`
        const afterId = qc.getQueryData<ChatPage>(key)?.items.at(-1)?.id ?? null
        setSystems((s) => [...s.slice(-49), { key: nextSystemKey++, afterId, text }])
        if (kind === 'ban') setPage((d) => ({ ...d, items: d.items.map((x) => (x.author.id === user.id ? { ...x, body: null, deleted: true } : x)) }))
        // 차단된 본인 화면은 상세의 viewer.ban을 다시 받아 입력창을 바꾼다
        if (user.id === meId) void qc.invalidateQueries({ queryKey: ['broadcast', broadcastId] })
      }),
      onEvent('donation.created', ({ donation }) => {
        // 후원 강조 줄도 저장되지 않는 실시간 줄이라 차단 알림 줄과 같은 방식으로 둔다(익명 처리는 서버가 이벤트 형태로 이미 함)
        const afterId = qc.getQueryData<ChatPage>(key)?.items.at(-1)?.id ?? null
        const name = donation.donor?.nickname ?? '익명의 후원자'
        setSystems((s) => [...s.slice(-49), { key: nextSystemKey++, afterId, text: `${name}님이 ${donation.amount}비트를 후원했어요`, donation }])
      }),
      onEvent('resync', () => void qc.invalidateQueries({ queryKey: key })),
    ]
    return () => offs.forEach((off) => off())
  }, [onEvent, upsert, markDeleted, setPage, qc, key, meId, broadcastId])

  // 끊겼다가 다시 연결되면 그 사이 놓친 채팅을 보충한다(첫 연결은 처음 조회가 맡는다)
  const wasConnected = useRef(false)
  const [dropped, setDropped] = useState(false) // 한 번 연결된 뒤 끊겨 있는 동안 true(안내 표시용)
  useEffect(() => {
    setDropped(wasConnected.current && !connected)
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
    reconnecting: dropped,
    items: data?.items ?? NONE,
    systems,
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
