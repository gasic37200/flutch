import { useParams } from 'react-router'
import BurstButton from '../features/watch/components/BurstButton.tsx'
import ClutchButton from '../features/watch/components/ClutchButton.tsx'
import SuperClutchOverlay from '../features/watch/components/SuperClutchOverlay.tsx'
import { useBurst } from '../features/watch/hooks/useBurst.ts'
import { useClutch } from '../features/watch/hooks/useClutch.ts'
import ClutchGauge from '../components/ui/ClutchGauge.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Panel from '../components/ui/Panel.tsx'
import Tabs from '../components/ui/Tabs.tsx'
import ChatCard from '../features/chat/components/ChatCard.tsx'
import { useChat } from '../features/chat/hooks/useChat.ts'
import Player from '../features/watch/components/Player.tsx'
import WatchInfo from '../features/watch/components/WatchInfo.tsx'
import { usePlayer } from '../features/watch/hooks/usePlayer.ts'
import { useWatch } from '../features/watch/hooks/useWatch.ts'
import { errorKindOf } from '../hooks/errorKind.ts'
import { useMediaQuery } from '../hooks/useMediaQuery.ts'
import ShellBody from '../layouts/ShellBody.tsx'

// 슬롯: 클러치 버튼·SUPER BURST(T19·T24)는 게이지 Panel 오른쪽, 후원하기(T23)·팔로우(T27)는 WatchInfo 오른쪽 끝, 후원 목록(T23)은 모바일 탭 추가 + 데스크톱 영상 아래 패널.
export default function WatchPage() {
  const id = Number(useParams().broadcastId) || undefined
  const { broadcast, error, refetch, view, viewer, events } = useWatch(id)
  const chat = useChat(id, events)
  const player = usePlayer(viewer.tracks, viewer.muted)
  const clutch = useClutch(broadcast, events.onEvent)
  const burst = useBurst(broadcast, events.onEvent)
  const desktop = useMediaQuery('(min-width: 1280px)')

  if (id === undefined) return <ErrorScreen kind='notFound' />
  // 재생 중 백그라운드 재조회가 실패해도 화면을 갈아엎지 않는다(삭제된 방송 404만 예외).
  const e = error ? errorKindOf(error) : undefined
  if (e && (!broadcast || e.kind === 'notFound')) {
    return <ErrorScreen {...e} onRetry={e.kind === 'notFound' ? undefined : () => void refetch()} />
  }

  // 채팅은 데스크톱 aside 또는 모바일 탭 한 곳에만 렌더한다(상태는 위의 useChat이 들고 있어 전환해도 유지)
  const chatCard = <ChatCard chat={chat} broadcast={broadcast} className='h-[30rem] xl:h-175' headerClassName={desktop ? '' : 'hidden'} />

  return (
    <>
      <SuperClutchOverlay superClutch={clutch.superClutch} />
      <ShellBody aside={desktop ? chatCard : undefined}>
        <div className='flex flex-col gap-3.5'>
          <Player view={view} broadcast={broadcast} player={player} onUnmute={() => void viewer.unmute()} />
          {broadcast && (
            <>
              <Panel innerClassName='px-3 py-2.5 md:px-4 md:py-3'>
                <div className='flex items-center gap-3'>
                  <div className='min-w-0 grow'>
                    <ClutchGauge percent={broadcast.gauge.percent} cooldown={broadcast.gauge.phase === 'cooldown'} boost={burst.boost} size='lg' />
                  </div>
                  <ClutchButton clutch={clutch} />
                  <BurstButton burst={burst} mode={clutch.mode} />
                </div>
              </Panel>
              <WatchInfo broadcast={broadcast} />
            </>
          )}
          {!desktop && (
            <div>
              <Tabs label='채팅' tabs={[{ id: 'chat', label: '채팅' }]} value='chat' onChange={() => {}} />
              <div role='tabpanel' className='pt-3'>
                {chatCard}
              </div>
            </div>
          )}
        </div>
      </ShellBody>
    </>
  )
}
