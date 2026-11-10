import Button from '../components/ui/Button.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import BroadcastGrid, { BroadcastCard, CategoryChips, GridSkeleton, SectionTitle } from '../features/broadcast/components/BroadcastGrid.tsx'
import HomeHero from '../features/broadcast/components/HomeHero.tsx'
import { RecommendedCard } from '../features/broadcast/components/HomeSections.tsx'
import RecentReplays from '../features/replay/components/RecentReplays.tsx'
import { useHome } from '../features/broadcast/hooks/useHome.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

// CLUTCH FEED(T30)는 "최근 다시보기" 자리에 붙는다.
export default function HomePage() {
  const { category, setCategory, grid, hero, recommended } = useHome()
  const empty = (
    <EmptyState
      title='지금은 조용하네요'
      description={category ? '이 카테고리에는 라이브 중인 방송이 없어요.' : '첫 번째 클러치를 만들어 보세요.'}
      action={<Button to='/broadcasts/new'>방송하기</Button>}
    />
  )

  return (
    <div className='flex flex-col gap-8 lg:gap-10'>
      <h1 className='sr-only'>FLUTCH 홈</h1>

      {/* LIVE가 없으면(empty) 히어로 없이 아래 그리드의 빈 화면(졸린 마스코트 + 방송하기)이 대신한다 */}
      {hero && <HomeHero view={hero} />}

      {recommended.length > 0 && (
        <section aria-labelledby='home-recommended' className='flex flex-col gap-3 lg:gap-4'>
          <SectionTitle id='home-recommended'>추천 라이브 채널</SectionTitle>
          <ul className='hidden gap-4 sm:grid sm:grid-cols-[repeat(auto-fit,minmax(260px,1fr))]'>
            {recommended.map((b) => (
              <li key={b.id}>
                <RecommendedCard broadcast={b} />
              </li>
            ))}
          </ul>
          <ul className='-mx-4 flex gap-3 overflow-x-auto px-4 sm:hidden'>
            {recommended.map((b) => (
              <li key={b.id} className='w-[220px] shrink-0'>
                <BroadcastCard broadcast={b} />
              </li>
            ))}
          </ul>
        </section>
      )}

      <section aria-labelledby='home-live' className='flex flex-col gap-3 lg:gap-4'>
        <div className='flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between'>
          <SectionTitle id='home-live'>지금 두근거리는 중!</SectionTitle>
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

      <RecentReplays />
    </div>
  )
}
