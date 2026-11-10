import { useState } from 'react'
import type { Category } from '../../../api/generated/Category.ts'
import { useLiveBroadcasts } from './useBroadcasts.ts'
import { useHero } from './useHero.ts'

/** 홈: 메인 히어로(SUPER CLUTCH 회전·게이지 1위) + 추천 라이브 채널(히어로 뺀 상위 3) + 카테고리 칩으로 거른 그리드. 모두 10초 폴링. */
export function useHome() {
  const [category, setCategory] = useState<Category | undefined>()
  const grid = useLiveBroadcasts({ category, limit: 50 })
  // 사이드바와 같은 params라 캐시를 공유한다(요청 1개)
  const all = useLiveBroadcasts({ limit: 5 })
  const hero = useHero()
  const heroId = hero?.hero.broadcast?.id
  const recommended = (all.data?.items ?? []).filter((b) => b.id !== heroId).slice(0, 3)
  return { category, setCategory, grid, hero, recommended }
}
