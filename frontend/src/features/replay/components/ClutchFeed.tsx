import type { Clip } from '../../../api/generated/Clip.ts'
import LiveCard from '../../../components/ui/LiveCard.tsx'
import { SectionTitle } from '../../broadcast/components/BroadcastGrid.tsx'
import { CATEGORY_LABEL } from '../../broadcast/categories.ts'
import { durationLabel, kstDate } from '../format.ts'
import { useClips } from '../hooks/useRecordings.ts'
import { gridClass } from './ReplayGrid.tsx'
import RecentReplays from './RecentReplays.tsx'

/** 클립 구간만 재생하는 다시보기 주소(`?t=시작&d=길이`, 초). */
const clipPath = (c: Pick<Clip, 'recordingId' | 'startSec' | 'durationSec'>) => `/replay/${c.recordingId}?t=${c.startSec}&d=${c.durationSec}`

/** 홈 CLUTCH FEED(SUPER CLUTCH 가상 클립). 클립이 없으면 "최근 다시보기"로 대신한다(decisions §8). */
export default function ClutchFeed() {
  const items = useClips(4).data?.items ?? []
  if (items.length === 0) return <RecentReplays />
  return (
    <section aria-labelledby='home-feed' className='flex flex-col gap-3 lg:gap-4'>
      <SectionTitle id='home-feed'>CLUTCH FEED</SectionTitle>
      <ul className={gridClass}>
        {items.map((c) => (
          <li key={c.id}>
            <LiveCard
              to={clipPath(c)}
              title={c.broadcast.title}
              streamerId={c.broadcast.owner.id}
              streamerName={c.broadcast.owner.nickname}
              category={CATEGORY_LABEL[c.broadcast.category]}
              thumbnailUrl={c.broadcast.thumbnailUrl}
              durationLabel={durationLabel(c.durationSec)}
              meta={`${kstDate(c.triggeredAt)} SUPER CLUTCH`}
              superClutch
            />
          </li>
        ))}
      </ul>
    </section>
  )
}
