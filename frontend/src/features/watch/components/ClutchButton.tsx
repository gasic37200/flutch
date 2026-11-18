import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import type { useClutch } from '../hooks/useClutch.ts'

// 게이지 Panel 오른쪽 슬롯. 비로그인은 로그인 유도, 소유자·비LIVE는 숨긴다. 콤보는 버튼 위 알약으로 보여 라벨 폭이 변하지 않는다. compact(모바일)는 글자 없는 아이콘 버튼.
export default function ClutchButton({ clutch, compact }: { clutch: ReturnType<typeof useClutch>; compact?: boolean }) {
  const { mode, press, sending, next, combo } = clutch
  if (mode === 'owner' || mode === 'closed') return null
  if (mode === 'guest') {
    return (
      <Button to='/login' variant='secondary' className='col-span-2 w-full md:w-auto' aria-label='클러치 (로그인 필요)'>
        <Icon name='pulse' className='size-[18px]' />
        로그인하고 클러치
      </Button>
    )
  }
  const label = next === 'paid' ? '1 Beat' : 'CLUTCH'
  return (
    <span className='relative flex'>
      {combo > 0 && next === 'free' && (
        <span className='pointer-events-none absolute -top-3.5 -right-2 z-10 rounded-full bg-clutch px-2 py-0.5 font-display text-xs font-bold text-bg italic max-[600px]:hidden'>COMBO x{combo}</span>
      )}
      <Button onClick={() => void press()} loading={sending} variant='primary' size={compact ? 'icon' : 'lg'} className='w-full font-display italic' aria-label={`클러치 (${label})`}>
        <Icon name='pulse' className='size-[18px]' />
        <span className={`font-display font-bold tracking-wide italic ${compact ? 'sr-only' : 'min-w-16 text-center'}`}>{label}</span>
      </Button>
    </span>
  )
}
