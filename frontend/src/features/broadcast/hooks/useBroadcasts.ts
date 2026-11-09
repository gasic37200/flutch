import { keepPreviousData, useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../../api/client.ts'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import type { Category } from '../../../api/generated/Category.ts'
import type { CreateBroadcastRequest } from '../../../api/generated/CreateBroadcastRequest.ts'
import type { Page } from '../../../api/generated/Page.ts'
import type { UpdateBroadcastRequest } from '../../../api/generated/UpdateBroadcastRequest.ts'
import type { SidebarChannel } from '../../../layouts/Sidebar.tsx'

/** 홈은 WebSocket 없이 10초마다 다시 가져온다(architecture §6). */
export const HOME_POLL_MS = 10_000

type Params = { category?: Category; q?: string; limit?: number; cursor?: string }

function qs(params: Params) {
  const s = new URLSearchParams()
  for (const [k, v] of Object.entries(params)) if (v !== undefined && v !== '') s.set(k, String(v))
  const text = s.toString()
  return text ? `?${text}` : ''
}

/** LIVE 방송 목록(홈 그리드·사이드바·추천). 같은 params는 캐시를 공유한다. */
export function useLiveBroadcasts(params: Omit<Params, 'cursor'>) {
  return useQuery({
    queryKey: ['broadcast', 'live', params],
    queryFn: () => api<Page<Broadcast>>(`/broadcasts${qs(params)}`),
    refetchInterval: HOME_POLL_MS,
    placeholderData: keepPreviousData,
  })
}

/** 탐색·검색: 커서 페이지로 "더 보기". */
export function useSearchBroadcasts(params: Omit<Params, 'cursor' | 'limit'>) {
  return useInfiniteQuery({
    queryKey: ['broadcast', 'search', params],
    queryFn: ({ pageParam }) => api<Page<Broadcast>>(`/broadcasts${qs({ ...params, limit: 20, cursor: pageParam })}`),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.nextCursor ?? undefined,
  })
}

/** 사이드바 "인기 라이브" 5개. */
export function usePopularChannels(): SidebarChannel[] {
  const { data } = useLiveBroadcasts({ limit: 5 })
  return (data?.items ?? []).map((b) => ({ broadcastId: b.id, userId: b.owner.id, nickname: b.owner.nickname, viewerCount: b.viewerCount }))
}

/** 내 방송(미숨김, 최신 등록순). 로그인 화면에서만 켠다. */
export function useMyBroadcasts(enabled = true) {
  return useQuery({
    queryKey: ['broadcast', 'mine'],
    queryFn: () => api<Page<Broadcast>>('/me/broadcasts?limit=50'),
    enabled,
  })
}

export function useBroadcast(id: number | undefined) {
  return useQuery({
    queryKey: ['broadcast', id],
    queryFn: () => api<BroadcastDetail>(`/broadcasts/${id}`),
    enabled: id !== undefined,
    retry: false,
  })
}

// 방송이 바뀌면 목록·상세 캐시를 모두 다시 받는다.
function useInvalidateBroadcasts() {
  const qc = useQueryClient()
  return () => qc.invalidateQueries({ queryKey: ['broadcast'] })
}

export function useCreateBroadcast() {
  const invalidate = useInvalidateBroadcasts()
  return useMutation({
    mutationFn: (json: CreateBroadcastRequest) => api<BroadcastDetail>('/broadcasts', { method: 'POST', json }),
    onSuccess: invalidate,
  })
}

export function useUpdateBroadcast(id: number | undefined) {
  const invalidate = useInvalidateBroadcasts()
  return useMutation({
    mutationFn: (json: UpdateBroadcastRequest) => api<BroadcastDetail>(`/broadcasts/${id}`, { method: 'PATCH', json }),
    onSuccess: invalidate,
  })
}

export function useDeleteBroadcast() {
  const invalidate = useInvalidateBroadcasts()
  return useMutation({
    mutationFn: (id: number) => api<void>(`/broadcasts/${id}`, { method: 'DELETE' }),
    onSuccess: invalidate,
  })
}
