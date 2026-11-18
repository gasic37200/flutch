import { Link } from 'react-router'
import BeatAmount from '../../../components/ui/BeatAmount.tsx'
import Button from '../../../components/ui/Button.tsx'
import { ChoiceChip } from '../../../components/ui/Chip.tsx'
import Field from '../../../components/ui/Field.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Modal from '../../../components/ui/Modal.tsx'
import { BURST_PRESETS, FILL, type useBurst } from '../hooks/useBurst.ts'
import type { ClutchMode } from '../hooks/useClutch.ts'

const num = (n: number) => n.toLocaleString('ko-KR')

// 게이지 Panel 오른쪽, 클러치 버튼 옆. 로그인한 시청자에게만(LIVE·소유자 아님). 클러치 버튼과 같은 폭.
export default function BurstButton({ burst, mode }: { burst: ReturnType<typeof useBurst>; mode: ClutchMode }) {
  if (mode !== 'open') return null
  const b = burst
  return (
    <>
      <Button variant='burst' size='md' className='w-full md:w-40' onClick={b.show}>
        <Icon name='bolt' className='size-4' />
        SUPER BURST
      </Button>
      <Modal
        open={b.open}
        onClose={b.close}
        title='SUPER BURST'
        footer={
          <>
            <Button variant='neutral' onClick={b.close} disabled={b.sending}>
              취소
            </Button>
            <Button variant='burst' onClick={() => void b.send()} loading={b.sending} disabled={!b.valid || b.short}>
              <Icon name='bolt' className='size-4' />
              {b.valid ? `${num(b.amount)}비트 보내기` : '보내기'}
            </Button>
          </>
        }
      >
        <div className='flex flex-col gap-3'>
          <p className='text-text-muted'>쓴 비트만큼 게이지가 올라요(1비트 = 1점). 스트리머에게 후원으로 전달돼요.</p>
          <div className='flex items-center justify-between gap-3'>
            <span className='text-text-muted'>100%까지</span>
            <BeatAmount amount={b.bitsToFull} unit className='text-lg' />
          </div>
          {b.cooldown && <p className='text-xs text-text-dim'>쿨다운 중이라 100%를 채워도 쿨다운이 끝난 뒤에 SUPER CLUTCH가 터져요.</p>}
          <div role='group' aria-label='SUPER BURST 금액' className='flex flex-wrap gap-2'>
            {BURST_PRESETS.map((n) => (
              <ChoiceChip key={n} selected={!b.custom && b.preset === n} onClick={() => b.selectPreset(n)}>
                {num(n)}
              </ChoiceChip>
            ))}
            <ChoiceChip selected={!b.custom && b.preset === FILL} onClick={() => b.selectPreset(FILL)}>
              100%까지 채우기
            </ChoiceChip>
          </div>
          <Field
            label={`직접 입력 (${num(b.min)} ~ 1,000,000)`}
            inputMode='numeric'
            placeholder='비트'
            maxLength={9}
            value={b.custom ? num(Number(b.custom)) : ''}
            onChange={(e) => b.setCustom(e.target.value)}
            error={b.amountError}
          />
          {b.balance !== undefined && (
            <div className='flex items-center justify-between gap-3'>
              <span className='text-text-muted'>보유</span>
              <BeatAmount amount={b.balance} unit />
            </div>
          )}
          <label className='flex items-center gap-2'>
            <input type='checkbox' checked={b.anonymous} onChange={(e) => b.setAnonymous(e.target.checked)} className='size-[18px] accent-brand' />
            익명으로 보내기
          </label>
          {b.short && (
            <p>
              비트가 부족해요.{' '}
              <Link to='/wallet' className='text-brand underline'>
                충전하러 가기
              </Link>
            </p>
          )}
          <p role='status' className='text-live empty:hidden'>
            {b.message}
          </p>
        </div>
      </Modal>
    </>
  )
}
