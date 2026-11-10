import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import type { useClutch } from '../hooks/useClutch.ts'

// 게이지 Panel 오른쪽 슬롯. 비로그인은 로그인 유도, 소유자·비LIVE는 숨긴다. 모바일은 아이콘 버튼만.
export default function ClutchButton({ clutch }: { clutch: ReturnType<typeof useClutch> }) {
  const { mode, press, sending, message, next, combo } = clutch
  if (mode === 'owner' || mode === 'closed') return null
  if (mode === 'guest') {
    return (
      <Button to='/login' variant='secondary' size='md' aria-label='클러치 (로그인 필요)'>
        <Icon name='bolt' className='size-4' />
        <span className='hidden md:inline'>로그인하고 클러치</span>
      </Button>
    )
  }
  const label = next === 'paid' ? '1비트' : combo > 0 ? `COMBO x${combo}` : 'CLUTCH'
  return (
    <div className='flex flex-col items-end gap-1'>
      <Button onClick={() => void press()} loading={sending} variant='primary' size='md' aria-label={`클러치 (${label})`}>
        <Icon name='bolt' className='size-4' />
        <span className='hidden font-mono md:inline'>{label}</span>
      </Button>
      <span className='font-mono text-xs text-text-muted md:hidden' aria-hidden>
        {label}
      </span>
      <p role='status' className='max-w-48 text-right text-xs text-live empty:hidden'>
        {message}
      </p>
    </div>
  )
}
