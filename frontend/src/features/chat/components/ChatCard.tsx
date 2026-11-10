import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import type { ChatApi, SystemLine } from '../hooks/useChat.ts'
import { useChatComposer } from '../hooks/useChatComposer.ts'
import { useChatScroll } from '../hooks/useChatScroll.ts'
import DonationLine from '../../donation/components/DonationLine.tsx'
import ChatComposer from './ChatComposer.tsx'
import ChatMessage from './ChatMessage.tsx'

type Props = {
  chat: ChatApi
  broadcast: BroadcastDetail | undefined
  /** 패널 높이(고정 높이여야 목록이 안에서 스크롤된다) */
  className: string
  /** 제목 줄 표시 방식. 모바일 탭처럼 이미 탭 이름이 있으면 `hidden xl:block` */
  headerClassName?: string
}

// 저장되지 않는 실시간 줄: 차단 알림(아이콘·글자로 구분)과 후원 강조 줄.
const systemLine = (s: SystemLine) =>
  s.donation ? (
    <DonationLine key={`s${s.key}`} donation={s.donation} />
  ) : (
    <li key={`s${s.key}`} className='flex items-center gap-1.5 text-xs text-text-muted'>
      <Icon name='info' className='size-3.5 shrink-0' />
      {s.text}
    </li>
  )

// 채팅 패널: 제목 · 목록(role=log) · 입력창. 시청 화면·스튜디오가 같이 쓴다.
export default function ChatCard({ chat, broadcast, className, headerClassName = '' }: Props) {
  const { ref, contentRef, onScroll, toBottom, unseen } = useChatScroll(chat)
  const composer = useChatComposer(broadcast, chat, toBottom)
  const isOwner = broadcast?.viewer?.isOwner ?? false
  const live = broadcast?.status === 'LIVE'
  return (
    <Panel aria-label='채팅' className={className} innerClassName='flex h-full flex-col'>
      <div className={`border-b border-surface-2 px-4 py-3.5 ${headerClassName}`}>
        <b className='text-[15px]'>채팅</b>
      </div>
      <div className='relative flex min-h-0 grow flex-col'>
        <div
          ref={ref}
          onScroll={onScroll}
          role='log'
          aria-live='polite'
          aria-label='채팅 메시지'
          tabIndex={0}
          className='min-h-0 grow overflow-y-auto px-4 py-3 text-sm leading-normal'
        >
          <div ref={contentRef}>
            {chat.hasMore && (
              <div className='mb-3 flex justify-center'>
                <Button size='sm' variant='neutral' onClick={() => void chat.loadMore()} loading={chat.loadingMore}>
                  {chat.moreFailed ? '불러오지 못했어요. 다시 시도' : '이전 채팅 불러오기'}
                </Button>
              </div>
            )}
            {chat.systems.some((s) => s.afterId === null) && <ul className='mb-2.5 flex flex-col gap-2.5'>{chat.systems.filter((s) => s.afterId === null).map(systemLine)}</ul>}
            {chat.error && chat.items.length === 0 ? (
              <div className='flex flex-col items-center gap-2 py-6 text-text-dim'>
                <p>채팅을 불러오지 못했어요.</p>
                <Button size='sm' variant='neutral' onClick={() => void chat.refetch()}>
                  다시 불러오기
                </Button>
              </div>
            ) : chat.items.length === 0 ? (
              !chat.loading && <p className='py-6 text-center text-text-dim'>아직 채팅이 없어요.</p>
            ) : (
              <ul className='flex flex-col gap-2.5'>
                {chat.items.flatMap((item) => [
                  <ChatMessage key={item.id} chat={chat} item={item} isOwner={isOwner} live={live} />,
                  ...chat.systems.filter((s) => s.afterId === item.id).map(systemLine),
                ])}
              </ul>
            )}
          </div>
        </div>
        {unseen > 0 && (
          <div className='pointer-events-none absolute inset-x-0 bottom-2 flex justify-center'>
            <button
              type='button'
              onClick={toBottom}
              className='chamfer pointer-events-auto flex items-center gap-1.5 bg-brand px-3 py-1.5 text-xs font-bold text-bg [--c:6px]'
            >
              <Icon name='arrowDown' className='size-3.5' />새 메시지 {unseen}개
            </button>
          </div>
        )}
      </div>
      <ChatComposer c={composer} />
    </Panel>
  )
}
