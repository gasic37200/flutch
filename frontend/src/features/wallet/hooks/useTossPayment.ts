import { ANONYMOUS, loadTossPayments, type TossPaymentsWidgets } from '@tosspayments/tosspayments-sdk'
import { useEffect, useRef, useState } from 'react'
import type { PaymentOrder } from '../../../api/generated/PaymentOrder.ts'

export const METHODS_ID = 'toss-methods'
export const AGREEMENT_ID = 'toss-agreement'

/**
 * 토스 결제위젯(v2). 주문 응답의 clientKey로 초기화하고 결제수단·약관 UI를 그린 뒤, pay()가 결제창으로 보낸다.
 * 결제 승인은 성공 리다이렉트 페이지가 서버 confirm으로 한다.
 */
export function useTossPayment(order: PaymentOrder) {
  const widgets = useRef<TossPaymentsWidgets>(null)
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>('loading')
  const [paying, setPaying] = useState(false)
  const [error, setError] = useState<string>()
  const { clientKey, amount } = order

  useEffect(() => {
    if (!clientKey) return
    let cancelled = false
    const rendered: { destroy: () => Promise<void> }[] = []
    // destroy 실패는 이미 사라진 UI라 무시한다
    const destroyAll = () => rendered.forEach((r) => void r.destroy().catch(() => undefined))
    const run = async () => {
      try {
        const w = (await loadTossPayments(clientKey)).widgets({ customerKey: ANONYMOUS })
        await w.setAmount({ currency: 'KRW', value: amount })
        rendered.push(await w.renderPaymentMethods({ selector: `#${METHODS_ID}` }))
        rendered.push(await w.renderAgreement({ selector: `#${AGREEMENT_ID}` }))
        if (cancelled) return destroyAll()
        widgets.current = w
        setStatus('ready')
      } catch {
        if (!cancelled) setStatus('error')
      }
    }
    // StrictMode의 첫 마운트가 곧바로 정리돼도 위젯을 그리지 않게(같은 자리에 두 번 그리면 겹친다)
    const timer = setTimeout(() => void run(), 0)
    return () => {
      cancelled = true
      clearTimeout(timer)
      widgets.current = null
      destroyAll()
    }
  }, [clientKey, amount])

  const pay = async () => {
    if (!widgets.current || paying) return
    setPaying(true)
    setError(undefined)
    try {
      await widgets.current.requestPayment({
        orderId: order.orderId,
        orderName: order.orderName,
        successUrl: `${location.origin}/payments/success`,
        failUrl: `${location.origin}/payments/fail`,
      })
    } catch (e) {
      // 결제창을 닫은 경우(USER_CANCEL)도 여기로 온다. 실패 사유는 failUrl 쪽 화면이 보여 준다.
      setError(e instanceof Error && e.message ? e.message : '결제를 시작하지 못했어요.')
    } finally {
      setPaying(false)
    }
  }

  return { status: clientKey ? status : ('noKey' as const), paying, error, pay }
}
