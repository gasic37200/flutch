import { useQueryClient } from '@tanstack/react-query'
import { useRef, useState } from 'react'
import { api, ApiRequestError } from '../../../api/client.ts'
import type { DonateResponse } from '../../../api/generated/DonateResponse.ts'
import type { DonationInput } from '../../../api/generated/DonationInput.ts'
import type { Me } from '../../../api/generated/Me.ts'
import { useToast } from '../../../hooks/useToast.ts'
import { ME_KEY } from '../../auth/hooks/useAuth.ts'
import { MAX_AMOUNT, MIN_AMOUNT } from '../../wallet/format.ts'
import { errorText } from '../../wallet/hooks/errorText.ts'
import { usePaymentOrder } from '../../wallet/hooks/usePaymentOrder.ts'
import { WALLET_KEY } from '../../wallet/hooks/useWallet.ts'
import { donationsKey } from './useDonationList.ts'

export const DONATE_PRESETS = [1000, 5000, 10000]
export const MAX_MESSAGE = 100

/**
 * 후원 창. 잔액이 충분하면 잔액 후원, 모자라면(또는 서버가 409 `WALLET_INSUFFICIENT_BALANCE`) "충전하고 후원하기" 주문.
 * 잔액은 응답 값으로만 갱신하고(낙관적 업데이트 금지), 전송 중에는 다시 누를 수 없다(이중 후원 방지).
 */
export function useDonate(broadcastId: number, balance: number, onClose: () => void) {
  const qc = useQueryClient()
  const toast = useToast()
  const [preset, setPreset] = useState(5000)
  const [custom, setCustom] = useState('')
  const [message, setMessage] = useState('')
  const [anonymous, setAnonymous] = useState(false)
  const [sending, setSending] = useState(false)
  const sendLock = useRef(false)
  const [error, setError] = useState<string>()
  const [fields, setFields] = useState<{ amount?: string; message?: string }>({})
  const payment = usePaymentOrder()

  const amount = custom ? Number(custom) : preset
  const valid = Number.isInteger(amount) && amount >= MIN_AMOUNT && amount <= MAX_AMOUNT
  const shortage = valid ? amount - balance : 0
  const needsCharge = shortage > 0
  const input: DonationInput = { amount, message: message.trim() || undefined, anonymous }

  const startOrder = () => payment.create({ purpose: 'donation', broadcastId, donation: input })

  const send = async () => {
    // 같은 틱 연타는 sending state가 아직 false라 ref로 막는다(서버는 별건 후원으로 받으므로 여기서 막아야 한다)
    if (sendLock.current) return
    sendLock.current = true
    setSending(true)
    try {
      const r = await api<DonateResponse>(`/broadcasts/${broadcastId}/donations`, { method: 'POST', json: input })
      qc.setQueryData<Me | null>(ME_KEY, (m) => m && { ...m, balance: r.balance })
      void qc.invalidateQueries({ queryKey: WALLET_KEY })
      void qc.invalidateQueries({ queryKey: donationsKey(broadcastId) })
      toast({ tone: 'success', message: `${amount.toLocaleString('ko-KR')} Beat를 후원했어요` })
      onClose()
    } catch (e) {
      if (e instanceof ApiRequestError && e.code === 'WALLET_INSUFFICIENT_BALANCE') {
        // 다른 곳에서 잔액을 쓴 경우: 최신 잔액을 받고 곧바로 충전 후원 주문으로 넘어간다
        void qc.invalidateQueries({ queryKey: ME_KEY })
        await startOrder()
      } else if (e instanceof ApiRequestError && (e.fields?.amount || e.fields?.message)) {
        setFields({ amount: e.fields.amount, message: e.fields.message })
      } else if (e instanceof ApiRequestError && e.code === 'DONATION_BANNED_WORD') {
        setFields({ message: e.message })
      } else setError(errorText(e))
    } finally {
      sendLock.current = false
      setSending(false)
    }
  }

  return {
    preset,
    custom,
    message,
    anonymous,
    amount,
    valid,
    shortage,
    needsCharge,
    /** 충전 주문 금액(서버도 같은 식으로 계산): 부족분과 최소 충전 단위 중 큰 값 */
    chargeAmount: Math.max(shortage, MIN_AMOUNT),
    sending: sending || payment.busy,
    error: error ?? payment.error,
    fields: { ...fields, amount: fields.amount ?? (custom && !valid ? '1,000 ~ 1,000,000 사이로 입력해 주세요.' : undefined) },
    order: payment.order,
    selectPreset: (n: number) => {
      setPreset(n)
      setCustom('')
      setFields({})
    },
    setCustom: (v: string) => {
      setCustom(v.replace(/\D/g, '').slice(0, 7))
      setFields({})
    },
    setMessage: (v: string) => {
      setMessage(v)
      setFields({})
    },
    setAnonymous,
    submit: () => {
      if (!valid || sendLock.current || payment.busy) return
      setError(undefined)
      setFields({})
      void (needsCharge ? startOrder() : send())
    },
    back: payment.reset,
  }
}
