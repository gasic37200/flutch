import { useParams } from 'react-router'
import { useBroadcast } from '../features/broadcast/hooks/useBroadcasts.ts'
import { useBroadcastEvents } from '../features/live/hooks/useBroadcastEvents.ts'
import { useViewer } from '../features/live/hooks/useViewer.ts'

// T11이 실제 시청 UI로 교체한다. 지금은 연결 확인용 최소 화면.
export default function WatchPage() {
  const id = Number(useParams().broadcastId) || undefined
  const { data: broadcast, error } = useBroadcast(id)
  const { connected } = useBroadcastEvents(id)
  const viewer = useViewer(id)

  return (
    <main>
      <h1>방송 시청</h1>
      <p>{error ? error.message : `${broadcast?.title ?? '불러오는 중'} · ${broadcast?.status ?? ''}`}</p>
      <p>실시간 연결: {connected ? '연결됨' : '끊김'} · 시청 연결: {viewer.status} · 시청자 {broadcast?.viewerCount ?? 0}명</p>
      <p>받은 트랙: {Object.keys(viewer.tracks).join(', ') || '없음'}</p>
      {viewer.muted && <button onClick={() => void viewer.unmute()}>소리 켜기</button>}
    </main>
  )
}
