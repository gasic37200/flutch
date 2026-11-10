import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { api } from '../../../api/client.ts'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { Page } from '../../../api/generated/Page.ts'
import type { UserProfile } from '../../../api/generated/UserProfile.ts'
import { useAuth } from '../../auth/hooks/useAuth.ts'
import { HOME_POLL_MS } from '../../broadcast/hooks/useBroadcasts.ts'
import { useRecordings } from '../../replay/hooks/useRecordings.ts'

export type StreamerTab = 'broadcasts' | 'replays' | 'bans'

/** 스트리머 페이지: 프로필·현재 방송·지난 방송·다시보기·본인 여부를 한 곳에서. */
export function useStreamer(userId: number | undefined) {
  const { me } = useAuth()
  const [tab, setTab] = useState<StreamerTab>('broadcasts')
  const profile = useQuery({
    queryKey: ['user', userId],
    queryFn: () => api<UserProfile>(`/users/${userId}`),
    enabled: userId !== undefined,
    retry: false,
    refetchInterval: HOME_POLL_MS, // 방송 시작·종료를 따라간다
  })
  const past = useInfiniteQuery({
    queryKey: ['user', userId, 'broadcasts'],
    queryFn: ({ pageParam }) => api<Page<Broadcast>>(`/users/${userId}/broadcasts?limit=20${pageParam ? `&cursor=${pageParam}` : ''}`),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.nextCursor ?? undefined,
    enabled: userId !== undefined && profile.isSuccess,
  })
  const replays = useRecordings({ userId })
  return { me, isSelf: me !== null && me.id === userId, tab, setTab, profile, past, replays }
}
