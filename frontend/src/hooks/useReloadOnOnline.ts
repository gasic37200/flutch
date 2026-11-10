import { useEffect } from 'react'

/** enabled인 동안 브라우저가 다시 온라인이 되면 화면을 새로 불러온다(네트워크 에러 화면의 "자동 재연결"). */
export function useReloadOnOnline(enabled: boolean) {
  useEffect(() => {
    if (!enabled) return
    const reload = () => window.location.reload()
    window.addEventListener('online', reload)
    return () => window.removeEventListener('online', reload)
  }, [enabled])
}
