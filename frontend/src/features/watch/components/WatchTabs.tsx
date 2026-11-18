import type { ReactNode } from 'react'

type Tab = { id: string; label: ReactNode }

// 시청 모바일(600px 이하) 전용 채팅·후원 탭: 시안처럼 두 탭이 폭을 반씩 차지한다. 공용 Tabs는 왼쪽 정렬이라 따로 둔다.
export default function WatchTabs({ tabs, value, onChange }: { tabs: Tab[]; value: string; onChange: (id: string) => void }) {
  return (
    <div role='tablist' aria-label='채팅·후원' className='flex border-b border-surface-2'>
      {tabs.map((t, i) => {
        const selected = t.id === value
        return (
          <button
            key={t.id}
            type='button'
            role='tab'
            aria-selected={selected}
            tabIndex={selected ? 0 : -1}
            onClick={() => onChange(t.id)}
            onKeyDown={(e) => {
              const step = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0
              if (step) onChange(tabs[(i + step + tabs.length) % tabs.length].id)
            }}
            className={`-mb-px h-11 flex-1 border-b-2 text-sm ${selected ? 'border-brand font-bold text-text' : 'border-transparent text-text-muted'}`}
          >
            {t.label}
          </button>
        )
      })}
    </div>
  )
}
