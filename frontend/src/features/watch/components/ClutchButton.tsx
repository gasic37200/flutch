import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import type { useClutch } from '../hooks/useClutch.ts'

// 게이지 Panel 오른쪽 슬롯. 비로그인은 로그인 유도, 소유자·비LIVE는 숨긴다. 라벨이 바뀌어도 폭이 변하지 않게 고정한다(안내 문구는 ClutchPanel이 줄 아래에 둔다).
export default function ClutchButton({ clutch }: { clutch: ReturnType<typeof useClutch> }) {
  const { mode, press, sending, next, combo } = clutch
  if (mode === 'owner' || mode === 'closed') return null
  if (mode === 'guest') {
    return (
      <Button to='/login' variant='secondary' size='md' className='col-span-2 w-full md:w-auto' aria-label='클러치 (로그인 필요)'>
        <Icon name='bolt' className='size-4' />
        로그인하고 클러치
      </Button>
    )
  }
  const label = next === 'paid' ? '1비트' : combo > 0 ? `COMBO x${combo}` : 'CLUTCH'
  return (
    <Button onClick={() => void press()} loading={sending} variant='primary' size='md' className='w-full md:w-40' aria-label={`클러치 (${label})`}>
      <Icon name='bolt' className='size-4' />
      <span className='font-mono'>{label}</span>
    </Button>
  )
}
