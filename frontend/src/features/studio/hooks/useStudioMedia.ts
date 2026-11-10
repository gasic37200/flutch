import { useCallback, useEffect, useRef, useState } from 'react'
import { Track } from 'livekit-client'
import type { MediaSource } from '../../live/hooks/usePublisher.ts'
import type { StudioEnv } from '../environment.ts'

export type Mode = 'screen' | 'camera'
export type Facing = 'user' | 'environment'
export type Channel ='mic' | 'sys'
export type MediaItem = { source: MediaSource; key: string; make: () => MediaStreamTrack }

type Node = { gain: GainNode; analyser: AnalyserNode; dest: MediaStreamAudioDestinationNode; src: MediaStreamAudioSourceNode | null }

const gainOf = (volume: number, muted: boolean) => (muted ? 0 : volume / 100)

/**
 * 송출 전·중에 같은 캡처를 쓰는 미디어 관리. 화면/카메라/마이크 원본 트랙을 쥐고,
 * 마이크와 화면 소리는 Web Audio(GainNode)를 거친 트랙으로 내보낸다(볼륨·음소거·레벨 미터).
 * 캡처는 반드시 사용자 클릭에서 부른다(conventions §4).
 */
export function useStudioMedia(env: StudioEnv) {
  const [mode, setMode] = useState<Mode>(env.canScreen ? 'screen' : 'camera')
  const [version, setVersion] = useState(0) // 소스가 바뀔 때마다 올려 송출 동기화를 깨운다
  const [stream, setStream] = useState<MediaStream | null>(null) // 미리보기용 영상
  const [hasScreenAudio, setHasScreenAudio] = useState(false)
  const [videoEnded, setVideoEnded] = useState(false)
  const [micOn, setMicOn] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [analysers, setAnalysers] = useState<Record<Channel, AnalyserNode | null>>({ mic: null, sys: null })
  const [volume, setVolume] = useState<Record<Channel, number>>({ mic: 100, sys: 100 })
  const [muted, setMuted] = useState<Record<Channel, boolean>>({ mic: false, sys: false })

  const ctx = useRef<AudioContext | null>(null)
  const nodes = useRef<Partial<Record<Channel, Node>>>({})
  const raw = useRef<{ video: MediaStreamTrack | null; screenAudio: MediaStreamTrack | null; mic: MediaStreamTrack | null }>({ video: null, screenAudio: null, mic: null })
  const pipRaw = useRef<MediaStreamTrack | null>(null) // 화면 모드에서 따로 보내는 웹캠(PIP)
  const [pipStream, setPipStream] = useState<MediaStream | null>(null)
  const [facing, setFacing] = useState<Facing>('user')
  const modeRef = useRef(mode)
  modeRef.current = mode
  const facingRef = useRef(facing)
  facingRef.current = facing

  const bump = () => setVersion((v) => v + 1)

  const node = (ch: Channel): Node => {
    const existing = nodes.current[ch]
    if (existing) return existing
    const c = (ctx.current ??= new AudioContext())
    void c.resume()
    const gain = c.createGain()
    const analyser = c.createAnalyser()
    analyser.fftSize = 512
    const dest = c.createMediaStreamDestination()
    gain.connect(analyser)
    gain.connect(dest)
    const n: Node = { gain, analyser, dest, src: null }
    nodes.current[ch] = n
    setAnalysers((a) => ({ ...a, [ch]: analyser }))
    return n
  }

  const feed = (ch: Channel, track: MediaStreamTrack | null) => {
    const n = node(ch)
    n.src?.disconnect()
    n.src = track && ctx.current ? ctx.current.createMediaStreamSource(new MediaStream([track])) : null
    n.src?.connect(n.gain)
  }

  // 볼륨·음소거를 GainNode에 반영. 노드가 새로 생기면(analysers 변경) 한 번 더 적용한다.
  useEffect(() => {
    for (const ch of ['mic', 'sys'] as const) {
      const n = nodes.current[ch]
      if (n) n.gain.gain.value = gainOf(volume[ch], muted[ch])
    }
  }, [volume, muted, analysers])

  const stopVideo = useCallback(() => {
    raw.current.video?.stop()
    raw.current.screenAudio?.stop()
    raw.current = { ...raw.current, video: null, screenAudio: null }
    if (nodes.current.sys) feed('sys', null)
    setStream(null)
    setHasScreenAudio(false)
    setVideoEnded(false)
    setVersion((v) => v + 1)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const stopPip = useCallback(() => {
    pipRaw.current?.stop()
    pipRaw.current = null
    setPipStream(null)
    setVersion((v) => v + 1)
  }, [])

  /** 웹캠 PIP 켜기: 360p30(대역폭). 사용자 클릭에서 부른다. */
  const capturePip = useCallback(async () => {
    setError(null)
    try {
      const s = await navigator.mediaDevices.getUserMedia({ video: { width: 640, height: 360, frameRate: 30 } })
      pipRaw.current?.stop()
      const track = s.getVideoTracks()[0]
      pipRaw.current = track
      track.addEventListener('ended', () => {
        if (pipRaw.current === track) stopPip()
      })
      setPipStream(new MediaStream([track]))
      setVersion((v) => v + 1)
    } catch {
      setError('웹캠을 켜지 못했어요. 주소창 옆 권한 설정에서 카메라를 허용해 주세요.')
    }
  }, [stopPip])

  const captureVideo = useCallback(async (m: Mode = modeRef.current, face: Facing = facingRef.current) => {
    setError(null)
    try {
      const s =
        m === 'screen'
          ? await navigator.mediaDevices.getDisplayMedia({
              video: { width: 1920, height: 1080, frameRate: 30 },
              // 화면 소리는 처리 없음(features/03). systemAudio는 TS lib에 아직 없다.
              audio: { echoCancellation: false, noiseSuppression: false, autoGainControl: false },
              systemAudio: 'include',
              surfaceSwitching: 'include',
            } as DisplayMediaStreamOptions)
          : await navigator.mediaDevices.getUserMedia({ video: { width: 1280, height: 720, frameRate: 30, facingMode: { ideal: face } } })
      // 새 캡처가 성공한 뒤에야 이전 것을 끈다(취소하면 기존 화면 유지)
      stopVideo()
      const video = s.getVideoTracks()[0]
      const audio = s.getAudioTracks()[0] ?? null
      raw.current = { ...raw.current, video, screenAudio: audio }
      video.addEventListener('ended', () => {
        if (raw.current.video === video) setVideoEnded(true)
      })
      if (m === 'screen') feed('sys', audio)
      else stopPip() // 웹캠 모드는 카메라가 메인 영상이라 PIP가 필요 없다
      setMode(m)
      setFacing(face)
      setStream(new MediaStream([video]))
      setHasScreenAudio(!!audio)
      bump()
      return true
    } catch (e) {
      if (!(e instanceof DOMException && e.name === 'NotAllowedError' && m === 'screen')) {
        // 화면 공유 선택 창을 닫은 경우는 조용히 넘어가고, 그 외(권한 거부·장치 없음)는 안내
        setError(m === 'screen' ? '화면을 공유하지 못했어요. 브라우저 권한을 확인해 주세요.' : '카메라를 켜지 못했어요. 주소창 옆 권한 설정에서 카메라를 허용해 주세요.')
      }
      return false
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [stopVideo, stopPip])

  /** 전·후면 전환. 모바일은 카메라를 동시에 두 번 열 수 없어 현재 트랙을 먼저 끄고, 실패하면 원래 카메라로 되돌린다. */
  const flipCamera = useCallback(async () => {
    const prev = facingRef.current
    raw.current.video?.stop()
    if (await captureVideo('camera', prev === 'user' ? 'environment' : 'user')) return
    if (await captureVideo('camera', prev)) setError('카메라를 전환하지 못했어요.')
  }, [captureVideo])

  const captureMic = useCallback(async () => {
    try {
      const s = await navigator.mediaDevices.getUserMedia({ audio: { echoCancellation: true, noiseSuppression: true, autoGainControl: true } })
      raw.current.mic?.stop()
      const track = s.getAudioTracks()[0]
      raw.current.mic = track
      setMicOn(true)
      track.addEventListener('ended', () => {
        if (raw.current.mic === track) {
          raw.current.mic = null
          setMicOn(false)
          bump()
        }
      })
      feed('mic', track)
      bump()
    } catch {
      setError('마이크를 켜지 못했어요. 주소창 옆 권한 설정에서 마이크를 허용해 주세요.')
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const stopAll = useCallback(() => {
    for (const t of [raw.current.video, raw.current.screenAudio, raw.current.mic]) t?.stop()
    raw.current = { video: null, screenAudio: null, mic: null }
    pipRaw.current?.stop()
    pipRaw.current = null
    setPipStream(null)
    for (const n of Object.values(nodes.current)) n?.src?.disconnect()
    nodes.current = {}
    void ctx.current?.close()
    ctx.current = null
    setStream(null)
    setHasScreenAudio(false)
    setAnalysers({ mic: null, sys: null })
    setMicOn(false)
  }, [])

  useEffect(() => stopAll, [stopAll]) // 화면을 떠나면 카메라 불·오디오 컨텍스트 정리

  /** 지금 송출할 수 있는 소스 목록. make()가 clone을 만들어 준다(송출 해제가 원본을 끄지 않도록). */
  const items = useCallback((): MediaItem[] => {
    const out: MediaItem[] = []
    const { video, mic } = raw.current
    if (video?.readyState === 'live') out.push({ source: modeRef.current === 'screen' ? Track.Source.ScreenShare : Track.Source.Camera, key: video.id, make: () => video.clone() })
    const pip = pipRaw.current
    if (modeRef.current === 'screen' && pip?.readyState === 'live') out.push({ source: Track.Source.Camera, key: pip.id, make: () => pip.clone() })
    const sysTrack = nodes.current.sys?.dest.stream.getAudioTracks()[0]
    // 화면 모드에서는 소리가 없어도 트랙을 보내 둔다(나중에 소리 있는 화면으로 바꿔도 이어지도록)
    if (modeRef.current === 'screen' && video?.readyState === 'live' && sysTrack) out.push({ source: Track.Source.ScreenShareAudio, key: sysTrack.id, make: () => sysTrack.clone() })
    const micTrack = mic && nodes.current.mic?.dest.stream.getAudioTracks()[0]
    if (micTrack) out.push({ source: Track.Source.Microphone, key: micTrack.id, make: () => micTrack.clone() })
    return out
  }, [])

  const toggleMute = (ch: Channel) => setMuted((m) => ({ ...m, [ch]: !m[ch] }))
  const changeVolume = (ch: Channel, v: number) => setVolume((x) => ({ ...x, [ch]: v }))

  return {
    mode,
    setMode: (m: Mode) => {
      if (m === modeRef.current) return
      stopVideo()
      if (m === 'camera') stopPip()
      setMode(m)
    },
    pipStream,
    capturePip,
    stopPip,
    stream,
    videoReady: !!stream && !videoEnded,
    videoEnded,
    micReady: micOn,
    hasScreenAudio,
    error,
    analysers,
    volume,
    muted,
    version,
    facing,
    captureVideo,
    flipCamera,
    captureMic,
    changeVolume,
    toggleMute,
    items,
    stopAll,
  }
}
