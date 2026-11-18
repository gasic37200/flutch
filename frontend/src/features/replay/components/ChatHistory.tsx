import { useState } from 'react'
import Panel from '../../../components/ui/Panel.tsx'
import { ChoiceChip } from '../../../components/ui/Chip.tsx'
import { useChatScroll } from '../../chat/hooks/useChatScroll.ts'
import DonationLine from '../../donation/components/DonationLine.tsx'
import { formatDuration } from '../../watch/hooks/useWatch.ts'
import type { useReplayTimeline } from '../hooks/useReplayTimeline.ts'
import { visibleAt } from '../timeline.ts'

type Props = { timeline: ReturnType<typeof useReplayTimeline>; time: number; synced: boolean }

const NO_MORE = { hasMore: false, loadMore: () => {} }

// 읽기 전용 채팅·후원 한 타임라인. 영상이 있으면 재생 시각까지만 보여 주고(라이브처럼 아래로 쌓임), "전체 기록 보기"로 전부 본다.
export default function ChatHistory({ timeline, time, synced }: Props) {
  const [all, setAll] = useState(false)
  const sync = synced && !all
  const items = sync ? visibleAt(timeline.items, time) : timeline.items
  const { ref, contentRef, onScroll } = useChatScroll({ items, ...NO_MORE })
  return (
    <Panel aria-label='채팅 기록' className='h-full' innerClassName='flex h-full max-h-[70vh] min-h-64 flex-col xl:max-h-175'>
      <div className='flex items-center gap-2 border-b border-surface-2 px-4 py-3'>
        <b className='mr-auto text-[15px]'>채팅 기록</b>
        {synced && (
          <span>
            <ChoiceChip selected={all} onClick={() => setAll((v) => !v)}>
              전체 기록 보기
            </ChoiceChip>
          </span>
        )}
        <span className='text-xs text-text-dim'>읽기 전용</span>
      </div>
      <div ref={ref} onScroll={onScroll} role='log' aria-label='채팅·후원 기록' tabIndex={0} className='flex min-h-0 grow flex-col overflow-y-auto px-4 py-3 text-[13px] leading-normal'>
        <div ref={contentRef} className='flex min-h-full flex-col'>
          {timeline.error ? (
            <p className='m-auto text-text-dim'>채팅·후원을 불러오지 못했어요</p>
          ) : timeline.loading && items.length === 0 ? (
            <p className='m-auto text-text-dim'>불러오는 중…</p>
          ) : items.length === 0 ? (
            <p className='m-auto text-text-dim'>{sync ? '아직 채팅이나 후원이 없어요' : '채팅이나 후원이 없어요'}</p>
          ) : (
            <ul className='flex flex-col gap-2'>
              {items.map((i) =>
                i.kind === 'donation' ? (
                  <DonationLine key={`d${i.donation.id}`} donation={i.donation} />
                ) : (
                  <li key={`c${i.chat.id}`} className={i.chat.deleted ? 'text-text-dim italic' : ''}>
                    <span className='font-mono text-text-dim'>{formatDuration(i.at * 1000)}</span>{' '}
                    {i.chat.deleted ? (
                      '삭제된 메시지입니다'
                    ) : (
                      <>
                        <b className={i.chat.badges.includes('streamer') ? 'text-brand' : 'text-text-muted'}>{i.chat.author.nickname}</b> {i.chat.body}
                        {i.chat.edited && <span className='text-text-dim'> (수정됨)</span>}
                      </>
                    )}
                  </li>
                ),
              )}
            </ul>
          )}
        </div>
      </div>
    </Panel>
  )
}
