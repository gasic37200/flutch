import { useLevel } from '../hooks/useLevel.ts'

const SEGMENTS = 20
const HOT = 14 // 이 칸부터는 높은 음량: 흰색(DESIGN §2)

// 가로는 20칸 막대(정상은 라임, 높으면 흰색), 세로(모바일)는 하나의 채움 막대.
export default function LevelMeter({ analyser, label, vertical }: { analyser: AnalyserNode | null; label: string; vertical?: boolean }) {
  const level = useLevel(analyser)
  const pct = Math.round(level * 100)
  const meter = { role: 'meter', 'aria-label': `${label} 레벨`, 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': pct }
  if (vertical)
    return (
      <div {...meter} className='flex h-[120px] w-2.5 flex-col justify-end bg-surface-2'>
        <div className={`transition-[height] duration-75 motion-reduce:transition-none ${level > 0.85 ? 'bg-text' : 'bg-brand'}`} style={{ height: `${pct}%` }} />
      </div>
    )
  const lit = Math.round(level * SEGMENTS)
  return (
    <div {...meter} className='flex h-1.5 w-full gap-0.5'>
      {Array.from({ length: SEGMENTS }, (_, i) => (
        <span key={i} className={`flex-1 ${i >= lit ? 'bg-border' : i < HOT ? 'bg-brand' : 'bg-text'}`} />
      ))}
    </div>
  )
}
