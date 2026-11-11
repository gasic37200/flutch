type Tab = { id: string; label: string }

type Props = { label: string; tabs: Tab[]; value: string; onChange: (id: string) => void; className?: string }

// 선택된 탭 = 라임 밑줄 + 굵은 글자. ←/→로 이동. 탭 내용은 호출하는 쪽이 value에 맞춰 그린다.
export default function Tabs({ label, tabs, value, onChange, className = '' }: Props) {
  const move = (i: number, step: number) => onChange(tabs[(i + step + tabs.length) % tabs.length].id)
  return (
    <div role='tablist' aria-label={label} className={`flex overflow-x-auto border-b border-surface-2 ${className}`}>
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
              if (e.key === 'ArrowRight') move(i, 1)
              if (e.key === 'ArrowLeft') move(i, -1)
            }}
            className={`-mb-px h-11 shrink-0 whitespace-nowrap border-b-2 px-3.5 text-sm ${selected ? 'border-brand font-bold text-text' : 'border-transparent text-text-muted hover:text-text'}`}
          >
            {t.label}
          </button>
        )
      })}
    </div>
  )
}
