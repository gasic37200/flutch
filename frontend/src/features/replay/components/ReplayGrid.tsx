import type { ReactNode } from 'react'
import type { RecordingCard } from '../../../api/generated/RecordingCard.ts'
import Button from '../../../components/ui/Button.tsx'
import ErrorScreen from '../../../components/ui/ErrorScreen.tsx'
import LiveCard from '../../../components/ui/LiveCard.tsx'
import { errorKindOf } from '../../../hooks/errorKind.ts'
import { CATEGORY_LABEL } from '../../broadcast/categories.ts'
import { cardGrid, GridSkeleton } from '../../broadcast/components/BroadcastGrid.tsx'
import { count, durationLabel, kstDate } from '../format.ts'
import type { useRecordings } from '../hooks/useRecordings.ts'

export const gridClass = `${cardGrid}`

export function ReplayCard({ recording: r }: { recording: RecordingCard }) {
  return (
    <LiveCard
      to={`/replay/${r.id}`}
      title={r.title}
      streamerId={r.owner.id}
      streamerName={r.owner.nickname}
      category={CATEGORY_LABEL[r.category]}
      thumbnailUrl={r.thumbnailUrl}
      durationLabel={durationLabel(r.durationSec)}
      meta={`조회 ${count(r.views)} · 최고 ${count(r.peakViewers)} · ${kstDate(r.startedAt)}`}
    />
  )
}

export function ReplayList({ items }: { items: RecordingCard[] }) {
  return (
    <ul className={gridClass}>
      {items.map((r) => (
        <li key={r.id}>
          <ReplayCard recording={r} />
        </li>
      ))}
    </ul>
  )
}

/** 로딩·에러·빈 상태·"더 보기"까지 한 번에 그리는 다시보기 그리드. */
export default function ReplayGrid({ query, empty }: { query: ReturnType<typeof useRecordings>; empty: ReactNode }) {
  if (query.isError) return <ErrorScreen {...errorKindOf(query.error)} onRetry={() => void query.refetch()} />
  if (query.isPending) return <GridSkeleton />
  const items = query.data.pages.flatMap((p) => p.items)
  if (items.length === 0) return <>{empty}</>
  return (
    <>
      <ReplayList items={items} />
      {query.hasNextPage && (
        <div className='flex justify-center'>
          <Button variant='secondary' onClick={() => void query.fetchNextPage()} loading={query.isFetchingNextPage}>
            더 보기
          </Button>
        </div>
      )}
    </>
  )
}
