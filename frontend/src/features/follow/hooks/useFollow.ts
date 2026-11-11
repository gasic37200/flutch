import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../../api/client.ts'
import type { FollowState } from '../../../api/generated/FollowState.ts'
import type { Me } from '../../../api/generated/Me.ts'
import type { Pulse } from '../../../api/generated/Pulse.ts'
import type { SidebarPanel } from '../../../layouts/Sidebar.tsx'
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

/** 사이드바 첫 패널(features/09): 비로그인·팔로우 0개는 인기 라이브 + 안내, 팔로우가 있으면 PULSE(라이브 → 오프라인, 최대 10개). */
export function useSidebarPanel(me: Me | null): SidebarPanel {
  const popular = usePopularChannels()
  const pulse = usePulse(10, me !== null)
  if (me === null) return { title: '인기 라이브', channels: popular, offline: [], empty: '지금은 라이브 중인 방송이 없어요', note: '로그인하면 내 PULSE가 보여요' }
  if (pulse.data && pulse.data.total > 0) {
    const live = pulse.data.live.map((b) => ({ broadcastId: b.id, userId: b.owner.id, nickname: b.owner.nickname, viewerCount: b.viewerCount }))
    const offline = pulse.data.offline.slice(0, Math.max(0, 10 - live.length)).map((o) => ({ userId: o.user.id, nickname: o.user.nickname }))
    return { title: 'PULSE', channels: live.slice(0, 10), offline, empty: '팔로우한 채널이 방송 중이 아니에요' }
  }
  if (pulse.data) return { title: 'PULSE', channels: popular.slice(0, 3), offline: [], empty: '지금은 라이브 중인 방송이 없어요', note: '스트리머를 팔로우하면 여기서 맥박이 뛰어요' }
  return { title: '인기 라이브', channels: popular, offline: [], empty: '지금은 라이브 중인 방송이 없어요' }
}
