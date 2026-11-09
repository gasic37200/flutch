import { useState } from 'react'
import type { Category } from '../../../api/generated/Category.ts'
import { useLiveBroadcasts } from './useBroadcasts.ts'

/** 홈: 추천 라이브 채널 상위 3개 + 카테고리 칩으로 거른 그리드. 둘 다 10초 폴링. */
export function useHome() {
  const [category, setCategory] = useState<Category | undefined>()
  const grid = useLiveBroadcasts({ category, limit: 50 })
  // 사이드바와 같은 params라 캐시를 공유한다(요청 1개)
  const all = useLiveBroadcasts({ limit: 5 })
  return { category, setCategory, grid, recommended: (all.data?.items ?? []).slice(0, 3) }
}
