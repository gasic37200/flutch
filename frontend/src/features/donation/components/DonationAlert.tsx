import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import { beats, BIG_AMOUNT, donorName } from '../format.ts'
import type { DonationAlerts } from '../hooks/useDonationAlerts.ts'

type Props = {
  alerts: DonationAlerts
  /** 스튜디오(소유자): 실명과 "넘기기" 버튼을 보여 준다 */
  owner?: boolean
  className?: string
}

// 후원 알림: 영상 위(시청·스튜디오 공통). 컨테이너는 항상 있어 aria-live가 바뀐 내용을 읽는다. 10,000비트 이상은 노랑 큰 카드.
export default function DonationAlert({ alerts, owner, className = 'top-3' }: Props) {
  const d = alerts.current
  const big = !!d && d.amount >= BIG_AMOUNT
  return (
    <div role='status' aria-live='polite' className={`pointer-events-none absolute inset-x-0 z-20 flex justify-center px-3 ${className}`}>
      {d && (
        <div className={`chamfer w-full [--c:12px] ${big ? 'max-w-lg bg-burst p-0.5' : 'max-w-md bg-brand p-px'}`}>
          <div className={`chamfer-inner flex items-center gap-3.5 bg-surface/95 ${big ? 'p-5' : 'px-4 py-3'}`}>
            <Icon name='pulse' className={`shrink-0 motion-safe:animate-beat ${big ? 'size-12 text-burst' : 'size-8 text-brand'}`} />
            <div className='flex min-w-0 grow flex-col gap-0.5'>
              <b className={`flex flex-wrap items-center gap-1.5 break-all ${big ? 'text-lg' : 'text-[15px]'}`}>
                {donorName(d)}님
                {owner && d.anonymous && <Badge tone='anonymous' />}
              </b>
              <b className={`font-mono ${big ? 'text-[28px] text-burst' : 'text-[15px] text-brand'}`}>
                {beats(d.amount)} <span className='font-display text-[0.7em]'>Beat 후원</span>
              </b>
              {d.message && <span className='text-sm break-words text-text-muted'>{d.message}</span>}
            </div>
            {owner && (
              <div className='pointer-events-auto flex shrink-0 flex-col items-end gap-1'>
                <Button size='sm' variant='neutral' onClick={() => void alerts.skip()} loading={alerts.skipping}>
                  넘기기
                </Button>
                {alerts.waiting > 0 && <span className='text-[11px] text-text-dim'>대기 {alerts.waiting}건</span>}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
