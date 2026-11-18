import Button from '../components/ui/Button.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Icon from '../components/ui/Icon.tsx'
import BroadcastGrid, { CategoryChips, GridSkeleton, SectionTitle } from '../features/broadcast/components/BroadcastGrid.tsx'
import { CATEGORY_LABEL } from '../features/broadcast/categories.ts'
import ReplayGrid from '../features/replay/components/ReplayGrid.tsx'
import { MAX_QUERY, useExplore } from '../features/broadcast/hooks/useExplore.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

// 카테고리 카드(라이브 수)·스트리머 섹션은 집계 API가 생기면 붙인다. 다시보기 결과는 LIVE 결과 뒤에 이어 붙인다.
export default function ExplorePage() {
  const x = useExplore()
  const { results } = x
  const heading = x.q ? `"${x.q}" 라이브` : x.category ? `${CATEGORY_LABEL[x.category]} 라이브` : '라이브 방송'
  return (
    <div className='flex flex-col gap-6 lg:gap-8'>
      <div className='flex flex-col gap-3.5'>
        <h1 className='text-2xl font-black sm:text-[26px]'>탐색</h1>
        <form onSubmit={x.search} role='search' className='max-w-[640px] focus-within:drop-shadow-field'>
          <label className='chamfer block bg-brand p-px [--c:6px]'>
            <span className='chamfer-inner flex h-12 items-center gap-2.5 bg-surface px-4'>
              <Icon name='search' className='size-5 shrink-0 text-brand' />
              <span className='sr-only'>방송 검색</span>
              <input
                type='search'
                value={x.draft}
                onChange={(e) => x.setDraft(e.target.value)}
                maxLength={MAX_QUERY}
                placeholder='제목, 스트리머, 태그'
                className='min-w-0 grow bg-transparent text-[15px] text-text outline-none placeholder:text-text-dim [&::-webkit-search-cancel-button]:hidden'
              />
              {x.draft && (
                <button type='button' onClick={x.clear} aria-label='검색어 지우기' className='grid size-9 shrink-0 place-items-center text-text-muted hover:text-text'>
                  <Icon name='close' className='size-4' />
                </button>
              )}
            </span>
          </label>
        </form>
        <p className='text-[13px] text-text-dim'>제목·스트리머 닉네임·태그에서 찾아요</p>
      </div>
      <CategoryChips value={x.category} onChange={x.setCategory} />

      <section className='flex flex-col gap-3.5' aria-labelledby='explore-results'>
        <div className='flex items-baseline gap-2.5'>
          <SectionTitle id='explore-results'>{heading}</SectionTitle>
          {!results.isPending && !results.isError && (
            <span className='font-mono text-[13px] text-text-muted'>
              {x.items.length}
              {results.hasNextPage ? '+' : ''}
            </span>
          )}
        </div>
        {results.isError ? (
          <ErrorScreen {...errorKindOf(results.error)} onRetry={() => void results.refetch()} />
        ) : results.isPending ? (
          <GridSkeleton />
        ) : (
          <>
            <BroadcastGrid
              items={x.items}
              empty={
                <EmptyState
                  emote={x.q ? 'tilt' : undefined}
                  title={x.q ? '검색 결과가 없어요' : '지금은 조용하네요'}
                  description={x.q ? `"${x.q}"에 맞는 방송을 찾지 못했어요. 다른 검색어나 카테고리로 찾아보세요.` : '라이브 중인 방송이 없어요.'}
                />
              }
            />
            {results.hasNextPage && (
              <div className='flex justify-center'>
                <Button variant='secondary' onClick={() => void results.fetchNextPage()} loading={results.isFetchingNextPage}>
                  더 보기
                </Button>
              </div>
            )}
          </>
        )}
      </section>

      {x.replays.isPending || (x.replays.data?.pages[0]?.items.length ?? 0) > 0 || x.replays.isError ? (
        <section className='flex flex-col gap-3.5' aria-labelledby='explore-replays'>
          <SectionTitle id='explore-replays'>{x.q ? `"${x.q}" 다시보기` : '다시보기'}</SectionTitle>
          <ReplayGrid query={x.replays} empty={null} />
        </section>
      ) : null}
    </div>
  )
}
