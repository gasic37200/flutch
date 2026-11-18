import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import type { useDonationList } from '../hooks/useDonationList.ts'
import { beats, BIG_AMOUNT, donorName } from '../format.ts'

// 시청 화면의 "이 방송 후원": 합계(유료 클러치·버스트 포함) + 최근 후원 목록. 데스크톱은 영상 아래, 모바일은 "후원" 탭.
export default function DonationPanel({ list }: { list: ReturnType<typeof useDonationList> }) {
  const { query, items, total, count } = list
  return (
    <Panel aria-label='이 방송 후원' innerClassName='flex flex-col'>
      <div className='flex flex-wrap items-baseline gap-x-2.5 gap-y-1 px-4 pt-3.5'>
        <b className='text-[15px]'>이 방송 후원</b>
                <b className='font-mono text-brand'>{beats(total)}</b>
        <span className='font-display text-[13px] font-bold text-brand'>Beat</span>
        <span className='text-[13px] text-text-dim'>{count.toLocaleString('ko-KR')}건 · 클러치·버스트 포함</span>
      </div>
      {query.isError ? (
        <div className='flex flex-col items-center gap-2 py-6 text-text-dim'>
          <p>후원 목록을 불러오지 못했어요.</p>
          <Button size='sm' variant='neutral' onClick={() => void query.refetch()}>
            다시 불러오기
          </Button>
        </div>
      ) : query.isPending ? (
        <Skeleton className='m-4 h-16' />
      ) : items.length === 0 ? (
        <p className='py-6 text-center text-sm text-text-dim'>아직 후원이 없어요.</p>
      ) : (
        <>
          <ul className='flex max-h-72 flex-col gap-1.5 overflow-y-auto px-4 pt-2.5 pb-3.5 text-[13px]'>
            {items.map((d) => (
              <li key={d.id} className='grid grid-cols-[minmax(0,7.5rem)_5rem_minmax(0,1fr)] gap-2.5 max-[600px]:grid-cols-[minmax(0,6rem)_4.5rem_minmax(0,1fr)]'>
                <b className='flex min-w-0 items-center gap-1.5 break-all'>
                  {donorName(d)}
                  {d.anonymous && d.donor && <Badge tone='anonymous' />}
                </b>
                <b className={`font-mono ${d.amount >= BIG_AMOUNT ? 'text-burst' : 'text-brand'}`}>
                  {beats(d.amount)}
                  <span className='sr-only'>비트</span>
                </b>
                <span className='min-w-0 truncate text-text-muted'>{d.message || (d.kind === 'burst' ? 'SUPER BURST' : '')}</span>
              </li>
            ))}
          </ul>
          {query.hasNextPage && (
            <div className='flex justify-center border-t border-surface-2 py-2.5'>
              <Button size='sm' variant='neutral' onClick={() => void query.fetchNextPage()} loading={query.isFetchingNextPage}>
                더 보기
              </Button>
            </div>
          )}
        </>
      )}
    </Panel>
  )
}
