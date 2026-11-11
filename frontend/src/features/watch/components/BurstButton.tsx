import { Link } from 'react-router'
import BeatAmount from '../../../components/ui/BeatAmount.tsx'
import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Modal from '../../../components/ui/Modal.tsx'
import type { useBurst } from '../hooks/useBurst.ts'
import type { ClutchMode } from '../hooks/useClutch.ts'

// 게이지 Panel 오른쪽, 클러치 버튼 옆. 로그인한 시청자에게만(LIVE·소유자 아님). 클러치 버튼과 같은 폭.
export default function BurstButton({ burst, mode }: { burst: ReturnType<typeof useBurst>; mode: ClutchMode }) {
  if (mode !== 'open') return null
  const { open, show, close, anonymous, setAnonymous, sending, message, send, price, balance, short } = burst
  return (
    <>
      <Button variant='burst' size='md' className='w-full md:w-40' onClick={show} aria-label={`SUPER BURST (${price.toLocaleString('ko-KR')}비트)`}>
        <Icon name='bolt' className='size-4' />
        SUPER BURST
      </Button>
      <Modal
        open={open}
        onClose={close}
        title='SUPER BURST'
        footer={
          <>
            <Button variant='neutral' onClick={close} disabled={sending}>
              취소
            </Button>
            <Button variant='burst' onClick={() => void send()} loading={sending} disabled={short}>
              <Icon name='bolt' className='size-4' />
              보내기
            </Button>
          </>
        }
      >
        <div className='flex flex-col gap-3'>
          <p className='text-text-muted'>비트를 한 번에 써서 게이지를 크게 올려요. 스트리머에게 후원으로 전달돼요.</p>
          <dl className='grid grid-cols-[auto_1fr] items-center gap-x-4 gap-y-2'>
            <dt className='text-text-muted'>가격</dt>
            <dd>
              <BeatAmount amount={price} unit />
            </dd>
            {balance !== undefined && (
              <>
                <dt className='text-text-muted'>보유</dt>
                <dd>
                  <BeatAmount amount={balance} unit />
                </dd>
              </>
            )}
          </dl>
          <label className='flex items-center gap-2'>
            <input type='checkbox' checked={anonymous} onChange={(e) => setAnonymous(e.target.checked)} className='size-[18px] accent-brand' />
            익명으로 보내기
          </label>
          {short && (
            <p>
              비트가 부족해요.{' '}
              <Link to='/wallet' className='text-brand underline'>
                충전하러 가기
              </Link>
            </p>
          )}
          <p role='status' className='text-live empty:hidden'>
            {message}
          </p>
        </div>
      </Modal>
    </>
  )
}
