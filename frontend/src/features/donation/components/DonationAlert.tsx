import type { ReactNode } from 'react'
import type { OverlayLayout } from '../../../api/generated/OverlayLayout.ts'
import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import { faceSrc, profileTone } from '../../../components/ui/mascot.ts'
import { alertStyle, DEFAULT_LAYOUT } from '../../live/overlayLayout.ts'
import { beats, BIG_AMOUNT, donorName } from '../format.ts'
import type { DonationAlerts } from '../hooks/useDonationAlerts.ts'

// 시안 AlertScale: 크기는 영상 폭(@container의 cqw) 비례. 일반 42%(200~520px), 10,000 Beat 이상 50%(240~600px). 글자·여백·아이콘도 비례.
export const ALERT_WIDTH = 'w-[clamp(200px,42cqw,520px)]'
const BIG_WIDTH = 'w-[clamp(240px,50cqw,600px)]'
const msg = 'truncate text-[length:clamp(11px,1.3cqw,14px)] text-text-muted'

/** 일반 후원 알림 카드(표현만). 배치 편집의 미리보기도 같은 카드를 쓴다. */
export function AlertCard({ title, message, side }: { title: ReactNode; message?: ReactNode; side?: ReactNode }) {
  return (
    <div className='chamfer w-full bg-brand p-px [--c:12px]'>
      <div className='chamfer-inner flex items-center gap-[clamp(8px,1.2cqw,14px)] bg-surface/96 px-[clamp(10px,1.8cqw,20px)] py-[clamp(8px,1.4cqw,16px)] [--c:12px]'>
        <Icon name='pulse' className='size-[clamp(18px,3cqw,32px)] text-brand motion-safe:animate-beat' />
        <div className='flex min-w-0 grow flex-col gap-0.5'>
          <span className='truncate text-[length:clamp(12px,1.5cqw,16px)] font-bold'>{title}</span>
          {message && <span className={msg}>{message}</span>}
        </div>
        {side}
      </div>
    </div>
  )
}

type Props = {
  alerts: DonationAlerts
  /** 스튜디오(소유자): 실명과 "넘기기" 버튼을 보여 준다 */
  owner?: boolean
  /** 스트리머 배치(없으면 기본: 위 가운데) */
  layout?: OverlayLayout
}

// 후원 알림: 영상 위(시청·스튜디오·전체화면 공통). 부모가 @container여야 크기가 영상 폭을 따른다. 컨테이너는 항상 있어 aria-live가 바뀐 내용을 읽는다.
export default function DonationAlert({ alerts, owner, layout = DEFAULT_LAYOUT }: Props) {
  const d = alerts.current
  const big = !!d && d.amount >= BIG_AMOUNT
  const name = d && (
    <>
      {donorName(d)}님{owner && d.anonymous && <Badge tone='anonymous' className='ml-1.5 align-middle' />}
    </>
  )
  const skip = owner && d && (
    <div className='pointer-events-auto flex shrink-0 flex-col items-end gap-1'>
      <Button size='sm' variant='neutral' onClick={() => void alerts.skip()} loading={alerts.skipping}>
        넘기기
      </Button>
      {alerts.waiting > 0 && <span className='text-[11px] text-text-dim'>대기 {alerts.waiting}건</span>}
    </div>
  )
  return (
    <div role='status' aria-live='polite' style={alertStyle(layout)} className={`pointer-events-none absolute z-20 -translate-x-1/2 ${big ? BIG_WIDTH : ALERT_WIDTH}`}>
      {d &&
        (big ? (
          <div className='filter-[drop-shadow(0_0_clamp(8px,1.4cqw,18px)_color-mix(in_srgb,var(--color-burst)_45%,transparent))]'>
            <div className='chamfer bg-burst p-0.5 [--c:12px]'>
              <div className='chamfer-inner flex items-center gap-[clamp(10px,1.4cqw,16px)] bg-surface/96 p-[clamp(10px,1.8cqw,22px)] [--c:12px]'>
                <img src={faceSrc(profileTone(d.donor?.id ?? 0).face)} alt='' className='size-[clamp(32px,6cqw,72px)] shrink-0 rounded-full' />
                <div className='flex min-w-0 grow flex-col gap-0.5'>
                  <span className='truncate text-[length:clamp(13px,1.8cqw,20px)] font-bold'>{name}</span>
                  <b className='font-mono text-[length:clamp(18px,3cqw,32px)] leading-[1.1] text-burst'>
                    {beats(d.amount)} <span className='font-display text-[length:clamp(12px,1.8cqw,20px)]'>Beat</span>
                  </b>
                  {d.message && <span className={msg}>{d.message}</span>}
                </div>
                {skip}
              </div>
            </div>
          </div>
        ) : (
          <AlertCard
            title={
              <>
                {name} <span className='font-mono text-brand'>{beats(d.amount)}</span> Beat 후원
              </>
            }
            message={d.message}
            side={skip}
          />
        ))}
    </div>
  )
}
