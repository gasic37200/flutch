import { useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { api, ApiRequestError } from '../../../api/client.ts'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import type { BurstRequest } from '../../../api/generated/BurstRequest.ts'
import type { BurstResult } from '../../../api/generated/BurstResult.ts'
import type { Me } from '../../../api/generated/Me.ts'
import { ME_KEY, useAuth } from '../../auth/hooks/useAuth.ts'
import type { OnEvent } from '../../live/hooks/useBroadcastEvents.ts'

const BOOST_MS = 1500

/** SUPER BURST 확인 창·전송·게이지 연출. 잔액은 응답 값으로만 갱신한다. */
export function useBurst(broadcast: BroadcastDetail | undefined, onEvent: OnEvent) {
  const qc = useQueryClient()
  const { me } = useAuth()
  const id = broadcast?.id
  const price = broadcast?.burstPrice ?? 0
  const [open, setOpen] = useState(false)
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

  const short = me !== null && me.balance < price

  const show = () => {
    setMessage(undefined)
    setOpen(true)
  }

  const send = async () => {
    if (id === undefined || sending) return
    setSending(true)
    setMessage(undefined)
    try {
      const r = await api<BurstResult>(`/broadcasts/${id}/bursts`, { method: 'POST', json: { anonymous } satisfies BurstRequest })
      qc.setQueryData<BroadcastDetail>(['broadcast', id], (d) => d && { ...d, gauge: r.gauge })
      qc.setQueryData<Me | null>(ME_KEY, (m) => m && { ...m, balance: r.balance })
      setBoost(Date.now())
      setOpen(false)
    } catch (e) {
      if (e instanceof ApiRequestError) {
        if (e.code === 'WALLET_INSUFFICIENT_BALANCE') setMessage('비트가 부족해요. 충전 후 다시 시도해 주세요.')
        else setMessage(e.message)
      } else setMessage('네트워크 연결을 확인해 주세요.')
    } finally {
      setSending(false)
    }
  }

  return {
    open,
    show,
    close: () => setOpen(false),
    anonymous,
    setAnonymous,
    sending,
    message,
    send,
    price,
    balance: me?.balance,
    short,
    boost: boost > 0,
  }
}
