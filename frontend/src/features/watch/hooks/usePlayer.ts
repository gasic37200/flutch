import type { RemoteTrack } from 'livekit-client'
import { useCallback, useEffect, useRef, useState } from 'react'
import type { ViewerTracks } from '../../live/hooks/useViewer.ts'

const CHAT_OVERLAY_KEY = 'flutch.chatOverlay'

/** 트랙을 요소에 붙이고 떼는 것까지 책임진다. */
function useAttach<T extends HTMLMediaElement>(track: RemoteTrack | undefined) {
  const ref = useRef<T>(null)
  useEffect(() => {
    const el = ref.current
    if (!el || !track) return
    track.attach(el)
    return () => void track.detach(el)
  }, [track])
  return ref
}

/**
 * 플레이어 요소 연결·볼륨·전체화면. 화면 공유 트랙이 우선이고 없으면 카메라를 크게 쓴다.
 * 화면 공유 소리가 있으면 마이크와 함께 각각 재생한다.
 */
export function usePlayer(tracks: ViewerTracks, muted: boolean) {
  const videoRef = useAttach<HTMLVideoElement>(tracks.screenShare ?? tracks.camera)
  // 화면 공유가 있을 때만 카메라를 PIP로 쓴다(없으면 위 videoRef가 카메라를 크게 보여 준다)
  const pipRef = useAttach<HTMLVideoElement>(tracks.screenShare ? tracks.camera : undefined)
  const [pipHidden, setPipHidden] = useState(false)
  const micRef = useAttach<HTMLAudioElement>(tracks.microphone)
  const shareAudioRef = useAttach<HTMLAudioElement>(tracks.screenShareAudio)
  const boxRef = useRef<HTMLDivElement>(null)
  const [volume, setVolume] = useState(1)
  const [fullscreen, setFullscreen] = useState(false)
  // 전체화면 채팅 오버레이: 시청자 로컬 설정(기본 켬)
  const [chatOverlay, setChatOverlay] = useState(() => {
    try {
      return localStorage.getItem(CHAT_OVERLAY_KEY) !== 'off'
    } catch {
      return true
    }
  })
  const toggleChatOverlay = () => {
    const on = !chatOverlay
    setChatOverlay(on)
    try {
      localStorage.setItem(CHAT_OVERLAY_KEY, on ? 'on' : 'off')
    } catch {
      /* 저장 실패해도 이번 방문에는 적용된다 */
    }
  }

  const audioMicTrack = tracks.microphone
  const audioShareTrack = tracks.screenShareAudio
  useEffect(() => {
    for (const el of [micRef.current, shareAudioRef.current]) if (el) el.volume = volume
  }, [volume, audioMicTrack, audioShareTrack, micRef, shareAudioRef])

  useEffect(() => {
    const on = () => setFullscreen(document.fullscreenElement === boxRef.current)
    document.addEventListener('fullscreenchange', on)
    return () => document.removeEventListener('fullscreenchange', on)
  }, [])

  const toggleFullscreen = useCallback(() => {
    if (document.fullscreenElement) void document.exitFullscreen()
    else if (boxRef.current?.requestFullscreen) void boxRef.current.requestFullscreen()
    // iOS Safari는 요소 전체화면이 없어 video 자체의 전용 전체화면을 쓴다
    else (videoRef.current as (HTMLVideoElement & { webkitEnterFullscreen?: () => void }) | null)?.webkitEnterFullscreen?.()
  }, [videoRef])

  return { videoRef, pipRef, pipHidden, togglePip: () => setPipHidden((h) => !h), hasPip: !!(tracks.screenShare && tracks.camera), micRef, shareAudioRef, boxRef, volume, setVolume, fullscreen, toggleFullscreen, chatOverlay, toggleChatOverlay, muted }
}
