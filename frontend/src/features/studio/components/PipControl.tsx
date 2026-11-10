import type { PipCorner } from '../../../api/generated/PipCorner.ts'
import Switch from '../../../components/ui/Switch.tsx'
import { PIP_CORNERS } from '../../live/pip.ts'

type Props = { on: boolean; corner: PipCorner; onToggle: (on: boolean) => void; onCorner: (c: PipCorner) => void }

// 표현만: 화면 공유 중 웹캠을 따로 보내 시청자 화면 모서리에 겹친다(architecture §9).
export default function PipControl({ on, corner, onToggle, onCorner }: Props) {
  return (
    <div className='flex flex-col gap-2.5'>
      <div className='flex items-center gap-2.5'>
        <span className='flex-1 text-sm'>웹캠 같이 보내기 (PIP)</span>
        <Switch label='웹캠 PIP' checked={on} onChange={onToggle} />
      </div>
      {on && (
        <>
          <div className='flex items-center gap-2.5'>
            <span className='flex-1 text-[13px] text-text-muted'>웹캠 위치</span>
            <div role='radiogroup' aria-label='웹캠 위치' className='grid w-16 grid-cols-2 gap-1'>
              {PIP_CORNERS.map((c) => (
                <button
                  key={c.value}
                  type='button'
                  role='radio'
                  aria-checked={corner === c.value}
                  aria-label={c.label}
                  onClick={() => onCorner(c.value)}
                  className={`h-6 rounded-sm focus-visible:outline-2 focus-visible:outline-text ${corner === c.value ? 'bg-brand' : 'bg-surface-2 hover:bg-border'}`}
                />
              ))}
            </div>
          </div>
          <p className='text-xs text-text-dim'>PC 사양이 낮아 방송이 끊기면 웹캠을 꺼 주세요.</p>
        </>
      )}
    </div>
  )
}
