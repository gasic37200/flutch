import BeatAmount from '../../../components/ui/BeatAmount.tsx'
import Button from '../../../components/ui/Button.tsx'
import DonationLine from '../../donation/components/DonationLine.tsx'
import { useDonationList } from '../../donation/hooks/useDonationList.ts'

// 다시보기 "후원" 탭: 방송이 받은 후원 합계 + 목록. 익명은 서버가 가려서 주고, 방송 소유자 세션이면 실명이다.
export default function ReplayDonations({ broadcastId }: { broadcastId: number }) {
  const list = useDonationList(broadcastId)
  const q = list.query
  if (q.isError) return <p className='m-auto text-text-dim'>후원 내역을 불러오지 못했어요</p>
  if (q.isPending) return <p className='m-auto text-text-dim'>불러오는 중…</p>
  return (
    <>
      <p className='flex items-center gap-2 text-text-muted'>
        받은 후원 <BeatAmount amount={list.total} unit className='text-sm' />
        <span className='text-text-dim'>· {list.count}건</span>
      </p>
      {list.items.length === 0 ? (
        <p className='m-auto text-text-dim'>받은 후원이 없어요</p>
      ) : (
        <ul className='flex flex-col gap-1.5'>
          {list.items.map((d) => (
            <DonationLine key={d.id} donation={d} />
          ))}
        </ul>
      )}
      {q.hasNextPage && (
        <Button variant='neutral' size='sm' onClick={() => void q.fetchNextPage()} loading={q.isFetchingNextPage}>
          후원 더 보기
        </Button>
      )}
      {list.clutchTotal > 0 && <p className='text-xs text-text-dim'>유료 클러치 {list.clutchTotal.toLocaleString('ko-KR')}비트는 합계에만 들어가요.</p>}
    </>
  )
}
