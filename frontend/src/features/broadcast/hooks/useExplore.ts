import { useState, type FormEvent } from 'react'
import { useSearchParams } from 'react-router'
import type { Category } from '../../../api/generated/Category.ts'
import { isCategory } from '../categories.ts'
import { useSearchBroadcasts } from './useBroadcasts.ts'

export const MAX_QUERY = 30

/** 탐색: 카테고리·검색어를 URL 쿼리(`?category=&q=`)와 동기화한다(헤더 검색창도 `/explore?q=`로 온다). */
export function useExplore() {
  const [params, setParams] = useSearchParams()
  const rawCategory = params.get('category')
  const category: Category | undefined = isCategory(rawCategory) ? rawCategory : undefined
  const q = (params.get('q') ?? '').trim().slice(0, MAX_QUERY) || undefined
  const [typed, setDraft] = useState<string | null>(null)
  // 입력 전에는 URL의 q를 따라가서 헤더 검색으로 q가 바뀌어도 입력창이 맞는다
  const draft = typed ?? q ?? ''
  const results = useSearchBroadcasts({ category, q })

  const update = (next: { category?: Category; q?: string }) => {
    const p = new URLSearchParams()
    if (next.category) p.set('category', next.category)
    if (next.q) p.set('q', next.q)
    setParams(p)
  }

  return {
    category,
    q,
    draft,
    setDraft,
    setCategory: (c: Category | undefined) => update({ category: c, q }),
    search: (e: FormEvent) => {
      e.preventDefault()
      update({ category, q: draft.trim().slice(0, MAX_QUERY) || undefined })
      setDraft(null)
    },
    clear: () => {
      update({ category })
      setDraft(null)
    },
    results,
    items: results.data?.pages.flatMap((p) => p.items) ?? [],
  }
}
