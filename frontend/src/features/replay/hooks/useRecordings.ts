import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { api } from '../../../api/client.ts'
import type { Category } from '../../../api/generated/Category.ts'
import type { ChatPage } from '../../../api/generated/ChatPage.ts'
import type { Page } from '../../../api/generated/Page.ts'
import type { RecordingCard } from '../../../api/generated/RecordingCard.ts'
import type { RecordingDetail } from '../../../api/generated/RecordingDetail.ts'
import { useGuestId } from '../../live/hooks/useClientIds.ts'

type Params = { category?: Category; q?: string; userId?: number; limit?: number }

/** 다시보기 목록(ready·최신순). 커서 페이지로 "더 보기". */
export function useRecordings({ limit = 20, ...filter }: Params) {
  return useInfiniteQuery({
    queryKey: ['recording', 'list', filter, limit],
    queryFn: ({ pageParam }) => {
      const s = new URLSearchParams({ limit: String(limit) })
      for (const [k, v] of Object.entries(filter)) if (v !== undefined && v !== '') s.set(k, String(v))
      if (pageParam) s.set('cursor', pageParam)
      return api<Page<RecordingCard>>(`/recordings?${s}`)
    },
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.nextCursor ?? undefined,
  })
}

/** 다시보기 상세. 처리 중이면 10초마다 다시 확인해 준비되면 영상으로 바뀐다. */
export function useRecording(id: number | undefined) {
  return useQuery({
    queryKey: ['recording', id],
    queryFn: () => api<RecordingDetail>(`/recordings/${id}`),
    enabled: id !== undefined,
    retry: false,
    refetchInterval: (q) => (q.state.data?.status === 'recording' ? 10_000 : false),
  })
}

const counted = new Set<number>() // 같은 탭 세션에서 같은 영상의 조회수는 한 번만 보낸다(서버도 KST 하루 1회로 거른다)

/** 재생을 시작할 때 호출하는 조회수 1회 증가. */
export function useCountView(id: number | undefined) {
  const guestId = useGuestId()
  return () => {
    if (id === undefined || counted.has(id)) return
    counted.add(id)
    api<void>(`/recordings/${id}/views`, { method: 'POST', json: { guestId } }).catch(() => counted.delete(id))
  }
}

/** 방송 채팅 전체 기록(읽기 전용). 처음엔 최신 50개, "이전 채팅"으로 더 거슬러 오른다. */
export function useReplayChats(broadcastId: number | undefined) {
  const q = useInfiniteQuery({
    queryKey: ['recording', 'chats', broadcastId],
    queryFn: ({ pageParam }) => api<ChatPage>(`/broadcasts/${broadcastId}/chats?limit=50${pageParam ? `&before=${pageParam}` : ''}`),
    initialPageParam: undefined as number | undefined,
    getNextPageParam: (last) => (last.hasMore && last.items.length > 0 ? last.items[0].id : undefined),
    enabled: broadcastId !== undefined,
    retry: false,
  })
  return { ...q, items: q.data ? [...q.data.pages].reverse().flatMap((p) => p.items) : [] }
}
