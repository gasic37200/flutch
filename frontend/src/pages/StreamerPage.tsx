import { useParams } from 'react-router'
import Avatar from '../components/ui/Avatar.tsx'
import Button from '../components/ui/Button.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Skeleton from '../components/ui/Skeleton.tsx'
import Tabs from '../components/ui/Tabs.tsx'
import { BroadcastCard, GridSkeleton, SectionTitle } from '../features/broadcast/components/BroadcastGrid.tsx'
import DonationHistory from '../features/donation/components/DonationHistory.tsx'
import DonationStatsPanel from '../features/donation/components/DonationStatsPanel.tsx'
import BanList from '../features/moderation/components/BanList.tsx'
import { gridClass } from '../features/replay/components/ReplayGrid.tsx'
import ReplayGrid from '../features/replay/components/ReplayGrid.tsx'
import FollowButton from '../features/follow/components/FollowButton.tsx'
import { count } from '../features/replay/format.ts'
import PastBroadcasts from '../features/streamer/components/PastBroadcasts.tsx'
import { useStreamer, type StreamerTab } from '../features/streamer/hooks/useStreamer.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

// 팔로우 버튼(T27)은 이름 줄 오른쪽 버튼 자리, 관리 탭(후원 내역·통계·차단 목록)은 본인에게만 보인다(서버도 본인만 응답).
// 주소의 id만 바뀌면 같은 인스턴스가 남아 이전 방송 상태가 섞이므로 id를 key로 다시 마운트한다.
export default function StreamerPage() {
  return <StreamerPageBody key={useParams().userId} />
}

function StreamerPageBody() {
  const id = Number(useParams().userId) || undefined
  const s = useStreamer(id)
  const { profile, past, replays } = s

  if (id === undefined) return <ErrorScreen kind='notFound' />
  if (profile.isError) return <ErrorScreen {...errorKindOf(profile.error)} onRetry={() => void profile.refetch()} />
  if (profile.isPending) return <Skeleton className='h-40 w-full max-w-5xl' />

  const { user, followerCount, liveBroadcast } = profile.data
  const pastItems = past.data?.pages.flatMap((p) => p.items) ?? []
  const tabs: { id: StreamerTab; label: string }[] = [
    { id: 'broadcasts', label: '방송' },
    { id: 'replays', label: '다시보기' },
    ...(s.isSelf
      ? [
          { id: 'donations' as const, label: '후원 내역' },
          { id: 'stats' as const, label: '통계' },
          { id: 'bans' as const, label: '차단 목록' },
        ]
      : []),
  ]

  return (
    <div className='flex flex-col gap-6'>
      <div className='flex flex-wrap items-end gap-x-4 gap-y-3 sm:gap-x-5'>
        <Avatar userId={user.id} nickname={user.nickname} size='lg' className='sm:size-28' />
        <div className='flex min-w-0 flex-col gap-1.5'>
          <h1 className='text-2xl font-black break-words sm:text-[28px]'>{user.nickname}</h1>
          <span className='text-sm text-text-muted'>
            팔로워 <b className='font-mono text-text'>{count(followerCount)}</b>
            {liveBroadcast && ' · 지금 방송 중'}
          </span>
        </div>
        {s.isSelf ? (
          <Button size='lg' to='/manage'>
            내 방송 관리
          </Button>
        ) : (
          <FollowButton userId={user.id} size='lg' />
        )}
      </div>

      <Tabs label='스트리머 페이지' tabs={tabs} value={s.tab} onChange={(t) => s.setTab(t as StreamerTab)} />

      <div role='tabpanel' className='flex flex-col gap-6'>
        {s.tab === 'broadcasts' ? (
          <>
            {liveBroadcast && (
              <section aria-labelledby='streamer-live' className='flex flex-col gap-3'>
                <SectionTitle id='streamer-live'>지금 방송 중</SectionTitle>
                <ul className={gridClass}>
                  <li>
                    <BroadcastCard broadcast={liveBroadcast} />
                  </li>
                </ul>
              </section>
            )}
            <section aria-labelledby='streamer-past' className='flex flex-col gap-3'>
              <SectionTitle id='streamer-past'>지난 방송</SectionTitle>
              {past.isError ? (
                <ErrorScreen {...errorKindOf(past.error)} onRetry={() => void past.refetch()} />
              ) : past.isPending ? (
                <GridSkeleton count={4} />
              ) : pastItems.length === 0 ? (
                <EmptyState title='아직 지난 방송이 없어요' />
              ) : (
                <>
                  <PastBroadcasts items={pastItems} isSelf={s.isSelf} />
                  {past.hasNextPage && (
                    <div className='flex justify-center'>
                      <Button variant='secondary' onClick={() => void past.fetchNextPage()} loading={past.isFetchingNextPage}>
                        더 보기
                      </Button>
                    </div>
                  )}
                </>
              )}
            </section>
          </>
        ) : s.tab === 'donations' && s.isSelf ? (
          <DonationHistory ownerId={user.id} />
        ) : s.tab === 'stats' && s.isSelf ? (
          <DonationStatsPanel />
        ) : s.tab === 'bans' && s.isSelf ? (
          <BanList ownerId={user.id} />
        ) : (
          <ReplayGrid query={replays} empty={<EmptyState title='다시보기가 없어요' description='녹화가 끝난 방송이 여기에 모여요.' />} />
        )}
      </div>
    </div>
  )
}
