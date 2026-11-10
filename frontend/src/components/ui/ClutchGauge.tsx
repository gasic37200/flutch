// 게이지 3단계: 충전 중(0~79) 라임 → 임박(80~99) 라임→주황 + 맥박 → MAX(100) 클러치 그라데이션(DESIGN §2-1)
function stageOf(p: number, cooldown: boolean) {
  if (cooldown)
    return {
      logo: 'logo-clutch-time',
      fill: 'bg-text-dim',
      beat: 'opacity-60',
    }
  if (p >= 100) return { logo: 'logo-super-clutch', fill: 'bg-clutch', beat: '' }
  if (p >= 80)
    return {
      logo: 'logo-clutch-time-hot',
      fill: 'bg-[linear-gradient(90deg,var(--color-brand)_60%,var(--color-imminent))]',
      beat: 'animate-beat motion-reduce:animate-none',
    }
  return { logo: 'logo-clutch-time', fill: 'bg-brand', beat: '' }
}

// lg는 시청 화면용(모바일에서 한 단계 작게). boost는 SUPER BURST 연출(동작 줄임이면 색만 바뀐다).
export default function ClutchGauge({
  percent,
  cooldown = false,
  boost = false,
  size = 'md',
  logo = true,
  className = '',
}: {
  percent: number
  cooldown?: boolean
  boost?: boolean
  size?: 'md' | 'lg'
  /** false면 바만(로고를 따로 크게 두는 홈 히어로 패널) */
  logo?: boolean
  className?: string
}) {
  const p = Math.round(Math.min(100, Math.max(0, percent)))
  const stage = stageOf(p, cooldown)
  const lg = size === 'lg'
  return (
    <div className={`flex items-center gap-3 ${className}`}>
      {logo && <img src={`/brand/${stage.logo}.png`} alt='CLUTCH TIME' className={lg ? 'h-7 w-auto md:h-10' : 'h-8 w-auto'} />}
      <div className={`grow ${boost ? 'motion-safe:animate-burst-boost' : stage.beat}`}>
        <div
          role='progressbar'
          aria-label='클러치 게이지'
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={p}
          className={`chamfer bg-surface-2 [--c:6px] ${lg ? 'h-2.5 md:h-3.5' : 'h-3'}`}
        >
          <div className={`chamfer h-full transition-[width] duration-500 [--c:6px] ${boost ? 'bg-burst' : stage.fill}`} style={{ width: `${p}%` }} />
        </div>
      </div>
      <span className={`w-12 text-right font-mono font-bold ${lg ? 'text-sm md:text-base' : 'text-sm'}`}>{p}%</span>
    </div>
  )
}
