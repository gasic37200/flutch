import { useState } from 'react'
import type { Category } from '../../../api/generated/Category.ts'
import { useLiveBroadcasts } from './useBroadcasts.ts'

/** 홈: 메인 라이브(1위) + 추천 라이브 채널(2~4위) + 카테고리 칩으로 거른 그리드. 둘 다 10초 폴링. */
export function useHome() {
  const [category, setCategory] = useState<Category | undefined>()
  const grid = useLiveBroadcasts({ category, limit: 50 })
  // 사이드바와 같은 params라 캐시를 공유한다(요청 1개)
  const all = useLiveBroadcasts({ limit: 5 })
  const top = all.data?.items ?? []
  return { category, setCategory, grid, main: top[0], recommended: top.slice(1, 4) }
}
