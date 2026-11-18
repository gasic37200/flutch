import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import ClutchGauge from '../../../components/ui/ClutchGauge.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { useMediaQuery } from '../../../hooks/useMediaQuery.ts'
import type { useBurst } from '../hooks/useBurst.ts'
import type { useClutch } from '../hooks/useClutch.ts'
import BurstButton from './BurstButton.tsx'
import ClutchButton from './ClutchButton.tsx'

// 게이지 + 클러치·SUPER BURST 버튼. 안내·오류 문구는 줄 아래 한 곳에만 둬서 버튼·게이지 정렬이 문구 길이에 흔들리지 않는다.
// 600px 이하(시안 모바일)는 테두리 없는 한 줄 띠: 바+%+아이콘 버튼 2개, 상시 안내줄은 숨기고 오류·쿨다운만 남긴다.
export default function ClutchPanel({ broadcast, clutch, burst }: { broadcast: BroadcastDetail; clutch: ReturnType<typeof useClutch>; burst: ReturnType<typeof useBurst> }) {
  const compact = useMediaQuery('(max-width: 600px)')
  const guideHint = clutch.mode === 'open' && !compact ? (clutch.next === 'paid' ? '무료 5회를 다 썼어요. 이제 1 Beat씩 차감돼요.' : `무료 클러치 ${clutch.freeLeft}회 남음 · 30초마다 5회 무료, 이후 1회 1 Beat`) : undefined
  const note = clutch.message ?? clutch.cooldownText ?? guideHint
  const body = (
    <>
      <div className='flex items-center gap-2 md:gap-3.5'>
        <div className='min-w-0 grow'>
          <ClutchGauge percent={broadcast.gauge.percent} cooldown={broadcast.gauge.phase === 'cooldown'} boost={burst.boost} size='lg' />
        </div>
        <div className='flex gap-2 empty:hidden md:gap-3.5'>
          <ClutchButton clutch={clutch} compact={compact} />
          <BurstButton burst={burst} mode={clutch.mode} compact={compact} />
        </div>
      </div>
      <p role='status' className={`text-xs empty:hidden ${clutch.message ? 'text-live' : 'text-text-dim'}`}>
        {note}
      </p>
    </>
  )
  if (compact) return <div className='flex flex-col gap-2 border-b border-brand-line px-3 py-2.5'>{body}</div>
  return <Panel innerClassName='flex flex-col gap-2.5 px-4 py-3'>{body}</Panel>
}
