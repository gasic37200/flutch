import { useParams } from 'react-router'
import ClutchGauge from '../components/ui/ClutchGauge.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Panel from '../components/ui/Panel.tsx'
import Tabs from '../components/ui/Tabs.tsx'
import Player from '../features/watch/components/Player.tsx'
import WatchInfo from '../features/watch/components/WatchInfo.tsx'
import { usePlayer } from '../features/watch/hooks/usePlayer.ts'
import { useWatch } from '../features/watch/hooks/useWatch.ts'
import { errorKindOf } from '../hooks/errorKind.ts'
import { useMediaQuery } from '../hooks/useMediaQuery.ts'
import ShellBody from '../layouts/ShellBody.tsx'

// 채팅 자리(T17): 데스크톱은 오른쪽 패널, 모바일은 아래 탭. 한 곳만 렌더하므로 T17은 이 컴포넌트를 교체하면 된다.
function ChatSlot() {
  return (
    <Panel aria-label='채팅' className='h-full' innerClassName='flex h-full min-h-64 flex-col xl:min-h-175'>
      <div className='hidden border-b border-surface-2 px-4 py-3.5 xl:block'>
        <b className='text-[15px]'>채팅</b>
      </div>
      <p className='flex grow items-center justify-center p-4 text-sm text-text-dim'>채팅은 곧 열려요</p>
    </Panel>
  )
}

// 슬롯: 클러치 버튼·SUPER BURST(T19·T24)는 게이지 Panel 오른쪽, 후원하기(T23)·팔로우(T27)는 WatchInfo 오른쪽 끝, 후원 목록(T23)은 모바일 탭 추가 + 데스크톱 영상 아래 패널.
export default function WatchPage() {
  const id = Number(useParams().broadcastId) || undefined
  const { broadcast, error, refetch, view, viewer } = useWatch(id)
  const player = usePlayer(viewer.tracks, viewer.muted)
  const desktop = useMediaQuery('(min-width: 1280px)')

  if (id === undefined) return <ErrorScreen kind='notFound' />
  // 재생 중 백그라운드 재조회가 실패해도 화면을 갈아엎지 않는다(삭제된 방송 404만 예외).
  const e = error ? errorKindOf(error) : undefined
  if (e && (!broadcast || e.kind === 'notFound')) {
    return <ErrorScreen {...e} onRetry={e.kind === 'notFound' ? undefined : () => void refetch()} />
  }

  return (
    <ShellBody aside={desktop ? <ChatSlot /> : undefined}>
      <div className='flex flex-col gap-3.5'>
        <Player view={view} broadcast={broadcast} player={player} onUnmute={() => void viewer.unmute()} />
        {broadcast && (
          <>
            <Panel innerClassName='px-3 py-2.5 md:px-4 md:py-3'>
              <ClutchGauge percent={broadcast.gauge.percent} size='lg' />
            </Panel>
            <WatchInfo broadcast={broadcast} />
          </>
        )}
        {!desktop && (
          <div>
            <Tabs label='채팅' tabs={[{ id: 'chat', label: '채팅' }]} value='chat' onChange={() => {}} />
            <div role='tabpanel' className='pt-3'>
              <ChatSlot />
            </div>
          </div>
        )}
      </div>
    </ShellBody>
  )
}
