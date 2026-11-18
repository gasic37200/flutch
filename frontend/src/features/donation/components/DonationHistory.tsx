import { useState } from 'react'
import type { UserSummary } from '../../../api/generated/UserSummary.ts'
import Badge from '../../../components/ui/Badge.tsx'
import BeatAmount from '../../../components/ui/BeatAmount.tsx'
import Button from '../../../components/ui/Button.tsx'
import EmptyState from '../../../components/ui/EmptyState.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import ViewerPopup from '../../moderation/components/ViewerPopup.tsx'
import { kstStamp } from '../../wallet/format.ts'
import { useDonationList } from '../hooks/useDonationList.ts'
import { useDonationStats } from '../hooks/useDonationStats.ts'
import { beats, BIG_AMOUNT, donorName } from '../format.ts'

const th = 'px-2 py-3 text-left text-xs font-medium text-text-dim first:pl-3.5 last:pr-3.5'
const td = 'px-2 py-2.5 first:pl-3.5 last:pr-3.5'

// 관리 탭 "후원 내역": 기본은 이번(가장 최근) 방송, 방송을 바꿀 수 있다(익명도 실명). 후원자는 바로 차단할 수 있다(ViewerPopup = useBanActions).
export default function DonationHistory({ ownerId, headingClass }: { ownerId: number; headingClass: string }) {
  const stats = useDonationStats()
  const [picked, setPicked] = useState<number>()
  const [target, setTarget] = useState<UserSummary>()
  const selected = picked ?? stats.items[0]?.broadcast.id
  const list = useDonationList(selected)
  const latest = selected === stats.items[0]?.broadcast.id

  if (stats.query.isPending) return <Skeleton className='h-40 w-full' card />
  if (stats.items.length === 0)
    return (
      <>
        <h2 className={`${headingClass} mt-3`}>후원 내역</h2>
        <EmptyState title='받은 후원이 없어요' description='방송을 시작하면 여기에서 방송별 후원 내역을 볼 수 있어요.' />
      </>
    )

  return (
    <>
      <div className='mt-3 flex flex-wrap items-center gap-x-3 gap-y-2'>
        <h2 className={`${headingClass} mr-auto`}>후원 내역{latest && ' · 이번 방송'}</h2>
        <label htmlFor='donation-broadcast' className='sr-only'>
          방송 선택
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
      </div>

      <Panel innerClassName='overflow-x-auto'>
        {list.query.isPending ? (
          <Skeleton className='m-4 h-16' />
        ) : list.query.isError ? (
          <p className='py-6 text-center text-sm text-text-dim'>후원 내역을 불러오지 못했어요.</p>
        ) : list.items.length === 0 ? (
          <p className='py-6 text-center text-sm text-text-dim'>이 방송에는 받은 후원이 없어요.</p>
        ) : (
          <table className='w-full min-w-[560px] border-collapse text-sm'>
            <thead>
              <tr>
                <th className={th}>후원자</th>
                <th className={th}>유형</th>
                <th className={th}>비트</th>
                <th className={th}>메시지</th>
                <th className={th} />
              </tr>
            </thead>
            <tbody>
              {list.items.map((d) => (
                <tr key={d.id} className='border-t border-surface-2'>
                  <td className={td}>
                    <b className='break-all'>{donorName(d)}</b>
                    {d.anonymous && d.donor && <Badge tone='anonymous' className='ml-1.5' />}
                  </td>
                  <td className={`${td} text-text-muted`}>{d.kind === 'burst' ? 'SUPER BURST' : '후원'}</td>
                  <td className={`${td} font-mono font-bold ${d.amount >= BIG_AMOUNT ? 'text-burst' : 'text-brand'}`}>{beats(d.amount)}</td>
                  <td className={`${td} break-words text-text-muted`}>{d.message || '—'}</td>
                  <td className={`${td} text-right`}>
                    {d.donor && (
                      <Button size='sm' variant='neutral' onClick={() => d.donor && setTarget(d.donor)} aria-label={`${d.donor.nickname}님 차단 관리`}>
                        차단
                      </Button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {list.query.hasNextPage && (
          <div className='flex justify-center border-t border-surface-2 py-2.5'>
            <Button size='sm' variant='neutral' onClick={() => void list.query.fetchNextPage()} loading={list.query.isFetchingNextPage}>
              더 보기
            </Button>
          </div>
        )}
      </Panel>
      <p className='flex flex-wrap items-center gap-x-2 text-xs text-text-dim'>
        합계 <BeatAmount amount={list.total} unit className='text-xs' /> · {list.count}건 · 유료 클러치는 합계에만 들어가고 목록에는 건별로 나오지 않아요.
      </p>
      {target && <ViewerPopup user={target} badges={[]} ownerId={ownerId} onClose={() => setTarget(undefined)} />}
    </>
  )
}
