import { useInfiniteQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect } from 'react'
import { api } from '../../../api/client.ts'
import type { DonationList } from '../../../api/generated/DonationList.ts'
import type { OnEvent } from '../../live/hooks/useBroadcastEvents.ts'

export const donationsKey = (broadcastId: number | undefined) => ['donations', broadcastId]
const LIVE_REFRESH_MS = 30_000 // 유료 클러치 합계는 이벤트가 없어 라이브 중에는 주기적으로 다시 받는다

/**
 * 한 방송의 후원 합계·목록. 공개 응답은 익명이 가려지고, 방송 소유자 세션이면 서버가 실명을 준다.
 * `onEvent`를 주면 `donation.created`마다 다시 조회한다(실시간 반영).
 */
export function useDonationList(broadcastId: number | undefined, opts: { onEvent?: OnEvent; live?: boolean } = {}) {
  const { onEvent, live } = opts
  const qc = useQueryClient()
  const query = useInfiniteQuery({
    queryKey: donationsKey(broadcastId),
    enabled: broadcastId !== undefined,
    queryFn: ({ pageParam }) => api<DonationList>(`/broadcasts/${broadcastId}/donations?limit=20${pageParam ? `&cursor=${pageParam}` : ''}`),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.nextCursor ?? undefined,
    refetchInterval: live ? LIVE_REFRESH_MS : false,
  })
  useEffect(() => onEvent?.('donation.created', () => void qc.invalidateQueries({ queryKey: donationsKey(broadcastId) })), [onEvent, qc, broadcastId])

  const first = query.data?.pages[0]
  return {
    query,
    items: query.data?.pages.flatMap((p) => p.items) ?? [],
    total: first?.total ?? 0,
    count: first?.count ?? 0,
    clutchTotal: first?.clutchTotal ?? 0,
  }
}
