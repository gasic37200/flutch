import type { CSSProperties, ReactNode } from 'react'
import type { Chat } from '../../../api/generated/Chat.ts'
import { profileTone } from '../../../components/ui/mascot.ts'
import { beats, donorName } from '../../donation/format.ts'
import type { SystemLine } from '../hooks/useChat.ts'

type Props = {
  items: Chat[]
  /** 후원 강조 줄(시청 전체화면). 채팅 사이 제자리에 끼운다 */
  systems?: SystemLine[]
  shown?: number
  className?: string
  style?: CSSProperties
}

// 영상 위에 얹는 최근 채팅(읽기 전용, 반투명): 모바일 방송 화면, 시청 전체화면(시안 WatchFullscreen). 삭제·입력은 PC 스튜디오나 시청 화면에서 한다.
export default function ChatOverlay({ items, systems = [], shown = 5, className = 'text-[13px] leading-snug', style }: Props) {
  const donations = systems.filter((s) => s.donation)
  const line = (key: string, body: ReactNode, hl = false) => (
    <p key={key} className={`w-fit max-w-full px-2 py-[3px] break-words ${hl ? 'bg-brand/18 ring-1 ring-brand/50 ring-inset' : 'bg-bg/60'}`}>
      {body}
    </p>
  )
  const donationLine = (s: SystemLine) =>
    s.donation &&
    line(
      `s${s.key}`,
      <>
        <b className='mr-1.5 text-brand'>
          {donorName(s.donation)} · {beats(s.donation.amount)} Beat
        </b>
        {s.donation.message}
      </>,
      true,
    )
  const lines = [
    ...donations.filter((s) => s.afterId === null).map(donationLine),
    ...items.flatMap((c) => [
      line(
        `c${c.id}`,
        <>
          <b className={`mr-1.5 ${profileTone(c.author.id).nicknameClass}`}>{c.author.nickname}</b>
          {c.deleted ? <span className='text-text-dim italic'>삭제된 메시지입니다</span> : c.body}
        </>,
      ),
      ...donations.filter((s) => s.afterId === c.id).map(donationLine),
    ]),
  ]
  return (
    <div role='log' aria-live='polite' aria-label='최근 채팅' style={style} className={`pointer-events-none flex flex-col justify-end gap-1 overflow-hidden ${className}`}>
      {lines.slice(-shown)}
    </div>
  )
}
