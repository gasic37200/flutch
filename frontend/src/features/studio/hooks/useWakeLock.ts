import { useEffect, useState } from 'react'

const supported = typeof navigator !== 'undefined' && 'wakeLock' in navigator

/**
 * active인 동안 화면이 꺼지지 않게 한다. 탭이 숨겨지면 브라우저가 풀어 버리므로 다시 보일 때 재요청한다.
 * 반환값: 화면 꺼짐 방지에 실패했거나 지원하지 않으면 true(사용자에게 직접 설정하라고 안내).
 */
export function useWakeLock(active: boolean) {
  const [failed, setFailed] = useState(false)
  useEffect(() => {
    if (!active || !supported) return
    let lock: WakeLockSentinel | null = null
    let cancelled = false
    const request = async () => {
      try {
        const l = await navigator.wakeLock.request('screen')
        if (cancelled) void l.release()
        else {
          lock = l
          setFailed(false)
        }
      } catch {
        setFailed(true)
      }
    }
    const onVisible = () => {
      if (document.visibilityState === 'visible') void request()
    }
    void request()
    document.addEventListener('visibilitychange', onVisible)
    return () => {
      cancelled = true
      document.removeEventListener('visibilitychange', onVisible)
      void lock?.release()
    }
  }, [active])
  return active && (!supported || failed)
}
