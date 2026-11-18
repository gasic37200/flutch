import Button from '../components/ui/Button.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import BroadcastGrid, { CategoryChips, GridSkeleton, SectionTitle } from '../features/broadcast/components/BroadcastGrid.tsx'
import HomeIntro from '../features/broadcast/components/HomeIntro.tsx'
import HomeHero from '../features/broadcast/components/HomeHero.tsx'
import { RecommendedCard } from '../features/broadcast/components/HomeSections.tsx'
import ClutchFeed from '../features/replay/components/ClutchFeed.tsx'
import { useHome } from '../features/broadcast/hooks/useHome.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

// CLUTCH FEED: 클립이 있으면 클립, 없으면 최근 다시보기.
export default function HomePage() {
  const { category, setCategory, grid, hero, recommended } = useHome()
  // 라이브가 전혀 없으면(카테고리 필터 밖) 히어로·그리드 대신 서비스 소개를 보여 준다
  const noLive = !category && !hero?.hero.broadcast && grid.data?.items.length === 0
  const empty = (
    <EmptyState
      title='지금은 조용하네요'
      description={category ? '이 카테고리에는 라이브 중인 방송이 없어요.' : '첫 번째 클러치를 만들어 보세요.'}
      action={<Button to='/broadcasts/new'>방송하기</Button>}
    />
  )

  return (
    <div className='flex flex-col gap-7 lg:gap-10'>
      <h1 className='sr-only'>FLUTCH 홈</h1>

      {noLive ? <HomeIntro /> : hero && <HomeHero view={hero} />}

      {recommended.length > 0 && (
        <section aria-labelledby='home-recommended' className='flex flex-col gap-4'>
          <SectionTitle id='home-recommended'>추천 라이브 채널</SectionTitle>
          <ul className='grid grid-cols-[repeat(auto-fit,minmax(300px,1fr))] gap-4'>
            {recommended.map((b) => (
              <li key={b.id}>
                <RecommendedCard broadcast={b} />
              </li>
            ))}
          </ul>
        </section>
      )}

      {!noLive && (
        <section aria-labelledby='home-live' className='flex flex-col gap-4'>
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
      )}

      <ClutchFeed />
    </div>
  )
}
