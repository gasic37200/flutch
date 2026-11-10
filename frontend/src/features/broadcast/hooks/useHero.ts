import { keepPreviousData, useQuery } from '@tanstack/react-query'
import { api } from '../../../api/client.ts'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { Hero } from '../../../api/generated/Hero.ts'
import { formatDuration, useNow } from '../../watch/hooks/useWatch.ts'
import { HOME_POLL_MS } from './useBroadcasts.ts'

export type HeroView = {
  hero: Hero
  /** super_clutch: 다음 방송까지 남은 시간 `M:SS` */
  slotLeft: string | null
  /** super_clutch: 지금 슬롯 진행률 0~100 */
  slotProgress: number
  /** super_clutch: 다음 순서 방송(혼자면 같은 방송) */
  next: Broadcast | null
  /** 진행 중 목록(회전 순서) + 남은 유효 시간 */
  active: { broadcast: Broadcast; left: string }[]
  /** 게이지 쿨다운이면 다음 클러치까지 남은 분(올림) */
  cooldownMin: number | null
}

/** 홈 히어로: 10초 폴링, 회전 슬롯이 그 전에 끝나면 전환 시각에 다시 조회(서버 1초 작업 여유 0.5초). */
// shortcut: 남은 시간을 기기 시계로 센다(서버와 몇 초 어긋날 수 있음), 어긋남이 문제되면 응답에 서버 시각을 싣는다.
export function useHero(): HeroView | undefined {
  const { data: hero } = useQuery({
    queryKey: ['clutch', 'hero'],
    queryFn: () => api<Hero>('/home/hero'),
    refetchInterval: (q) => {
      const next = q.state.data?.slot?.nextSwitchAt
      if (!next) return HOME_POLL_MS
      return Math.min(HOME_POLL_MS, Math.max(1_000, Date.parse(next) - Date.now() + 500))
    },
    placeholderData: keepPreviousData,
  })
  const cooldownUntil = hero?.gauge?.cooldownUntil
  const now = useNow(!!hero?.slot || !!cooldownUntil)
  if (!hero) return undefined

  const slot = hero.slot
  const start = slot ? Date.parse(slot.startedAt) : 0
  const end = slot ? Date.parse(slot.nextSwitchAt) : 0
  return {
    hero,
    slotLeft: slot ? formatDuration(end - now) : null,
    slotProgress: slot ? Math.min(100, Math.max(0, ((now - start) / (end - start)) * 100)) : 0,
    next: (hero.active[1] ?? hero.active[0])?.broadcast ?? null,
    active: hero.active.map((a) => ({ broadcast: a.broadcast, left: formatDuration(Date.parse(a.expiresAt) - now) })),
    cooldownMin: cooldownUntil ? Math.max(1, Math.ceil((Date.parse(cooldownUntil) - now) / 60_000)) : null,
  }
}
