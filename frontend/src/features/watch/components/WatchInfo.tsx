import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import Avatar from '../../../components/ui/Avatar.tsx'
import Badge from '../../../components/ui/Badge.tsx'
import { Chip } from '../../../components/ui/Chip.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import { CATEGORY_LABEL } from '../../broadcast/categories.ts'
import { formatDuration, useNow } from '../hooks/useWatch.ts'

// 방송 정보: broadcast.updated가 캐시를 바꾸면 그대로 다시 그려진다. 팔로우·후원하기 버튼은 T19·T23이 오른쪽 끝에 붙인다.
export default function WatchInfo({ broadcast: b }: { broadcast: BroadcastDetail }) {
  const live = b.status === 'LIVE'
  const now = useNow(live)
  const elapsed = live && b.startedAt ? formatDuration(now - Date.parse(b.startedAt)) : null
  return (
    <section aria-label='방송 정보' className='flex flex-col gap-3'>
      <div className='flex flex-wrap items-center gap-2'>
        {live ? <Badge tone='live' /> : <Badge tone={b.status === 'ENDED' ? 'ended' : 'ready'} />}
        <h1 className='min-w-0 text-xl font-bold break-words'>{b.title}</h1>
      </div>
      <div className='flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-text-muted'>
        <div className='flex items-center gap-2'>
          <Avatar userId={b.owner.id} nickname={b.owner.nickname} size='sm' />
          <span className='font-semibold text-text'>{b.owner.nickname}</span>
        </div>
        <span>{CATEGORY_LABEL[b.category]}</span>
        {live && (
          <>
            <span className='inline-flex items-center gap-1 font-mono text-live'>
              <Icon name='eye' className='size-4' title='시청자 수' />
              {b.viewerCount}
            </span>
            {elapsed && <span className='font-mono' aria-label={`방송 경과 시간 ${elapsed}`}>{elapsed}</span>}
          </>
        )}
      </div>
      {b.tags.length > 0 && (
        <ul className='flex flex-wrap gap-2'>
          {b.tags.map((t) => (
            <li key={t}>
              <Chip>#{t}</Chip>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}
