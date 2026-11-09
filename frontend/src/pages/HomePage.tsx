import Button from '../components/ui/Button.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import BroadcastGrid, { BroadcastCard, CategoryChips, GridSkeleton } from '../features/broadcast/components/BroadcastGrid.tsx'
import { useHome } from '../features/broadcast/hooks/useHome.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

// 히어로·SUPER CLUTCH 패널(T29)과 CLUTCH FEED(T30)는 이후 task에서 이 위에 붙는다.
export default function HomePage() {
  const { category, setCategory, grid, recommended } = useHome()
  const empty = (
    <EmptyState
      title='지금은 조용하네요'
      description={category ? '이 카테고리에는 라이브 중인 방송이 없어요.' : '첫 번째 클러치를 만들어 보세요.'}
      action={<Button to='/broadcasts/new'>방송하기</Button>}
    />
  )

  return (
    <div className='space-y-8'>
      <h1 className='sr-only'>FLUTCH 홈</h1>

      {recommended.length > 0 && (
        <section aria-labelledby='home-recommended'>
          <h2 id='home-recommended' className='mb-3 font-display text-lg font-bold tracking-wide'>
            추천 라이브 채널
          </h2>
          <ul className='grid gap-4 sm:grid-cols-2 xl:grid-cols-3'>
            {recommended.map((b) => (
              <li key={b.id}>
                <BroadcastCard broadcast={b} />
              </li>
            ))}
          </ul>
        </section>
      )}

      <section aria-labelledby='home-live'>
        <div className='mb-3 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between'>
          <h2 id='home-live' className='font-display text-lg font-bold tracking-wide'>
            지금 두근거리는 중!
          </h2>
          <CategoryChips value={category} onChange={setCategory} />
        </div>
        {grid.isError ? (
          <ErrorScreen {...errorKindOf(grid.error)} onRetry={() => void grid.refetch()} />
        ) : grid.isPending ? (
          <GridSkeleton />
        ) : (
          <BroadcastGrid items={grid.data.items} empty={empty} />
        )}
      </section>
    </div>
  )
}
