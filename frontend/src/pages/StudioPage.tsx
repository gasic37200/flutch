import { useSearchParams } from 'react-router'
import { useRequireLogin } from '../features/auth/hooks/useAuth.ts'
import { useBroadcast } from '../features/broadcast/hooks/useBroadcasts.ts'
import { useBroadcastEvents } from '../features/live/hooks/useBroadcastEvents.ts'
import { usePublisher } from '../features/live/hooks/usePublisher.ts'

// T09·T10이 실제 송출 UI로 교체한다. 지금은 연결 확인용 최소 화면.
export default function StudioPage() {
  const { me } = useRequireLogin()
  const id = Number(useSearchParams()[0].get('broadcast')) || undefined
  const { data: broadcast, error } = useBroadcast(id)
  const { connected } = useBroadcastEvents(id)
  const pub = usePublisher(id)

  if (!me) return <main><h1>방송 스튜디오</h1></main>
  if (!id) return <main><h1>방송 스튜디오</h1><p>방송을 선택해 주세요. 방송 관리(/manage)에서 스튜디오로 들어올 수 있어요.</p></main>
  return (
    <main>
      <h1>방송 스튜디오</h1>
      <p>{error ? error.message : `${broadcast?.title ?? '불러오는 중'} · ${broadcast?.status ?? ''}`}</p>
      <p>실시간 연결: {connected ? '연결됨' : '끊김'} · 송출: {pub.status} · 시청자 {broadcast?.viewerCount ?? 0}명</p>
      {pub.error && <p role="alert">{pub.error}</p>}
      {pub.status === 'confirm-takeover' && (
        <p role="alert">
          다른 기기에서 송출 중이에요. 이 기기로 전환할까요?{' '}
          <button onClick={() => void pub.confirmTakeover()}>전환</button>{' '}
          <button onClick={pub.cancelTakeover}>취소</button>
        </p>
      )}
      {pub.status === 'displaced' && <p role="alert">다른 기기로 전환됐어요.</p>}
      <button onClick={() => void pub.start()}>송출 시작</button>{' '}
      <button onClick={() => void pub.publishScreen(true)}>화면 공유</button>{' '}
      <button onClick={() => void pub.stop()}>방송 종료</button>
      <p>송출 중 소스: {pub.published.join(', ') || '없음'}</p>
    </main>
  )
}
