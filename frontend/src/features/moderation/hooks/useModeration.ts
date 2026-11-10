import { useInfiniteQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { api, ApiRequestError } from '../../../api/client.ts'
import type { Ban } from '../../../api/generated/Ban.ts'
import type { BanDuration } from '../../../api/generated/BanDuration.ts'
import type { BanRequest } from '../../../api/generated/BanRequest.ts'
import type { ChatPage } from '../../../api/generated/ChatPage.ts'
import type { Page } from '../../../api/generated/Page.ts'
import type { UserSummary } from '../../../api/generated/UserSummary.ts'
import { useToast } from '../../../hooks/useToast.ts'

export const TIMEOUTS: { duration: BanDuration; label: string }[] = [
  { duration: '1m', label: '1분' },
  { duration: '10m', label: '10분' },
  { duration: '1h', label: '1시간' },
]

const bansKey = (ownerId: number | undefined) => ['bans', ownerId]

/** 남은 시간 문구("7분 남음"). 영구면 "영구". */
export function leftText(ban: Pick<Ban, 'permanent' | 'until'>, now: number) {
  if (ban.permanent || !ban.until) return '영구 밴'
  const sec = Math.max(0, Math.ceil((Date.parse(ban.until) - now) / 1000))
  return sec >= 60 ? `임시차단 · ${Math.ceil(sec / 60)}분 남음` : `임시차단 · ${sec}초 남음`
}

/** 차단·해제 요청. 성공·실패는 토스트로 알리고, 성공하면 true. */
export function useBanActions(ownerId: number) {
  const qc = useQueryClient()
  const toast = useToast()
  const [busy, setBusy] = useState(false)

  const run = async (f: () => Promise<unknown>, done: string) => {
    setBusy(true)
    try {
      await f()
      void qc.invalidateQueries({ queryKey: bansKey(ownerId) })
      toast({ tone: 'success', message: done })
      return true
    } catch (e) {
      toast(
        e instanceof ApiRequestError
          ? { tone: 'error', message: e.message, requestId: e.requestId }
          : { tone: 'error', message: '네트워크 연결을 확인하고 다시 시도해 주세요.' },
      )
      return false
    } finally {
      setBusy(false)
    }
  }

  return {
    busy,
    ban: (user: UserSummary, duration: BanDuration) =>
      run(
        () => api<Ban>(`/users/${ownerId}/bans/${user.id}`, { method: 'PUT', json: { duration } satisfies BanRequest }),
        duration === 'permanent' ? `${user.nickname}님을 영구 차단했어요` : `${user.nickname}님을 ${TIMEOUTS.find((t) => t.duration === duration)?.label} 차단했어요`,
      ),
    unban: (user: UserSummary) => run(() => api<void>(`/users/${ownerId}/bans/${user.id}`, { method: 'DELETE' }), `${user.nickname}님의 차단을 해제했어요`),
  }
}

/** 시청자가 내 방송들에서 쓴 채팅(최신 50개 + 이전 기록). 오래된 → 최신 순으로 합쳐 돌려준다. */
export function useViewerChats(ownerId: number, userId: number, enabled: boolean) {
  const q = useInfiniteQuery({
    queryKey: ['viewerChats', ownerId, userId],
    enabled,
    queryFn: ({ pageParam }) => api<ChatPage>(`/users/${ownerId}/viewers/${userId}/chats${pageParam ? `?before=${pageParam}` : ''}`),
    initialPageParam: undefined as number | undefined,
    getNextPageParam: (last) => (last.hasMore ? last.items[0]?.id : undefined),
  })
  const items = q.data ? [...q.data.pages].reverse().flatMap((p) => p.items) : []
  return { items, query: q }
}

/** 차단 목록(유효한 차단만) + 남은 시간 갱신 + 해제. */
export function useBanList(ownerId: number) {
  const q = useInfiniteQuery({
    queryKey: bansKey(ownerId),
    queryFn: ({ pageParam }) => api<Page<Ban>>(`/users/${ownerId}/bans?limit=20${pageParam ? `&cursor=${pageParam}` : ''}`),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.nextCursor ?? undefined,
  })
  const actions = useBanActions(ownerId)
  const [now, setNow] = useState(() => Date.now())
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 15_000)
    return () => clearInterval(t)
  }, [])
  // 만료된 임시차단은 서버가 다음 조회부터 빼므로, 그 전에도 화면에서 숨긴다
  const items = (q.data?.pages.flatMap((p) => p.items) ?? []).filter((b) => b.permanent || !b.until || Date.parse(b.until) > now)
  return { query: q, items, now, ...actions }
}
