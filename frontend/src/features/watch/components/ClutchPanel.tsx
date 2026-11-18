import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import ClutchGauge from '../../../components/ui/ClutchGauge.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import type { useBurst } from '../hooks/useBurst.ts'
import type { useClutch } from '../hooks/useClutch.ts'
import BurstButton from './BurstButton.tsx'
import ClutchButton from './ClutchButton.tsx'

// 게이지 + 클러치·SUPER BURST 버튼. 안내·오류 문구는 줄 아래 한 곳에만 둬서 버튼·게이지 정렬이 문구 길이에 흔들리지 않는다.
export default function ClutchPanel({ broadcast, clutch, burst }: { broadcast: BroadcastDetail; clutch: ReturnType<typeof useClutch>; burst: ReturnType<typeof useBurst> }) {
  const paidHint = clutch.mode === 'open' && clutch.next === 'paid' ? '무료 5회를 다 썼어요. 이제 1 Beat씩 차감돼요.' : undefined
  const note = clutch.message ?? clutch.cooldownText ?? paidHint
  return (
    <Panel innerClassName='flex flex-col gap-2 px-3 py-2.5 md:px-4 md:py-3'>
      <div className='flex flex-col gap-2.5 md:flex-row md:items-center md:gap-3'>
        <div className='min-w-0 grow'>
          <ClutchGauge percent={broadcast.gauge.percent} cooldown={broadcast.gauge.phase === 'cooldown'} boost={burst.boost} size='lg' />
        </div>
        <div className='grid grid-cols-2 gap-2 empty:hidden md:flex md:gap-3'>
          <ClutchButton clutch={clutch} />
          <BurstButton burst={burst} mode={clutch.mode} />
        </div>
      </div>
      <p role='status' className={`text-xs empty:hidden ${clutch.message ? 'text-live' : 'text-text-muted'}`}>
        {note}
      </p>
    </Panel>
  )
}
