import Button from '../../../components/ui/Button.tsx'
import Switch from '../../../components/ui/Switch.tsx'
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
    <Panel className='shrink-0' innerClassName='flex flex-col gap-2 px-3.5 py-3'>
      <div className='flex items-center gap-2'>
        <h2 className='text-sm font-bold'>후원 알림 대기열</h2>
        <span className='grow text-xs text-text-dim'>10초씩</span>
        <span className='text-xs text-text-muted' aria-hidden>
          TTS 읽기
        </span>
        <Switch label='후원 TTS 읽기' checked={alerts.tts.on} onChange={alerts.tts.toggle} />
      </div>
      {d ? (
        <div className='chamfer flex items-center gap-2.5 bg-surface-2 px-2.5 py-2 [--c:6px]'>
          <p className='min-w-0 grow text-[13px] break-all'>
            <b>
              {name(d)} · <span className='font-mono text-brand'>{beats(d.amount)}</span>
            </b>{' '}
            <span className='text-text-muted'>표시 중</span>
          </p>
          <Button size='sm' variant='neutral' onClick={() => void alerts.skip()} loading={alerts.skipping}>
            스킵
          </Button>
        </div>
      ) : (
        <p className='bg-surface-2 px-2.5 py-2 text-[13px] text-text-muted'>대기 중인 후원 알림이 없어요.</p>
      )}
      {next && (
        <div className='flex justify-between gap-2 px-0.5 text-xs text-text-muted'>
          <span className='min-w-0 truncate'>
            다음 · {name(next)}
            {alerts.waiting > 1 && ` 외 ${alerts.waiting - 1}건`}
          </span>
          <b className='font-mono text-burst'>{beats(next.amount)}</b>
        </div>
      )}
    </Panel>
  )
}
