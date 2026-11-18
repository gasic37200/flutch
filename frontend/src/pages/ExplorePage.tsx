import Button from '../components/ui/Button.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Icon from '../components/ui/Icon.tsx'
import BroadcastGrid, { GridSkeleton } from '../features/broadcast/components/BroadcastGrid.tsx'
import { CATEGORY_LABEL } from '../features/broadcast/categories.ts'
import { CategoryCards, StreamerRow, VodRow } from '../features/broadcast/components/ExploreSections.tsx'
import { useLiveBroadcasts } from '../features/broadcast/hooks/useBroadcasts.ts'
import { MAX_QUERY, useExplore } from '../features/broadcast/hooks/useExplore.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

// 다시보기 결과는 LIVE 결과 뒤에 이어 붙인다.
export default function ExplorePage() {
  const x = useExplore()
  const { results } = x
  // 홈 그리드와 같은 params라 캐시를 공유한다. 카테고리 카드의 라이브 수에 쓴다.
  const all = useLiveBroadcasts({ limit: 50 })
  const vods = x.replays.data?.pages.flatMap((p) => p.items) ?? []
  const heading = x.q ? `"${x.q}" 라이브` : x.category ? `${CATEGORY_LABEL[x.category]} 라이브` : '라이브 방송'
  return (
    <div className='flex flex-col gap-7 min-[601px]:gap-9'>
      <div className='flex flex-col gap-3.5'>
        <h1 className='text-[22px] font-black min-[601px]:text-[26px]'>탐색</h1>
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
      <section aria-labelledby='explore-cats' className='flex flex-col gap-3.5'>
        <h2 id='explore-cats' className='text-lg font-black'>
          카테고리
        </h2>
        <CategoryCards value={x.category} onChange={x.setCategory} live={all.data?.items ?? []} />
      </section>

      <section className='flex flex-col gap-3.5' aria-labelledby='explore-results'>
        <div className='flex items-baseline gap-2.5'>
          <h2 id='explore-results' className='text-lg font-black'>
            {heading}
          </h2>
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

      <div className='grid gap-8 min-[901px]:grid-cols-2'>
        {x.items.length > 0 && (
          <section className='flex min-w-0 flex-col gap-3.5' aria-labelledby='explore-streamers'>
            <h2 id='explore-streamers' className='text-lg font-black'>
              스트리머
            </h2>
            <StreamerRow items={x.items} />
          </section>
        )}
        {vods.length > 0 && (
          <section className='flex min-w-0 flex-col gap-3.5' aria-labelledby='explore-replays'>
            <h2 id='explore-replays' className='text-lg font-black'>
              {x.q ? `"${x.q}" 다시보기` : '다시보기'}
            </h2>
            {vods.map((r) => (
              <VodRow key={r.id} recording={r} />
            ))}
            {x.replays.hasNextPage && (
              <div className='flex justify-center'>
                <Button variant='secondary' onClick={() => void x.replays.fetchNextPage()} loading={x.replays.isFetchingNextPage}>
                  더 보기
                </Button>
              </div>
            )}
          </section>
        )}
      </div>
    </div>
  )
}
