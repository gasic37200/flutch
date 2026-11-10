import { useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { api, ApiRequestError } from '../../../api/client.ts'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import type { ClutchResult } from '../../../api/generated/ClutchResult.ts'
import type { SuperClutch } from '../../../api/generated/SuperClutch.ts'
import { useAuth } from '../../auth/hooks/useAuth.ts'
import { ME_KEY } from '../../auth/hooks/useAuth.ts'
import type { Me } from '../../../api/generated/Me.ts'
import type { OnEvent } from '../../live/hooks/useBroadcastEvents.ts'

const FREE_WINDOW_MS = 30_000
const REFRESH_MS = 20_000 // 점수가 5분 창 밖으로 빠지는 감소는 이벤트가 없어 주기적으로 다시 받는다
const OVERLAY_MS = 3000

export type ClutchMode = 'guest' | 'owner' | 'closed' | 'open'

/** 클러치 버튼 상태: 콤보(내 화면만)·잔액 부족 안내·SUPER CLUTCH 연출 표시. */
export function useClutch(broadcast: BroadcastDetail | undefined, onEvent: OnEvent) {
  const qc = useQueryClient()
  const { me, isPending } = useAuth()
  const id = broadcast?.id
  const [last, setLast] = useState<{ combo: number; paid: boolean }>()
  const [sending, setSending] = useState(false)
  const [message, setMessage] = useState<string>()
  const [superClutch, setSuperClutch] = useState<SuperClutch>()

  const live = broadcast?.status === 'LIVE'
  const mode: ClutchMode = isPending || !me ? 'guest' : broadcast?.viewer?.isOwner ? 'owner' : !live ? 'closed' : 'open'

  useEffect(() => {
    if (!live || id === undefined) return
    const t = setInterval(() => void qc.invalidateQueries({ queryKey: ['broadcast', id] }), REFRESH_MS)
    return () => clearInterval(t)
  }, [live, id, qc])

  useEffect(
    () =>
      onEvent('clutch.super', (s) => {
        setSuperClutch(s)
        setTimeout(() => setSuperClutch((cur) => (cur === s ? undefined : cur)), OVERLAY_MS)
      }),
    [onEvent],
  )

  // shortcut: 30초 창을 "마지막 클릭 후 30초"로 근사한다(서버는 슬라이딩 창). 표시용일 뿐 서버가 최종 판단한다.
  const fresh = last
  const freeLeft = fresh ? (fresh.paid ? 0 : Math.max(0, 5 - fresh.combo)) : (broadcast?.viewer?.freeClutchesLeft ?? 5)
  const next = freeLeft > 0 ? 'free' : 'paid'

  const press = async () => {
    if (id === undefined || sending) return
    setSending(true)
    setMessage(undefined)
    try {
      const r = await api<ClutchResult>(`/broadcasts/${id}/clutches`, {
        method: 'POST',
      })
      const l = { combo: r.paid ? 5 : r.combo, paid: r.paid }
      setLast(l)
      setTimeout(() => setLast((cur) => (cur === l ? undefined : cur)), FREE_WINDOW_MS)
      qc.setQueryData<BroadcastDetail>(
        ['broadcast', id],
        (d) =>
          d && {
            ...d,
            gauge: r.gauge,
            viewer: d.viewer && {
              ...d.viewer,
              freeClutchesLeft: r.freeClutchesLeft,
            },
          },
      )
      if (r.balance !== null) qc.setQueryData<Me | null>(ME_KEY, (m) => m && { ...m, balance: r.balance ?? m.balance })
    } catch (e) {
      if (e instanceof ApiRequestError) {
        if (e.code === 'WALLET_INSUFFICIENT_BALANCE') setMessage('비트가 부족해요. 충전 후 다시 눌러 주세요.')
        else if (e.code === 'MODERATION_BANNED') setMessage('이 채널에서는 유료 클러치를 할 수 없어요.')
        else if (e.status === 429) setMessage('너무 빨라요. 잠시 후 다시 눌러 주세요.')
        else setMessage(e.message)
      } else setMessage('네트워크 연결을 확인해 주세요.')
    } finally {
      setSending(false)
    }
  }

  return {
    mode,
    press,
    sending,
    message,
    superClutch,
    next,
    combo: fresh && !fresh.paid ? fresh.combo : 0,
  }
}
