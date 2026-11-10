import { useEffect, useState } from 'react'
import { useBroadcast } from '../../broadcast/hooks/useBroadcasts.ts'
import { useBroadcastEvents } from '../../live/hooks/useBroadcastEvents.ts'
import { useViewer } from '../../live/hooks/useViewer.ts'

/** 플레이어 영역에 보이는 상태. playing 외에는 안내 문구를 영상 위에 덮는다. */
export type WatchView = 'loading' | 'waiting' | 'connecting' | 'playing' | 'streamerReconnecting' | 'netReconnecting' | 'screenPaused' | 'ended' | 'failed'

const PAUSE_GRACE_MS = 2000 // 연결 직후 트랙이 구독되기까지의 빈틈을 "중단"으로 보지 않는다

/** 시청 화면의 모든 상태: 방송 상세 캐시 + 실시간 이벤트 + LiveKit 구독을 한 화면 상태로 합친다. */
export function useWatch(broadcastId: number | undefined) {
  const query = useBroadcast(broadcastId)
  const broadcast = query.data
  const events = useBroadcastEvents(broadcastId)
  const viewer = useViewer(broadcastId)
  const hasVideo = Boolean(viewer.tracks.screenShare ?? viewer.tracks.camera)
  const noVideo = viewer.status === 'connected' && !hasVideo

  const [graceOver, setGraceOver] = useState(false)
  useEffect(() => {
    if (!noVideo) return
    const t = setTimeout(() => setGraceOver(true), PAUSE_GRACE_MS)
    return () => {
      clearTimeout(t)
      setGraceOver(false)
    }
  }, [noVideo])

  let view: WatchView
  if (!broadcast) view = 'loading'
  else if (broadcast.status === 'ENDED') view = 'ended'
  else if (broadcast.status === 'READY') view = 'waiting'
  else if (broadcast.reconnecting) view = 'streamerReconnecting'
  else if (viewer.status === 'reconnecting') view = 'netReconnecting'
  else if (viewer.status === 'disconnected' || viewer.status === 'error') view = 'failed'
  else if (viewer.status !== 'connected') view = 'connecting'
  else if (noVideo && graceOver) view = 'screenPaused'
  else if (noVideo) view = 'connecting'
  else view = 'playing'

  return { broadcast, error: query.error, refetch: query.refetch, view, viewer, events }
}

/** 1초마다 갱신되는 현재 시각(ms). 경과 시간 표시용. */
export function useNow(active: boolean) {
  const [now, setNow] = useState(() => Date.now())
  useEffect(() => {
    if (!active) return
    const t = setInterval(() => setNow(Date.now()), 1000)
    return () => clearInterval(t)
  }, [active])
  return now
}

/** 초 → `H:MM:SS`(1시간 미만은 `M:SS`). */
export function formatDuration(ms: number) {
  const s = Math.max(0, Math.floor(ms / 1000))
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = String(s % 60).padStart(2, '0')
  return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${sec}` : `${m}:${sec}`
}
