import { useRef, useState } from 'react'
import { api } from '../../../api/client.ts'
import type { CreateOrderRequest } from '../../../api/generated/CreateOrderRequest.ts'
import type { PaymentOrder } from '../../../api/generated/PaymentOrder.ts'
import { errorText } from './errorText.ts'

/** 결제 주문 만들기(충전·후원 공용). 만든 주문이 있으면 결제 단계로 넘어간다. */
export function usePaymentOrder() {
  const [order, setOrder] = useState<PaymentOrder>()
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string>()
  // 같은 틱 연타는 busy state가 아직 false라 ref로 막는다
  const lock = useRef(false)

  const create = async (req: CreateOrderRequest) => {
    if (lock.current) return
    lock.current = true
    setBusy(true)
    setError(undefined)
    try {
      setOrder(await api<PaymentOrder>('/payments/orders', { method: 'POST', json: req }))
    } catch (e) {
      setError(errorText(e))
    } finally {
      lock.current = false
      setBusy(false)
    }
  }
  return { order, busy, error, create, reset: () => setOrder(undefined) }
}
