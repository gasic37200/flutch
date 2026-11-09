import type { ReactNode } from 'react'

// live 색은 LIVE 배지에만, burst는 후원자 배지에만(DESIGN §2). 색만으로 구분하지 않도록 항상 글자가 있다.
const tones = {
  live: { cls: 'bg-live text-text', label: 'LIVE' },
  ready: { cls: 'bg-surface-2 text-text-muted', label: 'READY' },
  ended: { cls: 'bg-border text-text-muted', label: 'ENDED' },
  superClutch: { cls: 'bg-clutch text-bg', label: 'SUPER CLUTCH' },
  streamer: { cls: 'bg-brand text-bg', label: '스트리머' },
  donor: { cls: 'bg-burst text-bg', label: '후원자' },
  anonymous: { cls: 'bg-surface-2 text-text-muted', label: '익명' },
  edited: { cls: 'text-text-dim', label: '(수정됨)' },
  recording: { cls: 'bg-surface-2 text-text', label: '녹화 중' },
}

type Props = { tone: keyof typeof tones; children?: ReactNode; className?: string }

export default function Badge({ tone, children, className = '' }: Props) {
  const t = tones[tone]
  return (
    <span className={`inline-flex h-5 items-center gap-1 rounded-xs px-1.5 font-display text-[11px] leading-none font-bold tracking-wide whitespace-nowrap ${t.cls} ${className}`}>
      {tone === 'live' && <span className='size-1.5 rounded-full bg-text' aria-hidden />}
      {children ?? t.label}
    </span>
  )
}
