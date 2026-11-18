import { useSearchParams } from 'react-router'
import type { Category } from '../api/generated/Category.ts'
import EmptyState from '../components/ui/EmptyState.tsx'
import { CATEGORIES, CATEGORY_LABEL } from '../features/broadcast/categories.ts'
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
    <div className='flex flex-col gap-4'>
      <div className='flex items-center gap-2'>
        <span className='mr-0.5 size-2.5 rounded-full bg-brand' aria-hidden />
        <h1 className='mr-auto text-xl font-black'>다시보기</h1>
        {/* 정렬은 최신순뿐이라 표시만 한다 */}
        <span className='chamfer inline-flex h-9 items-center bg-brand px-4 text-sm font-bold text-bg [--c:6px]'>최신순</span>
        <label className='sr-only' htmlFor='replay-category'>
          카테고리
        </label>
        <select
          id='replay-category'
          value={category ?? ''}
          onChange={(e) => setCategory(CATEGORIES.find((c) => c === e.target.value))}
          className='chamfer h-9 bg-surface-2 px-3 text-sm text-text-muted [--c:6px]'
        >
          <option value=''>카테고리</option>
          {CATEGORIES.map((c) => (
            <option key={c} value={c}>
              {CATEGORY_LABEL[c]}
            </option>
          ))}
        </select>
      </div>
      <ReplayGrid
        query={query}
        empty={<EmptyState title='다시보기가 없어요' description={category ? '이 카테고리에는 다시보기가 없어요.' : '방송이 끝나고 녹화가 준비되면 여기에 모여요.'} />}
      />
    </div>
  )
}
