import { useState } from 'react'
import { MAX_AMOUNT, MIN_AMOUNT } from '../format.ts'
import { usePaymentOrder } from './usePaymentOrder.ts'

export const CHARGE_PRESETS = [1000, 5000, 10000, 50000]
export const BALANCE_LIMIT = 3_000_000

/** 충전 창: 프리셋·직접 입력 → 주문 생성 → 결제 단계. 금액은 서버가 다시 검사한다. */
export function useCharge(balance: number) {
  const [preset, setPreset] = useState(10000)
  const [custom, setCustom] = useState('')
  const payment = usePaymentOrder()
  const amount = custom ? Number(custom) : preset
  const valid = Number.isInteger(amount) && amount >= MIN_AMOUNT && amount <= MAX_AMOUNT
  const after = balance + (valid ? amount : 0)
  const error = custom && !valid ? '1,000 ~ 1,000,000 사이로 입력해 주세요.' : after > BALANCE_LIMIT ? '보유 한도(3,000,000 Beat)를 넘을 수 없어요.' : payment.error

  return {
    preset,
    custom,
    amount,
    after,
    error,
    selectPreset: (n: number) => {
      setPreset(n)
      setCustom('')
    },
    setCustom: (v: string) => setCustom(v.replace(/\D/g, '').slice(0, 7)),
    canSubmit: valid && after <= BALANCE_LIMIT,
    order: payment.order,
    busy: payment.busy,
    submit: () => void payment.create({ purpose: 'charge', amount }),
    back: payment.reset,
  }
}
