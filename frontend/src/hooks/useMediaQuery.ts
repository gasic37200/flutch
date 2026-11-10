import { useSyncExternalStore } from 'react'

/** CSS 미디어 쿼리 일치 여부. CSS로 숨기는 대신 한 곳만 렌더할 때 쓴다(상태 있는 패널의 이중 마운트 방지). */
export function useMediaQuery(query: string) {
  return useSyncExternalStore(
    (cb) => {
      const m = window.matchMedia(query)
      m.addEventListener('change', cb)
      return () => m.removeEventListener('change', cb)
    },
    () => window.matchMedia(query).matches,
  )
}
