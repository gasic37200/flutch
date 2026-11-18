import type { Chat } from '../../api/generated/Chat.ts'
import type { Donation } from '../../api/generated/Donation.ts'
import { ALERT_MS } from '../donation/hooks/useDonationAlerts.ts'

/** 다시보기 타임라인 한 줄. `at`은 영상 내 시점(초) = createdAt − recording.startedAt, `id`는 정렬 순번(스크롤 훅용). */
export type ReplayItem = { id: number; at: number } & ({ kind: 'chat'; chat: Chat } | { kind: 'donation'; donation: Donation })

export const offsetSec = (iso: string, startedAt: string) => (Date.parse(iso) - Date.parse(startedAt)) / 1000

/** 채팅과 후원을 영상 시점순으로 섞는다(같은 시각이면 채팅 먼저). */
export function buildTimeline(chats: Chat[], donations: Donation[], startedAt: string): ReplayItem[] {
  const rows = [
    ...chats.map((chat) => ({ at: offsetSec(chat.createdAt, startedAt), kind: 'chat' as const, chat })),
    ...donations.map((donation) => ({ at: offsetSec(donation.createdAt, startedAt), kind: 'donation' as const, donation })),
  ]
  rows.sort((a, b) => a.at - b.at || (a.kind === b.kind ? 0 : a.kind === 'chat' ? -1 : 1))
  return rows.map((r, i) => ({ ...r, id: i + 1 }))
}

/** 재생 시각 `t`(초)까지의 항목 */
export const visibleAt = (items: ReplayItem[], t: number) => items.filter((i) => i.at <= t)

/** `t`에 영상 위에 띄울 후원: 최근 ALERT_MS 안에 지나간 것 중 가장 늦은 것. 탐색으로 건너뛴 구간의 후원은 여기 안 들어와 쌓이지 않는다. */
export function alertAt(items: ReplayItem[], t: number): Donation | undefined {
  const win = ALERT_MS / 1000
  for (let k = items.length - 1; k >= 0; k--) {
    const i = items[k]
    if (i.kind === 'donation' && i.at <= t && t < i.at + win) return i.donation
  }
}

/** 영상 위에 겹쳐 보일 채팅: 재생 시각 `t` 전 `win`초 안에 올라온 것(오버레이가 최근 몇 줄만 보여 준다). 오래된 채팅이 영상 위에 남지 않게 창을 둔다. */
export const recentChatsAt = (items: ReplayItem[], t: number, win = 30): Chat[] =>
  items.flatMap((i) => (i.kind === 'chat' && i.at <= t && i.at > t - win ? [i.chat] : []))
