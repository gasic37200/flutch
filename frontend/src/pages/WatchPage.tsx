import { useState } from 'react'
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
import ShellBody from '../layouts/ShellBody.tsx'

const TABS = [
  { id: 'chat', label: '채팅' },
  { id: 'donate', label: '후원' },
]

// 슬롯: 채팅 패널(T17)은 `chatSlot`, 후원 목록(T23)은 모바일 후원 탭, 클러치 버튼·SUPER BURST(T19·T23)는 게이지 바 오른쪽, 팔로우·후원하기는 WatchInfo.
export default function WatchPage() {
  const id = Number(useParams().broadcastId) || undefined
  const { broadcast, error, refetch, view, viewer } = useWatch(id)
  const player = usePlayer(viewer.tracks, viewer.muted)
  const [tab, setTab] = useState('chat')

  if (id === undefined) return <ErrorScreen kind='notFound' />
  if (error) {
    const e = errorKindOf(error)
    return <ErrorScreen {...e} onRetry={e.kind === 'notFound' ? undefined : () => void refetch()} />
  }

  const chatSlot = (
    <Panel className='h-full' innerClassName='flex h-full min-h-64 items-center justify-center p-4 text-sm text-text-dim'>
      채팅이 여기에 표시돼요
    </Panel>
  )

  return (
    <ShellBody aside={<div className='hidden h-full xl:block'>{chatSlot}</div>}>
      <div className='flex flex-col gap-4'>
        <Player view={view} broadcast={broadcast} player={player} onUnmute={() => void viewer.unmute()} />
        {broadcast && (
          <>
            <Panel size='sm' innerClassName='p-3'>
              <ClutchGauge percent={broadcast.gauge.percent} />
            </Panel>
            <WatchInfo broadcast={broadcast} />
          </>
        )}
        <div className='xl:hidden'>
          <Tabs label='채팅·후원' tabs={TABS} value={tab} onChange={setTab} />
          <div role='tabpanel' className='pt-3'>
            {tab === 'chat' ? chatSlot : <p className='p-4 text-sm text-text-dim'>후원 목록이 여기에 표시돼요</p>}
          </div>
        </div>
      </div>
    </ShellBody>
  )
}
