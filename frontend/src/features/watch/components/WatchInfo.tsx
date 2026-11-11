import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import Avatar from '../../../components/ui/Avatar.tsx'
import { Chip } from '../../../components/ui/Chip.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import FollowButton from '../../follow/components/FollowButton.tsx'
import { CATEGORY_LABEL } from '../../broadcast/categories.ts'
import DonateButton from '../../donation/components/DonateButton.tsx'
import { formatDuration, useNow } from '../hooks/useWatch.ts'

// 방송 정보: broadcast.updated가 캐시를 바꾸면 그대로 다시 그려진다. 후원하기(T23)는 마지막 div(오른쪽 끝)의 팔로우 버튼 옆에 붙인다.
export default function WatchInfo({ broadcast: b }: { broadcast: BroadcastDetail }) {
  const live = b.status === 'LIVE'
  const now = useNow(live)
  const elapsed = live && b.startedAt ? formatDuration(now - Date.parse(b.startedAt)) : null
  return (
    <section aria-label='방송 정보' className='flex flex-wrap items-start gap-3 sm:gap-3.5'>
      <Avatar userId={b.owner.id} nickname={b.owner.nickname} size='xl' />
      <div className='flex min-w-0 flex-[1_1_16rem] flex-col gap-1.5'>
        <h1 className='text-base font-bold break-words sm:text-xl'>{b.title}</h1>
        <div className='flex flex-wrap items-center gap-x-2.5 gap-y-1.5 text-sm'>
          <span className='font-bold'>{b.owner.nickname}</span>
          <Chip>{CATEGORY_LABEL[b.category]}</Chip>
          {b.tags.length > 0 && (
            <ul className='flex flex-wrap gap-x-2 text-xs text-text-dim'>
              {b.tags.map((t) => (
                <li key={t}>#{t}</li>
              ))}
            </ul>
          )}
        </div>
      </div>
      <div className='flex w-full items-center gap-2.5 sm:w-auto'>
        <FollowButton userId={b.owner.id} />
        {live && (
          <div className='flex grow items-center justify-center gap-3 sm:grow-0'>
            <span className='inline-flex items-center gap-1.5 font-mono text-[15px] font-bold text-live'>
              <Icon name='user' className='size-4' title='시청자 수' />
              {b.viewerCount.toLocaleString('ko-KR')}
            </span>
            {elapsed && (
              <span className='font-mono text-sm text-text-muted' aria-label={`방송 경과 시간 ${elapsed}`}>
                {elapsed}
              </span>
            )}
          </div>
        )}
        <DonateButton broadcast={b} />
      </div>
    </section>
  )
}
