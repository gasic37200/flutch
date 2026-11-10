import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import LiveCard from '../../../components/ui/LiveCard.tsx'
import { CATEGORY_LABEL } from '../../broadcast/categories.ts'
import { gridClass } from '../../replay/components/ReplayGrid.tsx'
import { kstDate } from '../../replay/format.ts'
import { formatDuration } from '../../watch/hooks/useWatch.ts'

const RECORDING_NOTE = { ready: '다시보기', recording: '다시보기 처리 중', failed: '녹화에 실패했어요', deleted: '보관 기간이 지났어요' }

function PastCard({ broadcast: b }: { broadcast: Broadcast }) {
  const rec = b.recording
  const ready = rec?.status === 'ready'
  const ms = b.startedAt && b.endedAt ? Date.parse(b.endedAt) - Date.parse(b.startedAt) : undefined
  return (
    <LiveCard
      to={ready ? `/replay/${rec.id}` : `/live/${b.id}`}
      title={b.title}
      streamerId={b.owner.id}
      streamerName={b.owner.nickname}
      category={CATEGORY_LABEL[b.category]}
      thumbnailUrl={b.thumbnailUrl}
      durationLabel={ready && ms !== undefined ? formatDuration(ms) : undefined}
      meta={`${kstDate(b.endedAt ?? b.createdAt)} · ${rec ? RECORDING_NOTE[rec.status] : '다시보기 없음'}`}
    />
  )
}

/** 지난(ENDED) 방송 목록. 다시보기가 준비된 방송은 다시보기로, 아니면 종료 화면으로 간다. */
export default function PastBroadcasts({ items }: { items: Broadcast[] }) {
  return (
    <ul className={gridClass}>
      {items.map((b) => (
        <li key={b.id}>
          <PastCard broadcast={b} />
        </li>
      ))}
    </ul>
  )
}
