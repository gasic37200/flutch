import Button from '../../../components/ui/Button.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import type { Donation } from '../../../api/generated/Donation.ts'
import { beats } from '../../donation/format.ts'
import type { DonationAlerts } from '../../donation/hooks/useDonationAlerts.ts'

// 소유자에게는 익명이어도 실명이 내려온다(익명의 후원자 표시는 donor가 있을 때 닉네임을 덧붙인다)
const name = (d: Donation) => (d.anonymous ? `익명의 후원자${d.donor ? ` (실명: ${d.donor.nickname})` : ''}` : (d.donor?.nickname ?? '후원자'))

// 스튜디오 오른쪽 "후원 알림 대기열": 지금 화면에 뜬 후원과 다음 후원. 스킵은 영상 위 알림의 넘기기와 같은 동작.
// shortcut: 남은 초 표시는 없다(ALERT_MS 타이머가 훅 안에 있음), 필요해지면 훅이 시작 시각을 내보낸다.
export default function DonationQueueCard({ alerts }: { alerts: DonationAlerts }) {
  const { current: d, next } = alerts
  return (
    <Panel innerClassName='flex flex-col gap-2.5 p-4'>
      <div className='flex items-center'>
        <h2 className='text-[15px] font-bold'>후원 알림 대기열</h2>
        <span className='ml-auto text-xs text-text-dim'>10초씩 표시</span>
      </div>
      {d ? (
        <div className='chamfer flex items-center gap-2.5 bg-surface-2 px-3 py-2.5 [--c:6px]'>
          <div className='flex min-w-0 grow flex-col gap-0.5'>
            <b className='flex flex-wrap items-center gap-1.5 text-sm break-all'>
              {name(d)} ·<span className='font-mono text-brand'>{beats(d.amount)}</span>
            </b>
            <span className='text-xs text-text-muted'>표시 중{alerts.waiting > 0 ? ` · 대기 ${alerts.waiting}건` : ''}</span>
          </div>
          <Button size='sm' variant='neutral' onClick={() => void alerts.skip()} loading={alerts.skipping}>
            스킵
          </Button>
        </div>
      ) : (
        <p className='bg-surface-2 px-3 py-2.5 text-[13px] text-text-muted'>대기 중인 후원 알림이 없어요.</p>
      )}
      {next && (
        <div className='flex justify-between gap-2 px-1 text-[13px] text-text-muted'>
          <span className='min-w-0 truncate'>다음 · {name(next)}</span>
          <b className='font-mono text-burst'>{beats(next.amount)}</b>
        </div>
      )}
    </Panel>
  )
}
