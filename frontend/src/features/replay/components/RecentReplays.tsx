import Button from '../../../components/ui/Button.tsx'
import { SectionTitle } from '../../broadcast/components/BroadcastGrid.tsx'
import { useRecordings } from '../hooks/useRecordings.ts'
import { ReplayList } from './ReplayGrid.tsx'

/** 홈 "최근 다시보기"(CLUTCH FEED T30 전까지 이 자리를 쓴다). 없으면 섹션째 숨긴다. */
export default function RecentReplays() {
  const q = useRecordings({ limit: 4 })
  const items = q.data?.pages[0]?.items ?? []
  if (items.length === 0) return null
  return (
    <section aria-labelledby='home-replay' className='flex flex-col gap-3 lg:gap-4'>
      <div className='flex items-center justify-between gap-3'>
        <SectionTitle id='home-replay'>최근 다시보기</SectionTitle>
        <Button variant='neutral' size='sm' to='/replay'>
          전체 보기
        </Button>
      </div>
      <ReplayList items={items} />
    </section>
  )
}
