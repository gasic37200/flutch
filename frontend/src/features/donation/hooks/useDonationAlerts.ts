import { useEffect, useState } from 'react'
import { api } from '../../../api/client.ts'
import type { DonationCreated } from '../../../api/generated/DonationCreated.ts'
import { useToast } from '../../../hooks/useToast.ts'
import type { OnEvent } from '../../live/hooks/useBroadcastEvents.ts'
import { errorText } from '../../wallet/hooks/errorText.ts'

export const ALERT_MS = 10_000
const MAX_QUEUE = 30

export type DonationAlerts = ReturnType<typeof useDonationAlerts>

/**
 * 후원 알림 큐: 도착 순서대로 10초씩 보여 준다. `donation.skipped`는 그 후원을 큐에서 뺀다(표시 중이면 다음으로).
 * 시청 화면은 공개형, 스튜디오(소유자)는 실명 이벤트를 받는다(서버가 연결별로 골라 보냄).
 */
export function useDonationAlerts(onEvent: OnEvent) {
  const toast = useToast()
  const [queue, setQueue] = useState<DonationCreated[]>([])
  const [skipping, setSkipping] = useState(false)

  useEffect(() => {
    const offs = [
      onEvent('donation.created', (d) => setQueue((q) => (q.some((x) => x.donation.id === d.donation.id) ? q : [...q, d].slice(-MAX_QUEUE)))),
      onEvent('donation.skipped', ({ id }) => setQueue((q) => q.filter((x) => x.donation.id !== id))),
    ]
    return () => offs.forEach((off) => off())
  }, [onEvent])

  const current = queue[0]
  const currentId = current?.donation.id
  useEffect(() => {
    if (currentId === undefined) return
    const t = setTimeout(() => setQueue((q) => q.filter((x) => x.donation.id !== currentId)), ALERT_MS)
    return () => clearTimeout(t)
  }, [currentId])

  /** 소유자만(서버가 다시 검사). 성공하면 모든 연결에 skipped가 가고, 내 화면은 바로 넘긴다. */
  const skip = async () => {
    if (currentId === undefined || skipping) return
    setSkipping(true)
    try {
      await api<void>(`/donations/${currentId}/skip`, { method: 'POST' })
      setQueue((q) => q.filter((x) => x.donation.id !== currentId))
    } catch (e) {
      toast({ tone: 'error', message: errorText(e) })
    } finally {
      setSkipping(false)
    }
  }

  return { current: current?.donation, waiting: Math.max(0, queue.length - 1), skip, skipping }
}
