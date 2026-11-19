import { useRef, useState } from 'react'
import type { Donation } from '../../../api/generated/Donation.ts'
import { usePlayerShortcuts } from '../../../hooks/usePlayerShortcuts.ts'
import { useAlertSound } from '../../donation/hooks/useAlertSound.ts'
import { stopSpeaking } from '../../donation/tts.ts'

const clamp = (v: number) => Math.min(1, Math.max(0, v))

/** 다시보기 영상: 후원 알림 띠링·TTS(영상 볼륨을 따르고, 일시정지·탐색이면 끊음)와 단축키. */
export function useReplayPlayer(alert: Donation | undefined) {
  const videoRef = useRef<HTMLVideoElement>(null)
  const boxRef = useRef<HTMLDivElement>(null)
  const [volume, setVolume] = useState(1)
  const [paused, setPaused] = useState(true)
  useAlertSound(alert, paused ? 0 : volume)

  const v = () => videoRef.current
  usePlayerShortcuts({
    togglePlay: () => {
      const el = v()
      if (el) void (el.paused ? el.play().catch(() => {}) : el.pause())
    },
    toggleMute: () => {
      const el = v()
      if (el) el.muted = !el.muted
    },
    toggleFullscreen: () => {
      if (document.fullscreenElement) void document.exitFullscreen()
      else void boxRef.current?.requestFullscreen?.()
    },
    seek: (sec) => {
      const el = v()
      if (el) el.currentTime = Math.max(0, el.currentTime + sec)
    },
    volume: (d) => {
      const el = v()
      if (!el) return
      el.volume = clamp(el.volume + d)
      if (el.volume > 0) el.muted = false
    },
  })

  const stop = () => stopSpeaking()
  return {
    videoRef,
    boxRef,
    media: {
      onPlaying: () => setPaused(false),
      onPause: () => {
        setPaused(true)
        stop()
      },
      onSeeking: stop,
      onVolumeChange: (e: { currentTarget: HTMLVideoElement }) => setVolume(e.currentTarget.muted ? 0 : e.currentTarget.volume),
    },
  }
}
