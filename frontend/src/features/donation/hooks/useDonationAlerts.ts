import { useEffect, useRef, useState } from 'react'
import { api } from '../../../api/client.ts'
import type { DonationCreated } from '../../../api/generated/DonationCreated.ts'
import { useToast } from '../../../hooks/useToast.ts'
import type { OnEvent } from '../../live/hooks/useBroadcastEvents.ts'
import { loadTtsOn, saveTtsOn, speak, stopSpeaking, ttsText } from '../tts.ts'
import { errorText } from '../../wallet/hooks/errorText.ts'

export const ALERT_MS = 10_000
/** 후원 이벤트를 받고 알림을 띄우기까지(송출 지연에 맞춰 영상과 비슷한 때 뜨게) */
export const ALERT_DELAY_MS = 3_000
/** 알림이 뜨고 TTS를 읽기 시작하기까지 */
export const TTS_DELAY_MS = 1_000
const MAX_QUEUE = 30

export type DonationAlerts = ReturnType<typeof useDonationAlerts>

/**
 * 후원 알림 큐: 이벤트를 받고 3초 뒤 큐에 넣어, 도착 순서대로 10초씩 보여 준다. `donation.skipped`는 그 후원을 큐에서 뺀다(표시 중이면 다음으로).
 * 시청 화면은 공개형, 스튜디오(소유자)는 실명 이벤트를 받는다(서버가 연결별로 골라 보냄).
 */
export function useDonationAlerts(onEvent: OnEvent, silent = false) {
  const toast = useToast()
  const [queue, setQueue] = useState<DonationCreated[]>([])
  const [skipping, setSkipping] = useState(false)
  const [ttsOn, setTtsOn] = useState(loadTtsOn)
  const canSpeak = useRef(false)
  useEffect(() => {
    canSpeak.current = ttsOn && !silent
  })

  useEffect(() => {
    const timers = new Set<ReturnType<typeof setTimeout>>()
    const add = (d: DonationCreated) => setQueue((q) => (q.some((x) => x.donation.id === d.donation.id) ? q : [...q, d].slice(-MAX_QUEUE)))
    // 지연 중에 넘겨진 후원은 skipped 목록으로 기억해 늦게 큐에 들어가지 않게 한다
    const skipped = new Set<number>()
    const offs = [
      onEvent('donation.created', (d) => {
        const t = setTimeout(() => {
          timers.delete(t)
          if (!skipped.has(d.donation.id)) add(d)
        }, ALERT_DELAY_MS)
        timers.add(t)
      }),
      onEvent('donation.skipped', ({ id }) => {
        skipped.add(id)
        setQueue((q) => q.filter((x) => x.donation.id !== id))
      }),
    ]
    return () => {
      offs.forEach((off) => off())
      timers.forEach(clearTimeout)
    }
  }, [onEvent])

  const current = queue[0]
  const currentId = current?.donation.id
  useEffect(() => {
    if (currentId === undefined) return
    const t = setTimeout(() => setQueue((q) => q.filter((x) => x.donation.id !== currentId)), ALERT_MS)
    return () => clearTimeout(t)
  }, [currentId])

  // 알림이 바뀔 때만 읽는다(볼륨을 만져도 다시 읽지 않음). 뜬 뒤 1초 뒤 시작, 그 전에 넘어가면 읽지 않는다. 다음 알림·넘기기·언마운트는 cleanup이 끊는다.
  const currentDonation = current?.donation
  useEffect(() => {
    const text = currentDonation && ttsText(currentDonation)
    if (!text) return
    const t = setTimeout(() => canSpeak.current && speak(text), TTS_DELAY_MS)
    return () => {
      clearTimeout(t)
      stopSpeaking()
    }
  }, [currentDonation])

  const toggleTts = () => {
    if (ttsOn) stopSpeaking()
    saveTtsOn(!ttsOn)
    setTtsOn(!ttsOn)
  }

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

  return { current: current?.donation, next: queue.at(1)?.donation, waiting: Math.max(0, queue.length - 1), skip, skipping, tts: { on: ttsOn, toggle: toggleTts } }
}
