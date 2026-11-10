import { useState } from 'react'
import { api } from '../../../api/client.ts'
import type { CreateOrderRequest } from '../../../api/generated/CreateOrderRequest.ts'
import type { PaymentOrder } from '../../../api/generated/PaymentOrder.ts'
import { errorText } from './errorText.ts'

/** 결제 주문 만들기(충전·후원 공용). 만든 주문이 있으면 결제 단계로 넘어간다. */
export function usePaymentOrder() {
  const [order, setOrder] = useState<PaymentOrder>()
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string>()

  const create = async (req: CreateOrderRequest) => {
    if (busy) return
    setBusy(true)
    setError(undefined)
    try {
      setOrder(await api<PaymentOrder>('/payments/orders', { method: 'POST', json: req }))
    } catch (e) {
      setError(errorText(e))
    } finally {
      setBusy(false)
    }
  }
  return { order, busy, error, create, reset: () => setOrder(undefined) }
}
