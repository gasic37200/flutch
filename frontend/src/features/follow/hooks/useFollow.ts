import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../../api/client.ts'
import type { FollowState } from '../../../api/generated/FollowState.ts'
import type { Me } from '../../../api/generated/Me.ts'
import type { Pulse } from '../../../api/generated/Pulse.ts'
import type { SidebarChannel } from '../../../layouts/Sidebar.tsx'
import { HOME_POLL_MS, usePopularChannels } from '../../broadcast/hooks/useBroadcasts.ts'

/** 팔로우·언팔로우. 응답 값으로 스트리머 페이지·PULSE 캐시를 다시 받는다. */
export function useToggleFollow(userId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (follow: boolean) => api<FollowState>(`/users/${userId}/follow`, { method: follow ? 'PUT' : 'DELETE' }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ['user', userId] })
      void qc.invalidateQueries({ queryKey: ['pulse'] })
    },
  })
}

/** PULSE: 팔로우한 스트리머(라이브 먼저). 로그인 때만 켠다. */
export function usePulse(limit: number, enabled = true) {
  return useQuery({
    queryKey: ['pulse', limit],
    queryFn: () => api<Pulse>(`/pulse?limit=${limit}`),
    enabled,
    refetchInterval: HOME_POLL_MS,
  })
}

/** 사이드바 첫 패널: 로그인이고 팔로우가 있으면 PULSE(라이브 중인 팔로우), 아니면 인기 라이브(decisions §10). */
export function useSidebarPanel(me: Me | null): { title: string; channels: SidebarChannel[]; empty: string } {
  const popular = usePopularChannels()
  const pulse = usePulse(5, me !== null)
  if (me && pulse.data && pulse.data.total > 0) {
    const channels = pulse.data.live.map((b) => ({ broadcastId: b.id, userId: b.owner.id, nickname: b.owner.nickname, viewerCount: b.viewerCount }))
    return { title: 'PULSE', channels, empty: '팔로우한 채널이 방송 중이 아니에요' }
  }
  return { title: '인기 라이브', channels: popular, empty: '지금은 라이브 중인 방송이 없어요' }
}
