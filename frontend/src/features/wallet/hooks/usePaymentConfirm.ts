import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useSearchParams } from 'react-router'
import { api } from '../../../api/client.ts'
import type { ConfirmPaymentRequest } from '../../../api/generated/ConfirmPaymentRequest.ts'
import type { Me } from '../../../api/generated/Me.ts'
import type { PaymentResult } from '../../../api/generated/PaymentResult.ts'
import { ME_KEY, useRequireLogin } from '../../auth/hooks/useAuth.ts'
import { WALLET_KEY } from './useWallet.ts'

const RETURN_KEY = 'flutch:payment-return'

/** 결제창을 연 화면(지갑·시청 화면). 결제 결과 화면의 "다시 시도"가 그리로 돌아간다. */
export function savePaymentReturn() {
  try {
    sessionStorage.setItem(RETURN_KEY, location.pathname)
  } catch {
    // 저장소가 막힌 브라우저는 지갑으로 돌아간다
  }
}

export function paymentRetryPath() {
  let p: string | null = null
  try {
    p = sessionStorage.getItem(RETURN_KEY)
  } catch {
    // 위와 같음
  }
  return p && /^\/(?![/\\])/.test(p) ? p : '/wallet'
}

/**
 * 토스 성공 리다이렉트(`?paymentKey&orderId&amount`)를 그대로 서버 confirm에 보낸다(금액은 서버가 주문과 대조).
 * 같은 키로 다시 불러도 서버가 멱등이라 StrictMode 이중 호출·새로고침에 안전하다. 502면 같은 주문으로 refetch.
 */
export function usePaymentConfirm() {
  const qc = useQueryClient()
  const { me } = useRequireLogin()
  const [sp] = useSearchParams()
  const paymentKey = sp.get('paymentKey')
  const orderId = sp.get('orderId')
  const amount = Number(sp.get('amount'))
  const valid = !!paymentKey && !!orderId && Number.isInteger(amount) && amount > 0

  const query = useQuery({
    queryKey: ['payment-confirm', orderId, paymentKey],
    enabled: valid && !!me,
    retry: false,
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    queryFn: async () => {
      const body = { paymentKey, orderId, amount } as ConfirmPaymentRequest
      const result = await api<PaymentResult>('/payments/confirm', { method: 'POST', json: body })
      // 잔액은 서버 응답 값으로만 갱신한다
      qc.setQueryData<Me | null>(ME_KEY, (m) => m && { ...m, balance: result.balance })
      void qc.invalidateQueries({ queryKey: WALLET_KEY })
      return result
    },
  })
  return { valid, query }
}
