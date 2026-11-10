import BeatAmount from '../../../components/ui/BeatAmount.tsx'
import Button from '../../../components/ui/Button.tsx'
import EmptyState from '../../../components/ui/EmptyState.tsx'
import ErrorScreen from '../../../components/ui/ErrorScreen.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import { errorKindOf } from '../../../hooks/errorKind.ts'
import { kstStamp } from '../../wallet/format.ts'
import { useDonationStats } from '../hooks/useDonationStats.ts'

// 관리 탭 "통계": 누적 받은 비트 · 최근 7일(KST) · 방송별 합계. 후원자 순위는 없다.
export default function DonationStatsPanel() {
  const s = useDonationStats()
  const { query } = s
  if (query.isError) return <ErrorScreen {...errorKindOf(query.error)} onRetry={() => void query.refetch()} />
  if (query.isPending) return <Skeleton className='h-40 w-full' card />
  return (
    <div className='flex flex-col gap-4'>
      <dl className='grid gap-3 sm:grid-cols-2'>
        {[
          { label: '누적 받은 비트', amount: s.totalReceived },
          { label: '최근 7일', amount: s.last7Days },
        ].map((c) => (
          <Panel key={c.label} innerClassName='flex flex-col gap-1 p-4'>
            <dt className='text-[13px] text-text-muted'>{c.label}</dt>
            <dd>
              <BeatAmount amount={c.amount} unit className='text-2xl' />
            </dd>
          </Panel>
        ))}
      </dl>
      <h2 className='text-lg font-black'>방송별 받은 비트</h2>
      {s.items.length === 0 ? (
        <EmptyState title='아직 통계가 없어요' description='방송을 하고 후원을 받으면 방송별로 모여요.' />
      ) : (
        <Panel innerClassName='flex flex-col'>
          <ul>
            {s.items.map(({ broadcast: b, total, count }) => (
              <li key={b.id} className='flex items-center gap-3 border-b border-surface-2 px-4 py-3 last:border-b-0'>
                <div className='flex min-w-0 grow flex-col gap-0.5'>
                  <b className='truncate text-sm'>{b.title}</b>
                  <time dateTime={b.startedAt} className='text-xs text-text-dim'>
                    {kstStamp(b.startedAt)}
                  </time>
                </div>
                <span className='shrink-0 text-xs text-text-dim'>{count}건</span>
                <BeatAmount amount={total} className='shrink-0 text-sm' />
              </li>
            ))}
          </ul>
        </Panel>
      )}
      {query.hasNextPage && (
        <div className='flex justify-center'>
          <Button variant='secondary' onClick={() => void query.fetchNextPage()} loading={query.isFetchingNextPage}>
            더 보기
          </Button>
        </div>
      )}
    </div>
  )
}
