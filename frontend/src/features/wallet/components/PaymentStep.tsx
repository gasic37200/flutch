import type { PaymentOrder } from '../../../api/generated/PaymentOrder.ts'
import Button from '../../../components/ui/Button.tsx'
import { AGREEMENT_ID, METHODS_ID, useTossPayment } from '../hooks/useTossPayment.ts'

type Props = { order: PaymentOrder; onBack: () => void; backLabel: string }

// 결제 단계: 토스 결제위젯(결제수단·약관) + 결제하기. 결제창에서 돌아오면 /payments/success가 승인한다.
export default function PaymentStep({ order, onBack, backLabel }: Props) {
  const p = useTossPayment(order)
  const back = (
    <Button variant='neutral' onClick={onBack} disabled={p.paying}>
      {backLabel}
    </Button>
  )
  if (p.status === 'noKey')
    return (
      <div className='flex flex-col gap-3.5'>
        <p role='alert'>결제 설정이 아직 없어요. 운영자가 결제 키를 넣으면 결제할 수 있어요.</p>
        <div className='flex justify-end'>{back}</div>
      </div>
    )
  return (
    <div className='flex flex-col gap-3.5'>
      <div className='flex items-baseline justify-between'>
        <span className='text-text-muted'>{order.orderName}</span>
        <b className='font-mono text-base'>{order.amount.toLocaleString('ko-KR')}원</b>
      </div>
      {/* 위젯은 밝은 배경으로 그려지므로 text 토큰(흰색)을 바탕으로 깐다 */}
      <div id={METHODS_ID} className='min-h-24 bg-text' />
      <div id={AGREEMENT_ID} className='bg-text' />
      {p.status === 'loading' && (
        <p role='status' className='text-xs text-text-dim'>
          결제창을 불러오는 중이에요…
        </p>
      )}
      {p.status === 'error' && <p role='alert'>결제창을 불러오지 못했어요. 잠시 후 다시 시도해 주세요.</p>}
      {p.error && (
        <p role='alert' className='text-xs text-text-muted'>
          {p.error}
        </p>
      )}
      <div className='flex justify-end gap-2'>
        {back}
        <Button onClick={() => void p.pay()} loading={p.paying} disabled={p.status !== 'ready'}>
          {order.amount.toLocaleString('ko-KR')}원 결제하기
        </Button>
      </div>
      <p className='text-xs leading-relaxed text-text-dim'>
        토스페이먼츠 테스트 결제예요. 실제로 돈이 나가지 않아요.
        <br />
        테스트 모드 충전은 환불을 지원하지 않아요.
      </p>
    </div>
  )
}
