import type { ReactNode } from 'react'

// live 색은 LIVE 배지에만, burst는 후원자 배지에만(DESIGN §2). 색만으로 구분하지 않도록 항상 글자가 있다.
// 상태 배지(READY·ENDED)는 6px 사선, LIVE·SUPER CLUTCH는 깎지 않는다. 채팅 배지(스트리머·후원자·익명)는 작은 한글 라벨.
const state = 'chamfer [--c:6px] px-2 py-[3px] font-display text-xs font-bold'
const tones = {
  live: { cls: 'bg-live px-2 py-[3px] font-display text-xs font-bold tracking-wider text-text', label: 'LIVE' },
  ready: { cls: `${state} bg-brand/15 text-brand`, label: 'READY' },
  ended: { cls: `${state} bg-surface-2 text-text-muted`, label: 'ENDED' },
  superClutch: { cls: 'bg-clutch px-2 py-[3px] font-display text-xs font-bold text-bg italic', label: 'SUPER CLUTCH' },
  streamer: { cls: 'bg-brand px-1.5 py-0.5 text-[11px] font-bold text-bg', label: '스트리머' },
  donor: { cls: 'bg-burst px-1.5 py-0.5 text-[11px] font-bold text-bg', label: '후원자' },
  anonymous: { cls: 'bg-surface-2 px-1.5 py-0.5 text-[11px] text-text-muted', label: '익명' },
  edited: { cls: 'text-[11px] text-text-dim', label: '(수정됨)' },
  recording: { cls: 'bg-surface-2 px-2 py-[3px] text-xs text-text-muted', label: '녹화 중' },
}

type Props = { tone: keyof typeof tones; children?: ReactNode; className?: string }

export default function Badge({ tone, children, className = '' }: Props) {
  const t = tones[tone]
  return (
    <span className={`inline-flex items-center gap-1.5 leading-none whitespace-nowrap ${t.cls} ${className}`}>
      {tone === 'live' && <span className='size-1.5 rounded-full bg-text' aria-hidden />}
      {children ?? t.label}
    </span>
  )
}
