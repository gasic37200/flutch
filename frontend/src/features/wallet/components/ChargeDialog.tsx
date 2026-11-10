import Button from '../../../components/ui/Button.tsx'
import { ChoiceChip } from '../../../components/ui/Chip.tsx'
import Field from '../../../components/ui/Field.tsx'
import Modal from '../../../components/ui/Modal.tsx'
import { CHARGE_PRESETS, useCharge } from '../hooks/useCharge.ts'
import PaymentStep from './PaymentStep.tsx'

const won = (n: number) => n.toLocaleString('ko-KR')

// 비트 충전 창: 금액 고르기 → 결제위젯. 열 때마다 마운트해 상태를 새로 시작한다. 주문·결제 금액은 서버 값이 기준.
export default function ChargeDialog({ balance, onClose }: { balance: number; onClose: () => void }) {
  const c = useCharge(balance)
  return (
    <Modal open onClose={onClose} title={c.order ? '결제' : '비트 충전'}>
      {c.order ? (
        <PaymentStep order={c.order} onBack={c.back} backLabel='금액 다시 고르기' />
      ) : (
        <form
          className='flex flex-col gap-3.5'
          onSubmit={(e) => {
            e.preventDefault()
            if (c.canSubmit) c.submit()
          }}
        >
          <div role='group' aria-label='충전 금액' className='flex flex-wrap gap-2'>
            {CHARGE_PRESETS.map((n) => (
              <ChoiceChip key={n} selected={!c.custom && c.preset === n} onClick={() => c.selectPreset(n)}>
                {won(n)}
              </ChoiceChip>
            ))}
          </div>
          <Field
            label='직접 입력 (1,000 ~ 1,000,000)'
            inputMode='numeric'
            placeholder='비트'
            value={c.custom ? won(Number(c.custom)) : ''}
            onChange={(e) => c.setCustom(e.target.value)}
            error={c.error}
          />
          <dl className='flex flex-col gap-1.5'>
            <div className='flex justify-between'>
              <dt className='text-text-muted'>결제 금액</dt>
              <dd className='font-mono font-bold'>{won(c.amount)}원</dd>
            </div>
            <div className='flex justify-between'>
              <dt className='text-text-muted'>충전 후 잔액</dt>
              <dd className='font-mono font-bold text-brand'>{won(c.after)} 비트</dd>
            </div>
          </dl>
          <Button type='submit' size='lg' loading={c.busy} disabled={!c.canSubmit}>
            {won(c.amount)}원 결제하기
          </Button>
          <p className='text-xs leading-relaxed text-text-dim'>테스트 모드 충전은 환불을 지원하지 않아요.</p>
        </form>
      )}
    </Modal>
  )
}
