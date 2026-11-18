import type { ReactNode } from 'react'
import { Link } from 'react-router'
import Avatar from '../components/ui/Avatar.tsx'
import Button from '../components/ui/Button.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Icon from '../components/ui/Icon.tsx'
import Panel from '../components/ui/Panel.tsx'
import { useAuth } from '../features/auth/hooks/useAuth.ts'
import { BroadcastCard, GridSkeleton } from '../features/broadcast/components/BroadcastGrid.tsx'
import UnfollowButton from '../features/follow/components/UnfollowButton.tsx'
import { usePulse } from '../features/follow/hooks/useFollow.ts'
import { count } from '../features/replay/format.ts'
import { errorKindOf } from '../hooks/errorKind.ts'
import { kstStamp } from '../lib/kst.ts'

function PulseTitle({ children }: { children?: ReactNode }) {
  return (
    <div className='flex flex-wrap items-center gap-2.5'>
      <Icon name='pulse' className='size-7 text-brand' />
      <h1 className='font-display text-2xl font-bold tracking-[.04em] italic min-[601px]:text-[30px]'>PULSE</h1>
      {children}
    </div>
  )
}

/** 섹션 제목: 상태 점(빨강 = 라이브, 회색 = 오프라인) + 제목 + 개수. */
function StateTitle({ id, dot, count: n, children }: { id: string; dot: string; count: number; children: string }) {
  return (
    <div className='flex items-center gap-2.5'>
      <span className={`size-2.5 rounded-full ${dot}`} aria-hidden />
      <h2 id={id} className='text-lg font-black'>
        {children}
      </h2>
      <span className='font-mono text-[13px] text-text-muted'>{n}</span>
    </div>
  )
}

export default function PulsePage() {
  const { me, isPending } = useAuth()
  const pulse = usePulse(50, me !== null)
  if (isPending) return <GridSkeleton count={4} />
  if (me === null) {
    return (
      <div className='flex flex-col gap-6 lg:gap-8'>
        <PulseTitle />
        <EmptyState title='로그인하면 팔로우한 채널을 볼 수 있어요' action={<Button to='/login?next=/pulse'>로그인</Button>} />
      </div>
    )
  }
  if (pulse.isError) return <ErrorScreen {...errorKindOf(pulse.error)} onRetry={() => void pulse.refetch()} />
  if (pulse.isPending) return <GridSkeleton count={4} />
  const { live, offline, total } = pulse.data
  return (
    <div className='flex flex-col gap-7 min-[601px]:gap-8'>
      <PulseTitle>
        <span className='text-sm text-text-muted'>
          내가 팔로우한 채널 <b className='font-mono text-text'>{total}</b>
        </span>
      </PulseTitle>
      {total === 0 ? (
        <EmptyState title='아직 팔로우한 채널이 없어요' description='마음에 드는 스트리머를 팔로우하면 여기에 모여요.' action={<Button to='/explore'>방송 탐색하기</Button>} />
      ) : (
        <>
          <section aria-labelledby='pulse-live' className='flex flex-col gap-3.5'>
            <StateTitle id='pulse-live' dot='bg-live' count={live.length}>
              지금 방송 중
            </StateTitle>
            {live.length === 0 ? (
              <p className='text-sm text-text-dim'>팔로우한 채널 중 방송 중인 곳이 없어요</p>
            ) : (
              <ul className='grid grid-cols-1 gap-x-4 gap-y-6 min-[601px]:grid-cols-[repeat(auto-fill,minmax(280px,1fr))]'>
                {live.map((b) => (
                  <li key={b.id}>
                    <BroadcastCard broadcast={b} />
                  </li>
                ))}
              </ul>
            )}
          </section>
          {offline.length > 0 && (
            <section aria-labelledby='pulse-offline' className='flex max-w-[880px] flex-col gap-3.5'>
              <StateTitle id='pulse-offline' dot='bg-text-dim' count={offline.length}>
                오프라인
              </StateTitle>
              <Panel>
                <ul>
                  {offline.map((o) => (
                    <li key={o.user.id} className='flex flex-wrap items-center gap-3.5 border-b border-surface-2 px-[18px] py-3'>
                      <Avatar userId={o.user.id} nickname={o.user.nickname} size='md' className='size-11! opacity-55' />
                      <Link to={`/users/${o.user.id}`} className='flex min-w-0 grow basis-40 flex-col gap-0.5'>
                        <b className='truncate text-[15px]'>{o.user.nickname}</b>
                        <span className='text-xs text-text-dim'>
                          팔로워 {count(o.followerCount)} · {o.lastEndedAt ? `마지막 방송 ${kstStamp(o.lastEndedAt)}` : '방송 기록 없음'}
                        </span>
                      </Link>
                      <div className='flex gap-2 max-[600px]:w-full max-[600px]:justify-end'>
                        <Link to={`/users/${o.user.id}`} className='chamfer inline-flex h-9 items-center bg-surface-2 px-3 text-[13px] [--c:6px] hover:brightness-125'>
                          다시보기
                        </Link>
                        <UnfollowButton userId={o.user.id} nickname={o.user.nickname} />
                      </div>
                    </li>
                  ))}
                </ul>
              </Panel>
            </section>
          )}
        </>
      )}
    </div>
  )
}
