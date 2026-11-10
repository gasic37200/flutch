import { useEffect, useState } from 'react'

/** 시작 시각(ISO)부터 지난 시간을 `H:MM:SS`로. 시작 전이면 `0:00:00`, 종료 시각이 있으면 거기서 멈춘다. */
export function useElapsed(startedAt: string | null | undefined, endedAt?: string | null): string {
  const [now, setNow] = useState(() => Date.now())
  useEffect(() => {
    if (!startedAt || endedAt) return
    const id = setInterval(() => setNow(Date.now()), 1000)
    return () => clearInterval(id)
  }, [startedAt, endedAt])
  const sec = startedAt ? Math.max(0, Math.floor(((endedAt ? new Date(endedAt).getTime() : now) - new Date(startedAt).getTime()) / 1000)) : 0
  const p = (n: number) => String(n).padStart(2, '0')
  return `${Math.floor(sec / 3600)}:${p(Math.floor(sec / 60) % 60)}:${p(sec % 60)}`
}
