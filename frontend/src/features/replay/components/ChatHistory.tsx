import { useState } from 'react'
import Button from '../../../components/ui/Button.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { formatDuration } from '../../watch/hooks/useWatch.ts'
import { ChoiceChip } from '../../../components/ui/Chip.tsx'
import { useReplayChats } from '../hooks/useRecordings.ts'
import ReplayDonations from './ReplayDonations.tsx'

// 읽기 전용 채팅·후원 목록(탭). 시각은 방송 시작 기준(동기화 리플레이는 범위 밖).
export default function ChatHistory({ broadcastId, startedAt }: { broadcastId: number; startedAt: string }) {
  const q = useReplayChats(broadcastId)
  const [tab, setTab] = useState<'chat' | 'donation'>('chat')
  const start = Date.parse(startedAt)
  return (
    <Panel aria-label='채팅 기록' className='h-full' innerClassName='flex h-full max-h-[70vh] min-h-64 flex-col xl:max-h-175'>
      <div className='flex items-center gap-2 border-b border-surface-2 px-4 py-3' role='group' aria-label='기록 종류'>
        <ChoiceChip selected={tab === 'chat'} onClick={() => setTab('chat')}>
          채팅 기록
        </ChoiceChip>
        <ChoiceChip selected={tab === 'donation'} onClick={() => setTab('donation')}>
          후원
        </ChoiceChip>
        <span className='ml-auto text-xs text-text-dim'>읽기 전용</span>
      </div>
      <div className='flex grow flex-col gap-2 overflow-y-auto px-4 py-3 text-[13px] leading-normal'>
        {tab === 'donation' ? (
          <ReplayDonations broadcastId={broadcastId} />
        ) : q.isError ? (
          <p className='m-auto text-text-dim'>채팅을 불러오지 못했어요</p>
        ) : q.isPending ? (
          <p className='m-auto text-text-dim'>불러오는 중…</p>
        ) : q.items.length === 0 ? (
          <p className='m-auto text-text-dim'>채팅이 없어요</p>
        ) : (
          <>
            {q.hasNextPage && (
              <Button variant='neutral' size='sm' onClick={() => void q.fetchNextPage()} loading={q.isFetchingNextPage}>
                이전 채팅 더 보기
              </Button>
            )}
            {q.items.map((c) => (
              <p key={c.id} className={c.deleted ? 'text-text-dim italic' : ''}>
                <span className='font-mono text-text-dim'>{formatDuration(Date.parse(c.createdAt) - start)}</span>{' '}
                {c.deleted ? (
                  '삭제된 메시지입니다'
                ) : (
                  <>
                    <b className={c.badges.includes('streamer') ? 'text-brand' : 'text-text-muted'}>{c.author.nickname}</b> {c.body}
                    {c.edited && <span className='text-text-dim'> (수정됨)</span>}
                  </>
                )}
              </p>
            ))}
          </>
        )}
      </div>
    </Panel>
  )
}
