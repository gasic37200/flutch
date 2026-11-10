import type { ReactNode } from 'react'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { Category } from '../../../api/generated/Category.ts'
import { ChoiceChip } from '../../../components/ui/Chip.tsx'
import LiveCard from '../../../components/ui/LiveCard.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import { CATEGORIES, CATEGORY_LABEL } from '../categories.ts'

const gridClass = 'grid grid-cols-2 gap-3 sm:grid-cols-[repeat(auto-fill,minmax(220px,1fr))] sm:gap-4'

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
        <Skeleton key={i} card className='aspect-[4/3] w-full' />
      ))}
    </div>
  )
}

/** 섹션 제목: 라임 점 + 굵은 제목(시안 공통). */
export function SectionTitle({ id, children }: { id?: string; children: ReactNode }) {
  return (
    <div className='flex items-center gap-2.5'>
      <span className='size-2.5 rounded-full bg-brand' aria-hidden />
      <h2 id={id} className='text-[17px] font-black sm:text-xl'>
        {children}
      </h2>
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
    <div role='group' aria-label='카테고리' className='-mx-4 flex gap-2 overflow-x-auto px-4 sm:mx-0 sm:flex-wrap sm:overflow-visible sm:px-0'>
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
