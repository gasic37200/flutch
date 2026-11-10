import { useState } from 'react'
import type { UserSummary } from '../../../api/generated/UserSummary.ts'
import BeatAmount from '../../../components/ui/BeatAmount.tsx'
import Button from '../../../components/ui/Button.tsx'
import EmptyState from '../../../components/ui/EmptyState.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import ViewerPopup from '../../moderation/components/ViewerPopup.tsx'
import { kstStamp } from '../../wallet/format.ts'
import { useDonationList } from '../hooks/useDonationList.ts'
import { useDonationStats } from '../hooks/useDonationStats.ts'
import DonationRow from './DonationRow.tsx'

// 관리 탭 "후원 내역": 방송을 고르면 그 방송의 후원 전체(익명도 실명). 후원자는 바로 차단할 수 있다(ViewerPopup = useBanActions).
export default function DonationHistory({ ownerId }: { ownerId: number }) {
  const stats = useDonationStats()
  const [picked, setPicked] = useState<number>()
  const [target, setTarget] = useState<UserSummary>()
  const selected = picked ?? stats.items[0]?.broadcast.id
  const list = useDonationList(selected)

  if (stats.query.isPending) return <Skeleton className='h-40 w-full' card />
  if (stats.items.length === 0) return <EmptyState title='받은 후원이 없어요' description='방송을 시작하면 여기에서 방송별 후원 내역을 볼 수 있어요.' />

  return (
    <div className='flex flex-col gap-3.5'>
      <div className='flex flex-wrap items-center gap-2.5'>
        <label htmlFor='donation-broadcast' className='text-sm font-semibold text-text-muted'>
          방송
        </label>
        <select
          id='donation-broadcast'
          value={selected}
          onChange={(e) => setPicked(Number(e.target.value))}
          className='chamfer h-9 max-w-full min-w-0 bg-surface-2 px-2.5 text-[13px] text-text [--c:6px]'
        >
          {stats.items.map(({ broadcast: b }) => (
            <option key={b.id} value={b.id}>
              {b.title} · {kstStamp(b.startedAt)}
            </option>
          ))}
        </select>
        <span className='ml-auto flex items-center gap-2 text-[13px] text-text-muted'>
          합계 <BeatAmount amount={list.total} unit className='text-sm' />
          <span className='text-text-dim'>· {list.count}건</span>
        </span>
      </div>

      <Panel innerClassName='flex flex-col'>
        {list.query.isPending ? (
          <Skeleton className='m-4 h-16' />
        ) : list.query.isError ? (
          <p className='py-6 text-center text-sm text-text-dim'>후원 내역을 불러오지 못했어요.</p>
        ) : list.items.length === 0 ? (
          <p className='py-6 text-center text-sm text-text-dim'>이 방송에는 받은 후원이 없어요.</p>
        ) : (
          <ul>
            {list.items.map((d) => (
              <DonationRow
                key={d.id}
                donation={d}
                action={
                  d.donor && (
                    <Button size='sm' variant='neutral' onClick={() => d.donor && setTarget(d.donor)} aria-label={`${d.donor.nickname}님 차단 관리`}>
                      차단
                    </Button>
                  )
                }
              />
            ))}
          </ul>
        )}
        {list.query.hasNextPage && (
          <div className='flex justify-center border-t border-surface-2 py-2.5'>
            <Button size='sm' variant='neutral' onClick={() => void list.query.fetchNextPage()} loading={list.query.isFetchingNextPage}>
              더 보기
            </Button>
          </div>
        )}
      </Panel>
      <p className='text-xs text-text-dim'>유료 클러치는 합계에만 들어가고 목록에는 건별로 나오지 않아요.</p>
      {target && <ViewerPopup user={target} badges={[]} ownerId={ownerId} onClose={() => setTarget(undefined)} />}
    </div>
  )
}
