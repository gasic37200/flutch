import { useQueryClient } from '@tanstack/react-query'
import { DisconnectReason, type RemoteTrack, Room, RoomEvent, Track } from 'livekit-client'
import { useCallback, useEffect, useRef, useState } from 'react'
import { api, ApiRequestError } from '../../../api/client.ts'
import type { LiveKitToken } from '../../../api/generated/LiveKitToken.ts'
import { useAuth } from '../../auth/hooks/useAuth.ts'
import { useBroadcast } from '../../broadcast/hooks/useBroadcasts.ts'
import { useGuestId, useTabId } from './useClientIds.ts'

export type ViewerStatus = 'idle' | 'waiting' | 'connecting' | 'connected' | 'reconnecting' | 'ended' | 'disconnected' | 'error'
export type ViewerTracks = Partial<Record<'screenShare' | 'screenShareAudio' | 'camera' | 'microphone', RemoteTrack>>

const SLOT = {
  [Track.Source.ScreenShare]: 'screenShare',
  [Track.Source.ScreenShareAudio]: 'screenShareAudio',
  [Track.Source.Camera]: 'camera',
  [Track.Source.Microphone]: 'microphone',
} as const

/**
 * 시청용 구독 전용 연결. 방송이 LIVE일 때만 viewer-token을 받고, ENDED면 끊는다.
 * 스튜디오 탭에서는 `enabled: false`로 부른다(같은 tabId로 받으면 송출이 끊긴다).
 * 방송 상태는 `useBroadcast` 캐시를 보므로 같은 화면에서 `useBroadcastEvents`를 함께 켜야 READY → LIVE에 반응한다.
 */
export function useViewer(broadcastId: number | undefined, enabled = true) {
  const qc = useQueryClient()
  const { data: broadcast } = useBroadcast(broadcastId)
  const { me, isPending } = useAuth()
  const viewerTabId = `${useTabId()}-v` // 8~64자 규칙 안(UUID 36 + 2)
  const guestId = useGuestId()
  const status0 = broadcast?.status
  const [conn, setStatus] = useState<ViewerStatus>('connecting')
  const [tracks, setTracks] = useState<ViewerTracks>({})
  const [muted, setMuted] = useState(true)
  const roomRef = useRef<Room | null>(null)

  const live = enabled && !isPending && status0 === 'LIVE'
  const userId = me?.id
  // LIVE가 아니면 연결 상태 대신 방송 상태에서 파생한다.
  const status: ViewerStatus = live ? conn : !enabled || status0 === undefined ? 'idle' : status0 === 'ENDED' ? 'ended' : 'waiting'
  useEffect(() => {
    if (!live) return
    let cancelled = false
    const room = new Room({ adaptiveStream: true, dynacast: true })
    // TrackUnsubscribed는 pub.track이 비워지기 전에 발행되므로 떠나는 트랙은 직접 뺀다.
    const sync = (gone?: RemoteTrack) => {
      const next: ViewerTracks = {}
      room.remoteParticipants.forEach((p) =>
        p.trackPublications.forEach((pub) => {
          const slot = pub.track && SLOT[pub.source as keyof typeof SLOT]
          if (slot && pub.track && pub.track !== gone) next[slot] = pub.track
        }),
      )
      setTracks(next)
    }
    room
      .on(RoomEvent.TrackSubscribed, () => sync())
      .on(RoomEvent.TrackUnsubscribed, (t) => sync(t))
      .on(RoomEvent.Reconnecting, () => setStatus('reconnecting'))
      .on(RoomEvent.Reconnected, () => setStatus('connected'))
      .on(RoomEvent.Disconnected, (reason) => {
        if (cancelled) return
        // 서버가 방을 닫았다 = 방송 종료. 끊김 화면 대신 상세 조회가 종료 상태를 정한다.
        if (reason === DisconnectReason.ROOM_DELETED) void qc.invalidateQueries({ queryKey: ['broadcast', broadcastId] })
        else setStatus('disconnected')
      })
    // StrictMode의 첫 마운트가 곧바로 정리돼도 토큰 요청·연결을 만들지 않게 한 틱 미룬다.
    const timer = setTimeout(() => {
      setStatus('connecting')
      api<LiveKitToken>(`/broadcasts/${broadcastId}/viewer-token`, {
        method: 'POST',
        // 송출 탭과 같은 신원이 되면 서버가 송출자 복귀로 오인한다: 시청용 tabId는 따로 만든다.
        json: userId === undefined ? { tabId: viewerTabId, guestId } : { tabId: viewerTabId },
      })
        .then(async (t) => {
          if (cancelled) return
          await room.connect(t.url, t.token)
          if (!cancelled) {
            roomRef.current = room
            setStatus('connected')
            sync()
          }
        })
        .catch((e: unknown) => {
          // 토큰 요청 직전에 방송이 끝난 경우(409)는 곧 broadcast.ended가 상태를 정리한다.
          if (!cancelled) setStatus(e instanceof ApiRequestError && e.status === 409 ? 'waiting' : 'error')
        })
    }, 0)
    return () => {
      cancelled = true
      clearTimeout(timer)
      roomRef.current = null
      setTracks({})
      void room.disconnect()
    }
  }, [live, broadcastId, userId, viewerTabId, guestId, qc])

  /** 사용자 클릭에서 호출: 브라우저 오디오 재생을 풀고 `muted`를 false로. */
  const unmute = useCallback(async () => {
    await roomRef.current?.startAudio()
    setMuted(false)
  }, [])

  /** `muted`: 시작은 음소거(자동재생 정책). `<video muted={muted}>`·오디오 트랙 attach 요소에 그대로 쓴다. */
  return { status, tracks, muted, unmute }
}
