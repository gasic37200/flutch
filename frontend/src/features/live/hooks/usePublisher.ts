import { useQueryClient } from '@tanstack/react-query'
import { DisconnectReason, LocalAudioTrack, LocalVideoTrack, Room, RoomEvent, Track } from 'livekit-client'
import { useCallback, useEffect, useRef, useState } from 'react'
import { api, ApiRequestError } from '../../../api/client.ts'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import type { StartLiveResponse } from '../../../api/generated/StartLiveResponse.ts'
import { useTabId } from './useClientIds.ts'

export type PublisherStatus =
  | 'idle'
  | 'starting'
  | 'confirm-takeover' // 다른 기기가 송출 중: confirmTakeover()로 전환하거나 cancelTakeover()
  | 'connected'
  | 'displaced' // 다른 기기로 전환돼 내보내짐(자동 재접속 안 함)
  | 'disconnected' // LiveKit이 재연결을 포기: start()로 이어서 송출
  | 'ended'
  | 'error'

type PubSource = Track.Source.ScreenShare | Track.Source.Camera | Track.Source.Microphone
/** publishMediaTrack이 받는 소스(화면 소리 포함) */
export type MediaSource = PubSource | Track.Source.ScreenShareAudio

/**
 * 스튜디오용 송출 훅. 반드시 사용자 클릭 핸들러에서 start()·publish*를 부른다.
 * 소유자 아닌 사용자는 서버가 403으로 거절한다.
 */
export function usePublisher(broadcastId: number | undefined) {
  const qc = useQueryClient()
  const tabId = useTabId()
  const roomRef = useRef<Room | null>(null)
  const [status, setStatus] = useState<PublisherStatus>('idle')
  const [error, setError] = useState<string | null>(null)
  const [published, setPublished] = useState<Track.Source[]>([])

  // 언마운트·화면 이탈 시 카메라 불과 송출을 끊는다.
  useEffect(
    () => () => {
      const room = roomRef.current
      roomRef.current = null
      room?.localParticipant.trackPublications.forEach((p) => p.track?.stop())
      void room?.disconnect()
    },
    [],
  )

  const connect = useCallback(
    async (takeover: boolean) => {
      if (broadcastId === undefined) return
      setError(null)
      setStatus('starting')
      let res: StartLiveResponse
      try {
        res = await api<StartLiveResponse>(`/broadcasts/${broadcastId}/live`, {
          method: 'POST',
          json: takeover ? { tabId, takeover: true } : { tabId },
        })
      } catch (e) {
        if (e instanceof ApiRequestError && e.code === 'LIVE_PUBLISHER_ACTIVE') return setStatus('confirm-takeover')
        setError(e instanceof ApiRequestError ? e.message : '송출을 시작하지 못했어요.')
        return setStatus('error')
      }
      qc.setQueryData<BroadcastDetail>(['broadcast', broadcastId], res.broadcast)
      void roomRef.current?.disconnect() // 재시도 시 이전 연결 정리
      const room = new Room()
      roomRef.current = room
      const sync = () => setPublished([...room.localParticipant.trackPublications.values()].map((p) => p.source))
      room
        .on(RoomEvent.LocalTrackPublished, sync)
        .on(RoomEvent.LocalTrackUnpublished, sync)
        .on(RoomEvent.Disconnected, (reason) => {
          if (roomRef.current !== room) return // 우리가 정리한 연결
          setPublished([])
          setStatus(reason === DisconnectReason.PARTICIPANT_REMOVED ? 'displaced' : 'disconnected')
        })
      try {
        await room.connect(res.livekit.url, res.livekit.token)
        if (roomRef.current !== room) return void room.disconnect() // 연결 중 언마운트
        setStatus('connected')
      } catch {
        setError('방송 서버에 연결하지 못했어요.')
        setStatus('error')
      }
    },
    [broadcastId, tabId, qc],
  )

  const toggle = useCallback(async (source: PubSource, on: boolean) => {
    const lp = roomRef.current?.localParticipant
    if (!lp) return
    try {
      if (source === Track.Source.ScreenShare) await lp.setScreenShareEnabled(on, { audio: on })
      else if (source === Track.Source.Camera) await lp.setCameraEnabled(on)
      else await lp.setMicrophoneEnabled(on)
    } catch {
      setError('장치를 켜지 못했어요. 권한을 확인해 주세요.')
    }
  }, [])

  // 스튜디오가 직접 잡은(볼륨 처리된) 트랙을 송출한다. 같은 소스가 있으면 교체. 넘긴 트랙은 clone이어야 unpublish 때 원본이 안 꺼진다.
  const publishMediaTrack = useCallback(async (track: MediaStreamTrack, source: MediaSource) => {
    const lp = roomRef.current?.localParticipant
    if (!lp) return
    try {
      const old = lp.getTrackPublication(source)?.track
      if (old) await lp.unpublishTrack(old, true)
      const local = track.kind === 'video' ? new LocalVideoTrack(track, undefined, true) : new LocalAudioTrack(track, undefined, true)
      await lp.publishTrack(local, { source })
    } catch {
      setError('송출을 시작하지 못했어요. 잠시 후 다시 시도해 주세요.')
    }
  }, [])

  const unpublishSource = useCallback(async (source: MediaSource) => {
    const lp = roomRef.current?.localParticipant
    const old = lp?.getTrackPublication(source)?.track
    if (lp && old) await lp.unpublishTrack(old, true)
  }, [])

  const stop = useCallback(async () => {
    if (broadcastId === undefined) return
    try {
      const d = await api<BroadcastDetail>(`/broadcasts/${broadcastId}/live`, { method: 'DELETE' })
      qc.setQueryData(['broadcast', broadcastId], d)
    } catch (e) {
      setError(e instanceof ApiRequestError ? e.message : '방송을 종료하지 못했어요.')
      return
    }
    const room = roomRef.current
    roomRef.current = null
    room?.localParticipant.trackPublications.forEach((p) => p.track?.stop())
    void room?.disconnect()
    setPublished([])
    setStatus('ended')
  }, [broadcastId, qc])

  return {
    status,
    error,
    /** 지금 송출 중인 소스(screen_share·screen_share_audio·camera·microphone) */
    published,
    start: () => connect(false),
    confirmTakeover: () => connect(true),
    cancelTakeover: () => setStatus('idle'),
    publishScreen: (on: boolean) => toggle(Track.Source.ScreenShare, on),
    publishCamera: (on: boolean) => toggle(Track.Source.Camera, on),
    publishMicrophone: (on: boolean) => toggle(Track.Source.Microphone, on),
    publishMediaTrack,
    unpublishSource,
    stop,
  }
}
