import Button from '../components/ui/Button.tsx'
import { kstStamp } from '../lib/kst.ts'
import Avatar from '../components/ui/Avatar.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import { useAuth } from '../features/auth/hooks/useAuth.ts'
import BroadcastGrid, { GridSkeleton, SectionTitle } from '../features/broadcast/components/BroadcastGrid.tsx'
import { usePulse } from '../features/follow/hooks/useFollow.ts'
import { count } from '../features/replay/format.ts'
import { errorKindOf } from '../hooks/errorKind.ts'
import { Link } from 'react-router'


export default function PulsePage() {
  const { me, isPending } = useAuth()
  const pulse = usePulse(50, me !== null)
  if (isPending) return <GridSkeleton count={4} />
  if (me === null) {
    return (
      <div className='flex flex-col gap-6 lg:gap-8'>
        <h1 className='text-2xl font-black sm:text-[26px]'>PULSE</h1>
        <EmptyState title='로그인하면 팔로우한 채널을 볼 수 있어요' action={<Button to='/login?next=/pulse'>로그인</Button>} />
      </div>
    )
  }
  if (pulse.isError) return <ErrorScreen {...errorKindOf(pulse.error)} onRetry={() => void pulse.refetch()} />
  if (pulse.isPending) return <GridSkeleton count={4} />
  const { live, offline, total } = pulse.data
  return (
    <div className='flex flex-col gap-6 lg:gap-8'>
      <div className='flex flex-col gap-1'>
        <h1 className='text-2xl font-black sm:text-[26px]'>PULSE</h1>
        <p className='text-sm text-text-muted'>
          내가 팔로우한 채널 <b className='font-mono text-text'>{total}</b>
        </p>
      </div>
      {total === 0 ? (
        <EmptyState title='아직 팔로우한 채널이 없어요' description='마음에 드는 스트리머를 팔로우하면 여기에 모여요.' action={<Button to='/explore'>방송 탐색하기</Button>} />
      ) : (
        <>
          <section aria-labelledby='pulse-live' className='flex flex-col gap-3'>
            <SectionTitle id='pulse-live'>지금 라이브</SectionTitle>
            <BroadcastGrid items={live} empty={<p className='text-sm text-text-dim'>팔로우한 채널 중 방송 중인 곳이 없어요</p>} />
          </section>
          {offline.length > 0 && (
            <section aria-labelledby='pulse-offline' className='flex flex-col gap-3'>
              <SectionTitle id='pulse-offline'>오프라인</SectionTitle>
              <ul className='grid grid-cols-1 gap-3 min-[601px]:grid-cols-[repeat(auto-fill,minmax(280px,1fr))]'>
                {offline.map((o) => (
                  <li key={o.user.id}>
                    <Link to={`/users/${o.user.id}`} className='flex items-center gap-3 bg-surface p-3 hover:bg-surface-2'>
                      <Avatar userId={o.user.id} nickname={o.user.nickname} size='md' />
                      <span className='min-w-0 grow'>
                        <b className='block truncate text-sm'>{o.user.nickname}</b>
                        <span className='text-xs text-text-dim'>
                          팔로워 {count(o.followerCount)} · {o.lastEndedAt ? `마지막 방송 ${kstStamp(o.lastEndedAt)}` : '방송 기록 없음'}
                        </span>
                      </span>
                    </Link>
                  </li>
                ))}
              </ul>
            </section>
          )}
        </>
      )}
    </div>
  )
}
