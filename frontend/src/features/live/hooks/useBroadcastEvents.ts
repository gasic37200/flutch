import { useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useRef, useState } from 'react'
import { api, ApiRequestError } from '../../../api/client.ts'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import type { WsEvent } from '../../../api/generated/WsEvent.ts'

// shortcut: T08b가 WsEvent에 넣기 전까지 이 이벤트들은 여기서 직접 정의한다, 생성 타입에 들어오면 지운다.
type PendingEvent =
  | { type: 'viewer.count'; data: { count: number } }
  | { type: 'broadcast.reconnecting'; data: { since: string } }
  | { type: 'broadcast.resumed'; data: Record<string, never> }
  | { type: 'broadcast.duration_warning'; data: { endsAt: string } }
type AnyEvent = WsEvent | PendingEvent
type EventType = AnyEvent['type']
type EventData<T extends EventType> = Extract<AnyEvent, { type: T }>['data']

/** 이벤트 구독. 반환한 함수를 호출하면 해제한다(effect 정리 함수로 그대로 쓴다). */
export type OnEvent = <T extends EventType>(type: T, handler: (data: EventData<T>) => void) => () => void

const MAX_OPEN_FAILS = 5 // 한 번도 열리지 못하고 연속 실패하면 포기(Origin 불일치 403 등)
const backoff = (n: number) => Math.min(1000 * 2 ** n, 15_000)

/**
 * 방송 WebSocket 한 개를 열고 이벤트를 TanStack Query 캐시(`[broadcast, id]`)에 반영한다.
 * 채팅·후원·클러치 훅은 반환된 `onEvent`로 같은 연결에 구독한다.
 */
export function useBroadcastEvents(broadcastId: number | undefined) {
  const qc = useQueryClient()
  const [connected, setConnected] = useState(false)
  const handlers = useRef(new Map<string, Set<(data: never) => void>>())

  const onEvent = useCallback<OnEvent>((type, handler) => {
    const set = handlers.current.get(type) ?? new Set()
    set.add(handler as (data: never) => void)
    handlers.current.set(type, set)
    return () => void set.delete(handler as (data: never) => void)
  }, [])

  useEffect(() => {
    if (broadcastId === undefined) return
    const key = ['broadcast', broadcastId]
    const patch = (f: (d: BroadcastDetail) => BroadcastDetail) =>
      qc.setQueryData<BroadcastDetail>(key, (d) => (d ? f(d) : d))
    const refetch = () => void qc.invalidateQueries({ queryKey: key })

    let ws: WebSocket | null = null
    let timer: ReturnType<typeof setTimeout> | undefined
    let closed = false
    let failures = 0 // 열리기 전에 끊긴 연속 횟수
    let attempt = 0

    const apply = (e: AnyEvent) => {
      switch (e.type) {
        case 'broadcast.updated':
          patch((d) => ({ ...d, ...e.data }))
          break
        case 'broadcast.ended':
          patch((d) => ({
            ...d,
            status: 'ENDED',
            reconnecting: false,
            endedAt: e.data.endedAt,
            peakViewers: e.data.peakViewers,
            recording: e.data.recording,
          }))
          refetch()
          break
        case 'viewer.count':
          patch((d) => ({ ...d, viewerCount: e.data.count }))
          break
        case 'broadcast.reconnecting':
          patch((d) => ({ ...d, reconnecting: true }))
          break
        case 'broadcast.resumed':
          patch((d) => ({ ...d, reconnecting: false }))
          break
        case 'resync':
          refetch()
          break
      }
      handlers.current.get(e.type)?.forEach((h) => h(e.data as never))
    }

    const schedule = () => {
      timer = setTimeout(connect, backoff(attempt++))
    }

    const connect = () => {
      const proto = location.protocol === 'https:' ? 'wss' : 'ws'
      const sock = new WebSocket(`${proto}://${location.host}/api/v1/broadcasts/${broadcastId}/events`)
      ws = sock
      let opened = false
      sock.onopen = () => {
        opened = true
        failures = 0
        attempt = 0
        setConnected(true)
        refetch() // 재접속 사이에 놓친 이벤트 보정
      }
      sock.onmessage = (m) => {
        try {
          const e = JSON.parse(String(m.data)) as AnyEvent
          if (typeof e.type === 'string') apply(e)
        } catch {
          // 알 수 없는 메시지는 무시(이벤트 누락 허용)
        }
      }
      sock.onclose = () => {
        setConnected(false)
        if (closed) return
        if (opened) return schedule()
        // 브라우저는 업그레이드 실패 상태 코드를 알려 주지 않는다 → 상세 조회로 404를 확인, 연속 실패는 상한.
        // shortcut: 403(Origin)은 구별하지 못하고 상한으로만 멈춘다.
        if (++failures >= MAX_OPEN_FAILS) return
        api<BroadcastDetail>(`/broadcasts/${broadcastId}`).then(
          () => !closed && schedule(),
          (err: unknown) => {
            if (!closed && !(err instanceof ApiRequestError && err.status === 404)) schedule()
          },
        )
      }
    }

    timer = setTimeout(connect, 0) // StrictMode의 첫 마운트가 곧바로 정리돼도 소켓을 만들지 않게
    return () => {
      closed = true
      clearTimeout(timer)
      ws?.close()
      setConnected(false)
    }
  }, [broadcastId, qc])

  return { connected, onEvent }
}
