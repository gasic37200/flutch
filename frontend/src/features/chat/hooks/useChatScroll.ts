import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react'

const NEAR_BOTTOM_PX = 48
const NEAR_TOP_PX = 24

/** 목록 스크롤: 맨 아래에 있을 때만 새 메시지를 따라가고, 위를 보는 중이면 "새 메시지 n개"로 알린다. 위로 불러온 기록은 보던 자리를 유지한다. */
export function useChatScroll(chat: { items: { id: number }[]; hasMore: boolean; loadMore: () => unknown }) {
  const { items, hasMore, loadMore } = chat
  const ref = useRef<HTMLDivElement>(null)
  const contentRef = useRef<HTMLDivElement>(null)
  const stuck = useRef(true)
  const prev = useRef<{ first?: number; last?: number; height: number }>({ height: 0 })
  const pinnedTop = useRef(-1) // 코드로 맨 아래에 붙인 위치. 그 직후 도착하는 scroll 이벤트는 사용자가 위로 올린 것이 아니다
  const [unseen, setUnseen] = useState(0)

  const pin = (el: HTMLElement) => {
    el.scrollTop = el.scrollHeight
    pinnedTop.current = el.scrollTop
  }

  const toBottom = useCallback(() => {
    if (ref.current) pin(ref.current)
    stuck.current = true
    setUnseen(0)
  }, [])

  const onScroll = () => {
    const el = ref.current
    if (!el || el.scrollTop === pinnedTop.current) return
    stuck.current = el.scrollHeight - el.scrollTop - el.clientHeight < NEAR_BOTTOM_PX
    if (stuck.current) setUnseen(0)
    if (el.scrollTop < NEAR_TOP_PX && hasMore) void loadMore()
  }

  useLayoutEffect(() => {
    const el = ref.current
    if (!el) return
    const first = items[0]?.id
    const last = items.at(-1)?.id
    const p = prev.current
    if (last !== undefined && (p.last === undefined || last > p.last)) {
      if (stuck.current || p.last === undefined) pin(el)
      else {
        const seen = p.last
        setUnseen((n) => n + items.filter((c) => c.id > seen).length)
      }
    } else if (first !== undefined && p.first !== undefined && first < p.first) {
      el.scrollTop += el.scrollHeight - p.height
    }
    prev.current = { first, last, height: el.scrollHeight }
  }, [items])

  // 입력창이 나타나거나 글꼴이 늦게 적용돼 높이가 바뀌어도 맨 아래에 붙어 있던 상태를 유지한다
  useEffect(() => {
    const el = ref.current
    if (!el) return
    const ro = new ResizeObserver(() => {
      if (stuck.current) pin(el)
    })
    ro.observe(el)
    if (contentRef.current) ro.observe(contentRef.current)
    return () => ro.disconnect()
  }, [])

  return { ref, contentRef, onScroll, toBottom, unseen }
}
