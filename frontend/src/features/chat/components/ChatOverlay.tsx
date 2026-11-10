import type { Chat } from '../../../api/generated/Chat.ts'
import { profileTone } from '../../../components/ui/mascot.ts'

const SHOWN = 5

// 모바일 방송 화면 위에 얹는 최근 채팅(읽기 전용, 반투명). 삭제·입력은 PC 스튜디오나 시청 화면에서 한다.
export default function ChatOverlay({ items, className = '' }: { items: Chat[]; className?: string }) {
  return (
    <div role='log' aria-live='polite' aria-label='최근 채팅' className={`pointer-events-none flex flex-col justify-end gap-1 overflow-hidden ${className}`}>
      {items.slice(-SHOWN).map((c) => (
        <p key={c.id} className='w-fit max-w-full bg-bg/60 px-2 py-1 text-[13px] leading-snug break-words'>
          <b className={`mr-1.5 ${profileTone(c.author.id).nicknameClass}`}>{c.author.nickname}</b>
          {c.deleted ? <span className='text-text-dim italic'>삭제된 메시지입니다</span> : c.body}
        </p>
      ))}
    </div>
  )
}
