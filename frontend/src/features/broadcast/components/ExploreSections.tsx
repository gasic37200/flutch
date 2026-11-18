import { Link } from 'react-router'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { Category } from '../../../api/generated/Category.ts'
import type { RecordingCard } from '../../../api/generated/RecordingCard.ts'
import Avatar from '../../../components/ui/Avatar.tsx'
import { count as fmt, durationLabel, kstDate } from '../../replay/format.ts'
import { CATEGORIES, CATEGORY_LABEL } from '../categories.ts'
import ThumbnailImage from './ThumbnailImage.tsx'

/** 탐색 카테고리 카드(시안): 4:3 텍스처 + 이름 + 라이브 수. 선택을 다시 누르면 전체로 돌아간다. */
// shortcut: 라이브 수는 받아 둔 라이브 목록(최대 50개)에서 센다(전용 집계 API 없음), 50개를 넘는 규모가 되면 집계 API를 붙인다.
export function CategoryCards({ value, onChange, live }: { value: Category | undefined; onChange: (c: Category | undefined) => void; live: Broadcast[] }) {
  return (
    <ul className='grid grid-cols-2 gap-3 min-[601px]:grid-cols-[repeat(auto-fill,minmax(160px,1fr))]'>
      {CATEGORIES.map((c) => {
        const on = value === c
        return (
          <li key={c}>
            <button type='button' aria-pressed={on} onClick={() => onChange(on ? undefined : c)} className='group block w-full text-left focus-visible:outline-none'>
              <div className='transition-[filter] group-hover:drop-shadow-glow group-focus-visible:drop-shadow-glow motion-reduce:transition-none'>
                <div className={`chamfer p-px transition-colors [--c:12px] group-hover:bg-brand group-focus-visible:bg-brand ${on ? 'bg-brand' : 'bg-brand-line'}`}>
                  <div className='chamfer-inner relative flex aspect-[4/3] flex-col justify-end bg-surface-2 bg-cover bg-center p-3' style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }}>
                    <div className='absolute inset-0 bg-linear-to-b from-bg/30 to-bg/90' aria-hidden />
                    <b className='relative text-base'>{CATEGORY_LABEL[c]}</b>
                    <span className='relative text-xs text-text-muted'>
                      라이브 <b className='font-mono text-live'>{live.filter((b) => b.category === c).length}</b>
                    </span>
                  </div>
                </div>
              </div>
            </button>
          </li>
        )
      })}
    </ul>
  )
}

/** 스트리머 섹션: 지금 라이브 중인 스트리머(중복 제거). */
export function StreamerRow({ items }: { items: Broadcast[] }) {
  const owners = [...new Map(items.map((b) => [b.owner.id, b.owner])).values()].slice(0, 8)
  return (
    <ul className='-mx-4 flex gap-5 overflow-x-auto px-4 min-[601px]:mx-0 min-[601px]:flex-wrap min-[601px]:overflow-visible min-[601px]:px-0'>
      {owners.map((o) => (
        <li key={o.id} className='w-24 shrink-0'>
          <Link to={`/users/${o.id}`} className='flex flex-col items-center gap-2'>
            <Avatar userId={o.id} nickname={o.nickname} className='size-[72px]! border-[3px]' />
            <b className='max-w-full truncate text-sm'>{o.nickname}</b>
            <span className='text-xs text-live'>LIVE</span>
          </Link>
        </li>
      ))}
    </ul>
  )
}

/** 다시보기 한 줄 카드(시안: 150px 썸네일 + 제목·메타). */
export function VodRow({ recording: r }: { recording: RecordingCard }) {
  return (
    <Link to={`/replay/${r.id}`} className='group flex min-w-0 items-center gap-3 focus-visible:outline-none'>
      <div className='chamfer relative aspect-video w-[120px] shrink-0 overflow-hidden [--c:6px] min-[601px]:w-[150px]'>
        <ThumbnailImage src={r.thumbnailUrl} category={CATEGORY_LABEL[r.category]} />
        <span className='absolute right-1.5 bottom-1.5 bg-bg/85 px-1.5 py-px font-mono text-[11px]'>{durationLabel(r.durationSec)}</span>
      </div>
      <div className='flex min-w-0 flex-col gap-1'>
        <b className='truncate text-sm group-hover:text-brand-hover'>{r.title}</b>
        <span className='text-xs text-text-muted'>
          {r.owner.nickname} · 조회 {fmt(r.views)} · {kstDate(r.startedAt)}
        </span>
      </div>
    </Link>
  )
}
