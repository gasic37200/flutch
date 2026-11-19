import { useEffect, useRef } from 'react'
import type { Donation } from '../../../api/generated/Donation.ts'
import { chime, speak, stopSpeaking, ttsText } from '../tts.ts'

/** 알림이 뜨고 TTS를 읽기 시작하기까지 */
export const TTS_DELAY_MS = 1_000

/**
 * 알림이 바뀔 때 "띠링" 후 1초 뒤 메시지 TTS(스튜디오·시청·다시보기 공통). `volume` 0이면 무음, `tts` false면 띠링만.
 * 볼륨·TTS를 바꿔도 다시 울리지 않는다. 다음 알림·언마운트는 읽기를 끊는다.
 */
export function useAlertSound(d: Donation | undefined, volume: number, tts = true) {
  const now = useRef({ volume, tts })
  useEffect(() => {
    now.current = { volume, tts }
  })
  const id = d?.id
  const text = d ? ttsText(d) : ''
  useEffect(() => {
    if (id === undefined) return
    chime(now.current.volume)
    const t = setTimeout(() => {
      const { volume: v, tts: on } = now.current
      if (text && on && v > 0) speak(text, v)
    }, TTS_DELAY_MS)
    return () => {
      clearTimeout(t)
      stopSpeaking()
    }
  }, [id, text])
}
