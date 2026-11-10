// 게이지 3단계: 충전 중(0~79) 라임 → 임박(80~99) 라임→주황 + 맥박 → MAX(100) 클러치 그라데이션(DESIGN §2-1)
function stageOf(p: number) {
  if (p >= 100) return { logo: 'logo-super-clutch', fill: 'bg-clutch', beat: '' }
  if (p >= 80) return { logo: 'logo-clutch-time-hot', fill: 'bg-[linear-gradient(90deg,var(--color-brand)_60%,var(--color-imminent))]', beat: 'animate-beat motion-reduce:animate-none' }
  return { logo: 'logo-clutch-time', fill: 'bg-brand', beat: '' }
}

export default function ClutchGauge({ percent, className = '' }: { percent: number; className?: string }) {
  const p = Math.round(Math.min(100, Math.max(0, percent)))
  const stage = stageOf(p)
  return (
    <div className={`flex items-center gap-3 ${className}`}>
      <img src={`/brand/${stage.logo}.png`} alt='CLUTCH TIME' className='h-8 w-auto' />
      <div className={`grow ${stage.beat}`}>
        <div role='progressbar' aria-label='클러치 게이지' aria-valuemin={0} aria-valuemax={100} aria-valuenow={p} className='chamfer h-3 bg-surface-2 [--c:6px]'>
          <div className={`chamfer h-full transition-[width] duration-500 [--c:6px] ${stage.fill}`} style={{ width: `${p}%` }} />
        </div>
      </div>
      <span className='w-12 text-right font-mono text-sm font-bold'>{p}%</span>
    </div>
  )
}
