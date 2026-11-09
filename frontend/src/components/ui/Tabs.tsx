type Tab = { id: string; label: string }

type Props = { label: string; tabs: Tab[]; value: string; onChange: (id: string) => void; className?: string }

// 선택된 탭 = 라임 밑줄. ←/→로 이동. 탭 내용은 호출하는 쪽이 value에 맞춰 그린다.
export default function Tabs({ label, tabs, value, onChange, className = '' }: Props) {
  const move = (i: number, step: number) => onChange(tabs[(i + step + tabs.length) % tabs.length].id)
  return (
    <div role='tablist' aria-label={label} className={`flex gap-1 border-b border-border ${className}`}>
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
            className={`-mb-px border-b-2 px-4 py-2 text-sm font-semibold ${selected ? 'border-brand text-text' : 'border-transparent text-text-muted hover:text-text'}`}
          >
            {t.label}
          </button>
        )
      })}
    </div>
  )
}
