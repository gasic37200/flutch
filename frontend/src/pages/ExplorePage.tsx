import Button from '../components/ui/Button.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Field from '../components/ui/Field.tsx'
import BroadcastGrid, { CategoryChips, GridSkeleton } from '../features/broadcast/components/BroadcastGrid.tsx'
import { MAX_QUERY, useExplore } from '../features/broadcast/hooks/useExplore.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

export default function ExplorePage() {
  const x = useExplore()
  const { results } = x
  return (
    <div className='space-y-4'>
      <h1 className='font-display text-2xl font-bold tracking-wide'>탐색</h1>
      <form onSubmit={x.search} role='search' className='flex max-w-xl items-end gap-2'>
        <Field
          label='방송 검색'
          type='search'
          value={x.draft}
          onChange={(e) => x.setDraft(e.target.value)}
          maxLength={MAX_QUERY}
          placeholder='제목, 스트리머, 태그'
          className='grow'
        />
        <Button type='submit'>검색</Button>
      </form>
      <CategoryChips value={x.category} onChange={x.setCategory} />

      {x.q && (
        <p className='text-sm text-text-muted'>
          &lsquo;{x.q}&rsquo; 검색 결과
        </p>
      )}

      {results.isError ? (
        <ErrorScreen {...errorKindOf(results.error)} onRetry={() => void results.refetch()} />
      ) : results.isPending ? (
        <GridSkeleton />
      ) : (
        <>
          <BroadcastGrid
            items={x.items}
            empty={<EmptyState title={x.q ? '검색 결과가 없어요' : '지금은 조용하네요'} description={x.q ? '다른 검색어나 카테고리로 찾아보세요.' : '라이브 중인 방송이 없어요.'} />}
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
    </div>
  )
}
