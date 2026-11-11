import type { ReactNode } from 'react'

/** 정적 칩(카테고리·태그 표시) */
export function Chip({ children, className = '' }: { children: ReactNode; className?: string }) {
  return <span className={`chamfer inline-flex items-center bg-surface-2 px-2 py-0.5 text-xs text-text-muted [--c:6px] ${className}`}>{children}</span>
}

/** 선택 가능한 칩(카테고리 필터·금액 프리셋). 선택 = 라임 채움, 기본 = surface-2 */
export function ChoiceChip({ selected, onClick, children }: { selected?: boolean; onClick: () => void; children: ReactNode }) {
  return (
    <span className='inline-flex shrink-0 has-[:focus-visible]:drop-shadow-focus'>
      <button
        type='button'
        aria-pressed={selected}
        onClick={onClick}
        className={`chamfer inline-flex h-11 items-center px-4 text-sm sm:h-9 [--c:6px] ${selected ? 'bg-brand font-bold text-bg' : 'bg-surface-2 text-text-muted hover:text-text'}`}
      >
        {children}
      </button>
    </span>
  )
}
