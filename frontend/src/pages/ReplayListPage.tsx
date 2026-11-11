import { useSearchParams } from 'react-router'
import type { Category } from '../api/generated/Category.ts'
import EmptyState from '../components/ui/EmptyState.tsx'
import { CATEGORIES } from '../features/broadcast/categories.ts'
import { CategoryChips } from '../features/broadcast/components/BroadcastGrid.tsx'
import ReplayGrid from '../features/replay/components/ReplayGrid.tsx'
import { useRecordings } from '../features/replay/hooks/useRecordings.ts'

export default function ReplayListPage() {
  // 새로고침·공유해도 필터가 남도록 ?category=에 둔다
  const [sp, setSp] = useSearchParams()
  const param = sp.get('category')
  const category = CATEGORIES.find((c) => c === param)
  const setCategory = (c: Category | undefined) => setSp(c ? { category: c } : {}, { replace: true })
  const query = useRecordings({ category })
  return (
    <div className='flex flex-col gap-5 lg:gap-6'>
      <h1 className='text-2xl font-black sm:text-[26px]'>다시보기</h1>
      <CategoryChips value={category} onChange={setCategory} />
      <ReplayGrid
        query={query}
        empty={<EmptyState title='다시보기가 없어요' description={category ? '이 카테고리에는 다시보기가 없어요.' : '방송이 끝나고 녹화가 준비되면 여기에 모여요.'} />}
      />
    </div>
  )
}
