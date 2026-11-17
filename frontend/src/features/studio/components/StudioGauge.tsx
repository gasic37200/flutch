import Badge from '../../../components/ui/Badge.tsx'
import ClutchGauge from '../../../components/ui/ClutchGauge.tsx'
import type { useStudio } from '../hooks/useStudio.ts'

// 스트리머가 송출 중 자기 방송의 클러치 게이지를 본다(읽기 전용). SUPER CLUTCH 발동 순간엔 배지만 3초 띄운다(전체 화면 연출은 시청자용).
export default function StudioGauge({ s, className = '' }: { s: ReturnType<typeof useStudio>; className?: string }) {
  const g = s.data?.gauge
  if (!g) return null
  return (
    <div className={`flex items-center gap-2 ${className}`} title={s.clutch.cooldownText}>
      {s.clutch.superClutch && <Badge tone='superClutch' />}
      <ClutchGauge logo={false} percent={g.percent} cooldown={g.phase === 'cooldown'} size='lg' className='min-w-0 grow gap-2' />
    </div>
  )
}
