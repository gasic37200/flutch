import { useQuery } from '@tanstack/react-query'
import { api } from '../../../api/client.ts'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import type { UserProfile } from '../../../api/generated/UserProfile.ts'
import Avatar from '../../../components/ui/Avatar.tsx'
import { Chip } from '../../../components/ui/Chip.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import FollowButton from '../../follow/components/FollowButton.tsx'
import { CATEGORY_LABEL } from '../../broadcast/categories.ts'
import DonateButton from '../../donation/components/DonateButton.tsx'
import { formatDuration, useNow } from '../hooks/useWatch.ts'

const viewers = (n: number, cls: string, size: string) => (
  <span className={`inline-flex items-center gap-1.5 font-mono font-bold text-live ${cls}`}>
    <Icon name='user' className={size} title='시청자 수' />
    {n.toLocaleString('ko-KR')}
  </span>
)

// 방송 정보(시안 Watch): 제목 · 이름·팔로워·분류·태그 | 시청자·시간·팔로우·후원. 600px 이하는 제목+이름·분류·시청자+후원만.
export default function WatchInfo({ broadcast: b }: { broadcast: BroadcastDetail }) {
  const live = b.status === 'LIVE'
  const now = useNow(live)
  const elapsed = live && b.startedAt ? formatDuration(now - Date.parse(b.startedAt)) : null
  // FollowButton과 같은 캐시 키(['user', id])를 공유한다
  const followers = useQuery({ queryKey: ['user', b.owner.id], queryFn: () => api<UserProfile>(`/users/${b.owner.id}`) }).data?.followerCount
  return (
    <section aria-label='방송 정보' className='flex items-start gap-3.5 max-[600px]:items-center max-[600px]:gap-2.5 max-[600px]:p-3 min-[601px]:flex-wrap'>
      <Avatar userId={b.owner.id} nickname={b.owner.nickname} size='xl' />
      <div className='flex min-w-0 flex-[1_1_20rem] flex-col gap-1.5 max-[600px]:flex-1'>
        <h1 className='text-xl font-bold break-words max-[600px]:truncate max-[600px]:text-sm'>{b.title}</h1>
        <div className='flex flex-wrap items-center gap-x-2.5 gap-y-1 text-sm'>
          <span className='font-bold'>{b.owner.nickname}</span>
          {followers !== undefined && <span className='text-text-dim max-[600px]:hidden'>팔로워 {followers.toLocaleString('ko-KR')}</span>}
          <Chip>{CATEGORY_LABEL[b.category]}</Chip>
          {live && viewers(b.viewerCount, 'text-xs min-[601px]:hidden', 'size-3.5')}
          {b.tags.length > 0 && (
            <ul className='flex flex-wrap gap-x-2 text-xs text-text-dim max-[600px]:hidden'>
              {b.tags.map((t) => (
                <li key={t}>#{t}</li>
              ))}
            </ul>
          )}
        </div>
      </div>
      <div className='flex flex-none items-center gap-2.5 min-[601px]:flex-wrap'>
        {live && viewers(b.viewerCount, 'text-[15px] max-[600px]:hidden', 'size-4')}
        {elapsed && (
          <span className='font-mono text-sm text-text-muted max-[600px]:hidden' aria-label={`방송 경과 시간 ${elapsed}`}>
            {elapsed}
          </span>
        )}
        <span className='max-[600px]:hidden'>
          <FollowButton userId={b.owner.id} idle='secondary' />
        </span>
        <DonateButton broadcast={b} />
      </div>
    </section>
  )
}
