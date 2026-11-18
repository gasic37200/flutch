import { useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { api, ApiRequestError } from '../../../api/client.ts'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import type { BurstRequest } from '../../../api/generated/BurstRequest.ts'
import type { BurstResult } from '../../../api/generated/BurstResult.ts'
import type { Me } from '../../../api/generated/Me.ts'
import { ME_KEY, useAuth } from '../../auth/hooks/useAuth.ts'
import type { OnEvent } from '../../live/hooks/useBroadcastEvents.ts'
import { MAX_AMOUNT } from '../../wallet/format.ts'

const BOOST_MS = 1500
export const BURST_PRESETS = [1000, 10_000, 50_000]
/** "100%까지 채우기" 프리셋: 금액이 게이지의 남은 비트를 따라간다 */
export const FILL = 'fill'

/** SUPER BURST 창(금액 선택)·전송·게이지 연출. 1비트 = 1점. 잔액은 응답 값으로만 갱신한다. */
export function useBurst(broadcast: BroadcastDetail | undefined, onEvent: OnEvent) {
  const qc = useQueryClient()
  const { me } = useAuth()
  const id = broadcast?.id
  const min = broadcast?.burstMin ?? 0
  const bitsToFull = broadcast?.gauge.bitsToFull ?? 0
  const cooldown = broadcast?.gauge.phase === 'cooldown'
  const [open, setOpen] = useState(false)
  const [preset, setPreset] = useState<number | typeof FILL>(BURST_PRESETS[0])
  const [custom, setCustom] = useState('')
  const [anonymous, setAnonymous] = useState(false)
  const [sending, setSending] = useState(false)
  const [message, setMessage] = useState<string>()
  // 연출 시작 시각(0이면 꺼짐). 누가 보냈든 버스트 후원 이벤트로 모든 시청자에게 켠다.
  const [boost, setBoost] = useState(0)

  useEffect(() => onEvent('donation.created', (d) => d.donation.kind === 'burst' && setBoost(Date.now())), [onEvent])
  useEffect(() => {
    if (!boost) return
    const t = setTimeout(() => setBoost(0), BOOST_MS)
    return () => clearTimeout(t)
  }, [boost])

  // 남은 비트가 최소 금액보다 적어도 최소 금액은 써야 한다
  const fillAmount = Math.min(Math.max(bitsToFull, min), MAX_AMOUNT)
  const amount = custom ? Number(custom) : preset === FILL ? fillAmount : preset
  const valid = Number.isInteger(amount) && amount >= min && amount <= MAX_AMOUNT
  const short = me !== null && valid && me.balance < amount

  const show = () => {
    setMessage(undefined)
    setPreset(BURST_PRESETS[0])
    setCustom('')
    setOpen(true)
  }

  const send = async () => {
    if (id === undefined || sending || !valid) return
    setSending(true)
    setMessage(undefined)
    try {
      const r = await api<BurstResult>(`/broadcasts/${id}/bursts`, { method: 'POST', json: { amount, anonymous } satisfies BurstRequest })
      qc.setQueryData<BroadcastDetail>(['broadcast', id], (d) => d && { ...d, gauge: r.gauge })
      qc.setQueryData<Me | null>(ME_KEY, (m) => m && { ...m, balance: r.balance })
      setBoost(Date.now())
      setOpen(false)
    } catch (e) {
      if (e instanceof ApiRequestError) {
        if (e.code === 'WALLET_INSUFFICIENT_BALANCE') setMessage('비트가 부족해요. 충전 후 다시 시도해 주세요.')
        else setMessage(e.fields?.amount ?? e.message)
      } else setMessage('네트워크 연결을 확인해 주세요.')
    } finally {
      setSending(false)
    }
  }

  return {
    open,
    show,
    close: () => setOpen(false),
    preset,
    custom,
    selectPreset: (p: number | typeof FILL) => {
      setPreset(p)
      setCustom('')
      setMessage(undefined)
    },
    setCustom: (v: string) => {
      setCustom(v.replace(/\D/g, '').slice(0, 7))
      setMessage(undefined)
    },
    amount,
    valid,
    amountError: custom && !valid ? `${min.toLocaleString('ko-KR')} ~ 1,000,000 사이로 입력해 주세요.` : undefined,
    min,
    bitsToFull,
    cooldown,
    anonymous,
    setAnonymous,
    sending,
    message,
    send,
    balance: me?.balance,
    short,
    boost: boost > 0,
  }
}
