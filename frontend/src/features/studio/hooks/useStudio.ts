import { Track } from 'livekit-client'
import { useEffect, useRef, useState } from 'react'
import { useRequireLogin } from '../../auth/hooks/useAuth.ts'
import { useChat } from '../../chat/hooks/useChat.ts'
import { useBroadcast, useUpdateBroadcast } from '../../broadcast/hooks/useBroadcasts.ts'
import { useDonationAlerts } from '../../donation/hooks/useDonationAlerts.ts'
import { useBroadcastEvents } from '../../live/hooks/useBroadcastEvents.ts'
import { usePublisher, type MediaSource } from '../../live/hooks/usePublisher.ts'
import { useToast } from '../../../hooks/useToast.ts'
import type { PipCorner } from '../../../api/generated/PipCorner.ts'
import { detectEnv } from '../environment.ts'
import { useElapsed } from './useElapsed.ts'
import { useWakeLock } from './useWakeLock.ts'
import { useThumbnailCapture } from './useThumbnailCapture.ts'
import { useStudioMedia, type Mode } from './useStudioMedia.ts'

const LIVE_STATUSES = ['connected', 'displaced', 'disconnected']

/** PC 송출 화면의 모든 상태·동작. 화면 컴포넌트는 이 반환값만 그린다. */
export function useStudio(broadcastId: number | undefined) {
  const { me } = useRequireLogin()
  const toast = useToast()
  const [env] = useState(detectEnv)
  const broadcast = useBroadcast(broadcastId)
  const events = useBroadcastEvents(broadcastId)
  const chat = useChat(broadcastId, events)
  const alerts = useDonationAlerts(events.onEvent)
  const pub = usePublisher(broadcastId)
  const media = useStudioMedia(env)
  const updateBroadcast = useUpdateBroadcast(broadcastId)
  const [endConfirm, setEndConfirm] = useState(false)
  const [ending, setEnding] = useState(false)
  const data = broadcast.data
  const elapsed = useElapsed(data?.startedAt, data?.endedAt)

  const { onEvent } = events
  useEffect(
    () => onEvent('broadcast.duration_warning', () => toast({ tone: 'info', message: '10분 후 방송이 자동으로 종료돼요.' })),
    [onEvent, toast],
  )

  // 연결된 동안 캡처한 소스와 송출 중인 소스를 맞춘다: 새 소스는 보내고, 사라진(화면 공유 중지 등) 소스는 내린다.
  const sent = useRef(new Map<MediaSource, string>())
  const { publishMediaTrack, unpublishSource } = pub
  const { items, version } = media
  useEffect(() => {
    if (pub.status !== 'connected') return void sent.current.clear()
    const now = new Map(items().map((i) => [i.source, i] as const))
    for (const [source, key] of [...sent.current]) {
      if (now.get(source)?.key !== key) {
        sent.current.delete(source)
        if (!now.has(source)) void unpublishSource(source)
      }
    }
    for (const [source, item] of now) {
      if (sent.current.get(source) === item.key) continue
      sent.current.set(source, item.key)
      void publishMediaTrack(item.make(), source)
    }
  }, [pub.status, version, items, publishMediaTrack, unpublishSource])

  const status = broadcast.data?.status
  const isOwner = !!me && !!data && data.owner.id === me.id
  const phase = pub.status === 'ended' || status === 'ENDED' ? 'ended' : LIVE_STATUSES.includes(pub.status) ? 'live' : 'ready'

  // 방송이 끝났거나(서버 종료 포함) 다른 기기로 넘어가면 캡처를 끊는다(카메라 불·화면 공유 표시 해제)
  useEffect(() => {
    if (phase === 'ended' || pub.status === 'displaced') media.stopAll()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [phase, pub.status])

  const wakeLockMissing = useWakeLock(phase === 'live')
  useThumbnailCapture(broadcastId, phase === 'live', media.stream)

  const startPreview = async () => {
    if (await media.captureVideo()) await media.captureMic()
  }

  /** 모드 전환: 송출 전에는 선택만, 송출 중에는 새 소스를 잡아 바로 교체한다 */
  const chooseMode = async (m: Mode) => {
    if (m === media.mode) return
    if (phase === 'live') await media.captureVideo(m)
    else media.setMode(m)
  }

  /** 카메라 전·후면 전환. 게시 중인 clone이 카메라를 잡고 있으면 모바일이 새로 못 열어서, 먼저 내린다(새 트랙은 동기화 effect가 다시 게시) */
  const flipCamera = async () => {
    await pub.unpublishSource(Track.Source.Camera)
    await media.flipCamera()
  }

  /** 웹캠 PIP 위치: 서버에 저장하면 broadcast.updated로 시청자에게 반영된다 */
  const setPipCorner = (pipCorner: PipCorner) =>
    updateBroadcast.mutate(
      { title: null, category: null, tags: null, record: null, pipCorner },
      { onError: () => toast({ tone: 'error', message: '웹캠 위치를 바꾸지 못했어요.' }) },
    )

  const stop = async () => {
    setEnding(true)
    await pub.stop()
    setEnding(false)
    setEndConfirm(false)
  }

  return {
    me,
    env,
    broadcast,
    chat,
    alerts,
    data,
    isOwner,
    phase,
    pub,
    media,
    connected: events.connected,
    elapsed,
    wakeLockMissing,
    endConfirm,
    ending,
    askEnd: () => setEndConfirm(true),
    cancelEnd: () => setEndConfirm(false),
    stop,
    startPreview,
    chooseMode,
    /** 화면 소스 변경·화면 공유 중지 뒤 다시 공유 */
    reshare: () => media.captureVideo(),
    flipCamera,
    pipOn: !!media.pipStream,
    togglePip: (on: boolean) => (on ? media.capturePip() : media.stopPip()),
    setPipCorner,
    goLive: () => pub.start(),
  }
}
