import { useState } from 'react'
import type { Category } from '../api/generated/Category.ts'
import EmptyState from '../components/ui/EmptyState.tsx'
import { CategoryChips } from '../features/broadcast/components/BroadcastGrid.tsx'
import ReplayGrid from '../features/replay/components/ReplayGrid.tsx'
import { useRecordings } from '../features/replay/hooks/useRecordings.ts'

export default function ReplayListPage() {
  const [category, setCategory] = useState<Category | undefined>()
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
