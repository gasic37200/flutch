import type { ReactNode } from 'react'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { Category } from '../../../api/generated/Category.ts'
import { ChoiceChip } from '../../../components/ui/Chip.tsx'
import LiveCard from '../../../components/ui/LiveCard.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import { CATEGORIES, CATEGORY_LABEL } from '../categories.ts'

const gridClass = 'grid gap-4 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4'

export function BroadcastCard({ broadcast: b }: { broadcast: Broadcast }) {
  return (
    <LiveCard
      to={`/live/${b.id}`}
      title={b.title}
      streamerId={b.owner.id}
      streamerName={b.owner.nickname}
      category={CATEGORY_LABEL[b.category]}
      thumbnailUrl={b.thumbnailUrl}
      live
      viewerCount={b.viewerCount}
    />
  )
}

export function GridSkeleton({ count = 8 }: { count?: number }) {
  return (
    <div className={gridClass} aria-busy aria-label='불러오는 중'>
      {Array.from({ length: count }, (_, i) => (
        <Skeleton key={i} className='aspect-[4/3] w-full' />
      ))}
    </div>
  )
}

/** 카드 그리드. 비었을 때 보여 줄 화면(`empty`)은 호출하는 쪽이 정한다. */
export default function BroadcastGrid({ items, empty, className = '' }: { items: Broadcast[]; empty: ReactNode; className?: string }) {
  if (items.length === 0) return <>{empty}</>
  return (
    <ul className={`${gridClass} ${className}`}>
      {items.map((b) => (
        <li key={b.id}>
          <BroadcastCard broadcast={b} />
        </li>
      ))}
    </ul>
  )
}

/** "전체" + 8개 카테고리 칩. 선택을 다시 누르면 전체로 돌아간다. */
export function CategoryChips({ value, onChange }: { value: Category | undefined; onChange: (c: Category | undefined) => void }) {
  return (
    <div role='group' aria-label='카테고리' className='flex flex-wrap gap-2'>
      <ChoiceChip selected={value === undefined} onClick={() => onChange(undefined)}>
        전체
      </ChoiceChip>
      {CATEGORIES.map((c) => (
        <ChoiceChip key={c} selected={value === c} onClick={() => onChange(value === c ? undefined : c)}>
          {CATEGORY_LABEL[c]}
        </ChoiceChip>
      ))}
    </div>
  )
}
