import { useLevel } from '../hooks/useLevel.ts'

// 정상은 라임, 높으면 흰색(DESIGN §2).
export default function LevelMeter({ analyser, label }: { analyser: AnalyserNode | null; label: string }) {
  const level = useLevel(analyser)
  const pct = Math.round(level * 100)
  return (
    <div role='meter' aria-label={`${label} 레벨`} aria-valuemin={0} aria-valuemax={100} aria-valuenow={pct} className='h-2 w-full overflow-hidden rounded-xs bg-border'>
      <div className={`h-full transition-[width] duration-75 motion-reduce:transition-none ${level > 0.85 ? 'bg-text' : 'bg-brand'}`} style={{ width: `${pct}%` }} />
    </div>
  )
}
