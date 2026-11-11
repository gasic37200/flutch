import { useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { useLocation } from 'react-router'
import { ApiRequestError } from '../../../api/client.ts'
import type { BanState } from '../../../api/generated/BanState.ts'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import { useAuth } from '../../auth/hooks/useAuth.ts'
import type { ChatApi } from './useChat.ts'

export const MAX_CHAT = 200

/** until까지 남은 시간(ms). until이 없으면 0. 1초마다 갱신한다. */
function useRemaining(until: string | null) {
  const [now, setNow] = useState(() => Date.now())
  useEffect(() => {
    if (!until) return
    const tick = () => setNow(Date.now())
    const first = setTimeout(tick, 0) // 마운트 시점 이후 흐른 시간을 바로 반영
    const t = setInterval(tick, 1000)
    return () => {
      clearTimeout(first)
      clearInterval(t)
    }
  }, [until])
  return until ? Math.max(0, Date.parse(until) - now) : 0
}

function banText(ban: BanState, remainMs: number) {
  if (ban.permanent) return '이 채널에서 채팅이 제한되었어요'
  const sec = Math.ceil(remainMs / 1000)
  return sec >= 60 ? `${Math.ceil(sec / 60)}분 후 채팅할 수 있어요` : `${sec}초 후 채팅할 수 있어요`
}

/** 채팅 입력창의 상태: 누가 어떤 이유로 쓸 수 없는지(mode)와 입력·전송. */
export function useChatComposer(broadcast: BroadcastDetail | undefined, chat: Pick<ChatApi, 'send'>, onSent?: () => void) {
  const { me, isPending } = useAuth()
  const { pathname, search } = useLocation()
  const [text, setText] = useState('')
  const [sending, setSending] = useState(false)
  const [error, setError] = useState<string>()
  const [blocked, setBlocked] = useState<BanState | null>(null) // 서버가 403 MODERATION_*으로 알려 준 차단

  const ban = blocked ?? broadcast?.viewer?.ban ?? null
  const remainMs = useRemaining(ban && !ban.permanent ? ban.until : null)
  const banned = ban !== null && (ban.permanent || remainMs > 0)

  // 해제는 실시간 이벤트가 없으므로, 차단된 동안만 가끔 상세를 다시 받아 풀렸는지 본다
  const qc = useQueryClient()
  const broadcastId = broadcast?.id
  useEffect(() => {
    if (!banned || broadcastId === undefined) return
    const key = ['broadcast', broadcastId]
    const t = setInterval(async () => {
      await qc.refetchQueries({ queryKey: key })
      if (!qc.getQueryData<BroadcastDetail>(key)?.viewer?.ban) setBlocked(null)
    }, 10_000)
    return () => clearInterval(t)
  }, [banned, broadcastId, qc])

  const mode = isPending ? 'pending' : !me ? 'guest' : broadcast?.status !== 'LIVE' ? 'closed' : banned ? 'banned' : 'open'

  const submit = async () => {
    const body = text.trim()
    if (!body || sending) return
    setSending(true)
    setError(undefined)
    setText('') // 전송 중에 이어 친 글자가 지워지지 않게 먼저 비우고, 실패하면 되돌린다
    try {
      await chat.send(body)
      onSent?.()
    } catch (e) {
      setText((cur) => cur || body)
      if (e instanceof ApiRequestError && e.code === 'MODERATION_TIMED_OUT') {
        setBlocked({ permanent: false, until: new Date(Date.now() + (e.retryAfterSec ?? 60) * 1000).toISOString() })
      } else if (e instanceof ApiRequestError && e.code === 'MODERATION_BANNED') {
        setBlocked({ permanent: true, until: null })
      } else if (e instanceof ApiRequestError && e.code === 'RATE_LIMITED') {
        setError('채팅은 1초에 한 번 보낼 수 있어요')
      } else {
        setError(e instanceof ApiRequestError ? e.message : '네트워크 연결을 확인하고 다시 시도해 주세요.')
      }
    } finally {
      setSending(false)
    }
  }

  return {
    mode,
    loginTo: `/login?next=${encodeURIComponent(pathname + search)}`,
    banText: ban && banned ? banText(ban, remainMs) : '',
    permanent: ban?.permanent ?? false,
    text,
    setText: (v: string) => {
      setText(v)
      setError(undefined)
    },
    sending,
    error,
    canSend: text.trim().length > 0 && !sending,
    submit,
  }
}
