import type { ReactNode } from 'react'
import type { Donation } from '../../../api/generated/Donation.ts'
import Badge from '../../../components/ui/Badge.tsx'
import { beats, BIG_AMOUNT, donorName } from '../format.ts'
import { kstStamp } from '../../wallet/format.ts'

/** 후원 한 줄: 표시 이름(소유자 응답이면 실명 + 익명 배지) · 비트 · 메시지 · 시각. `action`은 오른쪽 버튼 자리. */
export default function DonationRow({ donation: d, action }: { donation: Donation; action?: ReactNode }) {
  return (
    <li className='flex items-start gap-3 border-b border-surface-2 px-4 py-2.5 text-sm last:border-b-0'>
      <div className='flex min-w-0 grow flex-wrap items-baseline gap-x-3 gap-y-0.5'>
      <b className='flex min-w-0 max-w-40 items-center gap-1.5 break-all text-text-muted sm:w-36 sm:shrink-0'>
        {donorName(d)}
        {d.anonymous && d.donor && <Badge tone='anonymous' />}
      </b>
      <span className='shrink-0 font-mono font-bold'>
        <span className={d.amount >= BIG_AMOUNT ? 'text-burst' : 'text-brand'}>{beats(d.amount)}</span>
        <span className='sr-only'>비트</span>
      </span>
      {d.kind === 'burst' && <span className='shrink-0 font-display text-xs font-bold text-burst italic'>SUPER BURST</span>}
      <span className='min-w-0 grow break-words text-text-muted'>{d.message}</span>
      <time dateTime={d.createdAt} className='shrink-0 text-xs text-text-dim'>
        {kstStamp(d.createdAt)}
      </time>
      </div>
      {action && <span className='shrink-0'>{action}</span>}
    </li>
  )
}
