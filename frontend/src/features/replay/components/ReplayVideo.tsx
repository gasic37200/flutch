import { useState } from 'react'
import type { RecordingDetail } from '../../../api/generated/RecordingDetail.ts'
import Icon from '../../../components/ui/Icon.tsx'
import type { Chat } from '../../../api/generated/Chat.ts'
import type { Donation } from '../../../api/generated/Donation.ts'
import ChatOverlay from '../../chat/components/ChatOverlay.tsx'
import DonationAlert from '../../donation/components/DonationAlert.tsx'
import { useAuth } from '../../auth/hooks/useAuth.ts'

function Notice({ title, sub, busy }: { title: string; sub?: string; busy?: boolean }) {
  return (
    <div role='status' className='flex size-full flex-col items-center justify-center gap-2 bg-bg/70 p-4 text-center'>
      <Icon name={busy ? 'spinner' : 'alert'} className={`size-10 text-text-muted ${busy ? 'animate-spin motion-reduce:animate-none' : ''}`} />
      <b className='text-base sm:text-lg'>{title}</b>
      {sub && <span className='text-sm text-text-muted'>{sub}</span>}
    </div>
  )
}

type Props = { recording: RecordingDetail; onPlay: () => void; clip?: { start: number; end?: number }; onTime?: (sec: number) => void; alert?: Donation; chats?: Chat[] }

// 탐색·볼륨·전체화면·배속은 브라우저 기본 컨트롤을 쓴다. 영상이 아니면 같은 비율의 안내 박스를 보여 준다.
/** `clip`이 있으면 그 구간(초)만 재생한다(미디어 프래그먼트 `#t=시작,끝`: 구간 끝에서 멈춘다). `onTime`은 재생 위치(초)를 알리고, `alert`는 영상 위에 후원 알림으로, `chats`는 최근 채팅으로 띄운다(채팅·후원 리플레이). */
export default function ReplayVideo({ recording: r, onPlay, clip, onTime, alert, chats }: Props) {
  const [broken, setBroken] = useState(false)
  const { me } = useAuth()
  return (
    <div className='relative aspect-video w-full overflow-hidden bg-bg bg-cover bg-center' style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }}>
      {r.status === 'ready' && r.videoUrl && !broken ? (
        <video
          src={clip ? `${r.videoUrl}#t=${clip.start}${clip.end ? `,${clip.end}` : ''}` : r.videoUrl}
          poster={r.thumbnailUrl ?? undefined}
          controls
          playsInline
          preload='metadata'
          aria-label={`${r.title} 다시보기`}
          onPlay={onPlay}
          onError={() => setBroken(true)}
          onTimeUpdate={(e) => onTime?.(e.currentTarget.currentTime)}
          onSeeked={(e) => onTime?.(e.currentTarget.currentTime)}
          className='size-full bg-bg object-contain'
        />
      ) : r.status === 'recording' ? (
        <Notice busy title='다시보기 처리 중이에요' sub='준비되면 자동으로 재생할 수 있어요' />
      ) : r.status === 'failed' ? (
        me?.id === r.owner.id ? <Notice title='녹화에 실패했어요' sub='이 방송은 다시보기를 볼 수 없어요' /> : <Notice title='다시보기가 없어요' sub='이 방송은 다시보기를 볼 수 없어요' />
      ) : (
        <Notice title='영상을 불러올 수 없어요' sub='잠시 후 다시 시도해 주세요' />
      )}
      {/* 아래쪽 브라우저 재생 컨트롤을 가리지 않게 띄운다 */}
      {chats && chats.length > 0 && <ChatOverlay items={chats} className='absolute bottom-14 left-3 max-h-[45%] w-[min(22rem,65%)]' />}
      <DonationAlert key={alert?.id} alerts={{ current: alert as Donation, // 타입상 항상 있다고 나오지만 없으면 알림을 그리지 않는다
 waiting: 0, skip: async () => {}, skipping: false }} />
    </div>
  )
}
