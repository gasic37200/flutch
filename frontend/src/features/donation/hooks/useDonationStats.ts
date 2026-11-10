import { useInfiniteQuery } from '@tanstack/react-query'
import { api } from '../../../api/client.ts'
import type { DonationStats } from '../../../api/generated/DonationStats.ts'

/** 스트리머 본인의 받은 비트 통계(누적·최근 7일·방송별). 방송별 목록은 후원 내역의 방송 선택에도 쓴다. */
export function useDonationStats() {
  const query = useInfiniteQuery({
    queryKey: ['donation-stats'],
    queryFn: ({ pageParam }) => api<DonationStats>(`/me/donation-stats?limit=20${pageParam ? `&cursor=${pageParam}` : ''}`),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.nextCursor ?? undefined,
  })
  const first = query.data?.pages[0]
  return {
    query,
    items: query.data?.pages.flatMap((p) => p.items) ?? [],
    totalReceived: first?.totalReceived ?? 0,
    last7Days: first?.last7Days ?? 0,
  }
}
