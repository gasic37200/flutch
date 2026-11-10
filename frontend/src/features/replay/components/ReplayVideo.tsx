import { useState } from 'react'
import type { RecordingDetail } from '../../../api/generated/RecordingDetail.ts'
import Icon from '../../../components/ui/Icon.tsx'

function Notice({ title, sub, busy }: { title: string; sub?: string; busy?: boolean }) {
  return (
    <div role='status' className='flex size-full flex-col items-center justify-center gap-2 bg-bg/70 p-4 text-center'>
      <Icon name={busy ? 'spinner' : 'alert'} className={`size-10 text-text-muted ${busy ? 'animate-spin motion-reduce:animate-none' : ''}`} />
      <b className='text-base sm:text-lg'>{title}</b>
      {sub && <span className='text-sm text-text-muted'>{sub}</span>}
    </div>
  )
}

// 탐색·볼륨·전체화면·배속은 브라우저 기본 컨트롤을 쓴다. 영상이 아니면 같은 비율의 안내 박스를 보여 준다.
export default function ReplayVideo({ recording: r, onPlay }: { recording: RecordingDetail; onPlay: () => void }) {
  const [broken, setBroken] = useState(false)
  return (
    <div className='relative aspect-video w-full overflow-hidden bg-bg bg-cover bg-center' style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }}>
      {r.status === 'ready' && r.videoUrl && !broken ? (
        <video
          src={r.videoUrl}
          poster={r.thumbnailUrl ?? undefined}
          controls
          playsInline
          preload='metadata'
          aria-label={`${r.title} 다시보기`}
          onPlay={onPlay}
          onError={() => setBroken(true)}
          className='size-full bg-bg object-contain'
        />
      ) : r.status === 'recording' ? (
        <Notice busy title='다시보기 처리 중이에요' sub='준비되면 자동으로 재생할 수 있어요' />
      ) : r.status === 'failed' ? (
        <Notice title='녹화에 실패했어요' sub='이 방송은 다시보기를 볼 수 없어요' />
      ) : (
        <Notice title='영상을 불러올 수 없어요' sub='잠시 후 다시 시도해 주세요' />
      )}
    </div>
  )
}
