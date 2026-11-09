import type { ReactNode } from 'react'

const base = 'chamfer inline-flex p-px [--c:6px]'
const face = 'chamfer-inner inline-flex h-8 items-center px-3 text-sm'

/** 정적 칩(카테고리·태그 표시) */
export function Chip({ children, className = '' }: { children: ReactNode; className?: string }) {
  return (
    <span className={`${base} bg-brand-line ${className}`}>
      <span className={`${face} bg-surface-2 text-text-muted`}>{children}</span>
    </span>
  )
}

/** 선택 가능한 칩(카테고리 필터·금액 프리셋). 선택 = 라임 채움 */
export function ChoiceChip({ selected, onClick, children }: { selected?: boolean; onClick: () => void; children: ReactNode }) {
  return (
    <span className='inline-flex has-[:focus-visible]:drop-shadow-focus'>
      <button type='button' aria-pressed={selected} onClick={onClick} className={`${base} ${selected ? 'bg-brand' : 'bg-brand-line hover:bg-brand'}`}>
        <span className={`${face} font-semibold ${selected ? 'bg-brand text-bg' : 'bg-surface-2 text-text'}`}>{children}</span>
      </button>
    </span>
  )
}
