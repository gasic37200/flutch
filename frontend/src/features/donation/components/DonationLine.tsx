import type { Donation } from '../../../api/generated/Donation.ts'
import Badge from '../../../components/ui/Badge.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import { beats, donorName } from '../format.ts'

// 채팅창 후원 강조 한 줄(저장되지 않는 실시간 줄). 색만이 아니라 아이콘·"비트 후원" 글자로 구분한다.
export default function DonationLine({ donation: d }: { donation: Donation }) {
  return (
    <li className='flex flex-col gap-0.5 bg-burst/10 px-2.5 py-2 text-sm'>
      <div className='flex flex-wrap items-center gap-x-1.5'>
        <Icon name='pulse' className='size-4 shrink-0 text-burst' />
        <b className='break-all'>{donorName(d)}</b>
        {d.anonymous && d.donor && <Badge tone='anonymous' />}
        <b className='font-mono text-burst'>{beats(d.amount)}</b>
        <span className='text-text-muted'>비트 후원</span>
      </div>
      {d.message && <span className='break-words text-text-muted'>{d.message}</span>}
    </li>
  )
}
