import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import DonationAlert from '../features/donation/components/DonationAlert.tsx'
import DonationPanel from '../features/donation/components/DonationPanel.tsx'
import { useDonationAlerts } from '../features/donation/hooks/useDonationAlerts.ts'
import { useDonationList } from '../features/donation/hooks/useDonationList.ts'
import ClutchPanel from '../features/watch/components/ClutchPanel.tsx'
import SuperClutchOverlay from '../features/watch/components/SuperClutchOverlay.tsx'
import { useBurst } from '../features/watch/hooks/useBurst.ts'
import { useClutch } from '../features/watch/hooks/useClutch.ts'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Icon from '../components/ui/Icon.tsx'
import ChatCard from '../features/chat/components/ChatCard.tsx'
import { useChat } from '../features/chat/hooks/useChat.ts'
import Player from '../features/watch/components/Player.tsx'
import WatchTabs from '../features/watch/components/WatchTabs.tsx'
import WatchInfo from '../features/watch/components/WatchInfo.tsx'
import { usePlayer } from '../features/watch/hooks/usePlayer.ts'
import { useWatch } from '../features/watch/hooks/useWatch.ts'
import { errorKindOf } from '../hooks/errorKind.ts'
import { useMediaQuery } from '../hooks/useMediaQuery.ts'
import ShellBody from '../layouts/ShellBody.tsx'

// 슬롯: 클러치 버튼·SUPER BURST(T19·T24)는 게이지 Panel 오른쪽, 후원하기(T23)·팔로우(T27)는 WatchInfo 오른쪽 끝, 후원 목록(T23)은 모바일 탭 추가 + 데스크톱 영상 아래 패널.
// 주소의 id만 바뀌면 같은 인스턴스가 남아 이전 방송 상태가 섞이므로 id를 key로 다시 마운트한다.
export default function WatchPage() {
  return <WatchPageBody key={useParams().broadcastId} />
}

function WatchPageBody() {
  const id = Number(useParams().broadcastId) || undefined
  const { broadcast, error, refetch, view, source, viewer, events } = useWatch(id)
  const chat = useChat(id, events)
  const player = usePlayer(viewer.tracks, viewer.muted)
  const clutch = useClutch(broadcast, events.onEvent)
  const burst = useBurst(broadcast, events.onEvent)
  const desktop = useMediaQuery('(min-width: 1024px)')
  const mobile = useMediaQuery('(max-width: 600px)')
  const alerts = useDonationAlerts(events.onEvent, viewer.muted || player.volume === 0)
  const donations = useDonationList(id, { onEvent: events.onEvent, live: broadcast?.status === 'LIVE' })
  const [tab, setTab] = useState('chat')
  const navigate = useNavigate()

  if (id === undefined) return <ErrorScreen kind='notFound' />
  // 재생 중 백그라운드 재조회가 실패해도 화면을 갈아엎지 않는다(삭제된 방송 404만 예외).
  const e = error ? errorKindOf(error) : undefined
  if (e && (!broadcast || e.kind === 'notFound')) {
    return <ErrorScreen {...e} onRetry={e.kind === 'notFound' ? undefined : () => void refetch()} />
  }

  // 채팅은 데스크톱 aside, 601~1023은 후원 패널 아래, 모바일은 탭 중 한 곳에만 렌더한다(상태는 위의 useChat이 들고 있어 전환해도 유지)
  const chatCard = <ChatCard chat={chat} broadcast={broadcast} className={desktop ? 'sticky top-22 h-[calc(100dvh-7rem)] max-h-full min-h-96' : mobile ? 'h-[30rem] max-[600px]:[&>div]:bg-transparent max-[600px]:[&>div]:p-0 max-[600px]:[&>div]:[clip-path:none] max-[600px]:[&>div>div]:[clip-path:none] max-[600px]:[&>div>div]:bg-bg' : 'h-[32.5rem]'} headerClassName={mobile ? 'hidden' : ''} recent />

  return (
    <>
      <SuperClutchOverlay superClutch={clutch.superClutch} />
      <ShellBody aside={desktop ? chatCard : undefined}>
        <div className='flex flex-col gap-3.5 max-[600px]:gap-0'>
          {/* 가로 화면(낮은 뷰포트)에서는 영상이 화면 높이 안에 들어오도록 폭을 줄인다 */}
          <div className='relative mx-auto w-full max-w-[calc((100dvh-5rem)*16/9)]'>
            <Player view={view} source={source} broadcast={broadcast} player={player} tts={alerts.tts} onUnmute={() => void viewer.unmute()} />
            <DonationAlert alerts={alerts} owner={broadcast?.viewer?.isOwner} />
            {/* 600 이하는 헤더를 숨기므로 영상 위에 뒤로가기를 둔다 */}
            <button type='button' aria-label='뒤로 가기' onClick={() => void navigate(-1)} className='absolute top-2 left-2 z-10 flex size-11 items-center justify-center rounded-full bg-bg/70 text-text min-[601px]:hidden'>
              <Icon name='back' />
            </button>
          </div>
          {broadcast && (
            <div className='contents'>
              <ClutchPanel broadcast={broadcast} clutch={clutch} burst={burst} />
              <WatchInfo broadcast={broadcast} />
              {!mobile && <DonationPanel list={donations} />}
            </div>
          )}
          {!desktop && !mobile && chatCard}
          {mobile && (
            <div>
              <WatchTabs
                tabs={[
                  { id: 'chat', label: '채팅' },
                  { id: 'donations', label: <>후원 <span className='font-mono'>{donations.total.toLocaleString('ko-KR')}</span></> },
                ]}
                value={tab}
                onChange={setTab}
              />
              <div role='tabpanel' className={tab === 'donations' ? 'p-3' : ''}>
                {tab === 'donations' ? <DonationPanel list={donations} /> : chatCard}
              </div>
            </div>
          )}
        </div>
      </ShellBody>
    </>
  )
}
