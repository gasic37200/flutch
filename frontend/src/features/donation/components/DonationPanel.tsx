import BeatAmount from '../../../components/ui/BeatAmount.tsx'
import Button from '../../../components/ui/Button.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import type { useDonationList } from '../hooks/useDonationList.ts'
import DonationRow from './DonationRow.tsx'

// 시청 화면의 "이 방송 후원": 합계(유료 클러치·버스트 포함) + 최근 후원 목록. 데스크톱은 영상 아래, 모바일은 "후원" 탭.
export default function DonationPanel({ list }: { list: ReturnType<typeof useDonationList> }) {
  const { query, items, total, count } = list
  return (
    <Panel aria-label='이 방송 후원' innerClassName='flex flex-col'>
      <div className='flex flex-wrap items-center gap-x-2.5 gap-y-1 border-b border-surface-2 px-4 py-3'>
        <b className='text-[15px]'>이 방송 후원</b>
        <span className='text-[13px] text-text-muted'>합계</span>
        <BeatAmount amount={total} unit className='text-sm' />
        <span className='text-[13px] text-text-dim'>· {count.toLocaleString('ko-KR')}건 (클러치·버스트 포함)</span>
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
          <ul className='max-h-72 overflow-y-auto'>
            {items.map((d) => (
              <DonationRow key={d.id} donation={d} />
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
