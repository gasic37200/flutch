import type { LedgerEntry } from '../../../api/generated/LedgerEntry.ts'
import Panel from '../../../components/ui/Panel.tsx'
import { kstStamp, signed } from '../format.ts'
import { describe, typeLabel } from '../hooks/useWallet.ts'

// 내역 목록. 충전(+)만 라임, 나머지(-)는 흰색. 유료 클러치는 서버가 방송별로 묶어 내려준다.
export default function LedgerList({ items }: { items: LedgerEntry[] }) {
  return (
    <Panel innerClassName='flex flex-col'>
      <ul>
        {items.map((e) => (
          <li key={e.id} className='flex items-center gap-3.5 border-b border-surface-2 px-4 py-3.5 last:border-b-0 sm:px-[18px]'>
            <span className='chamfer w-[84px] shrink-0 bg-surface-2 py-1 text-center text-xs max-sm:w-16 max-sm:text-[11px] text-text-muted [--c:6px]'>{typeLabel(e.type)}</span>
            <div className='flex min-w-0 grow flex-col gap-0.5'>
              <b className='truncate text-sm'>{describe(e)}</b>
              <time dateTime={e.createdAt} className='text-xs text-text-dim'>
                {kstStamp(e.createdAt)}
              </time>
            </div>
            <div className='flex shrink-0 flex-col items-end gap-0.5 font-mono'>
              <b className={`text-[15px] ${e.amount > 0 ? 'text-brand' : 'text-text'}`}>
                {signed(e.amount)}
                <span className='sr-only'>비트</span>
              </b>
              <span className='text-xs text-text-dim'>잔액 {e.balanceAfter.toLocaleString('ko-KR')}</span>
            </div>
          </li>
        ))}
      </ul>
    </Panel>
  )
}
