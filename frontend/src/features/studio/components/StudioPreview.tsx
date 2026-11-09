import { useEffect, useRef, type ReactNode } from 'react'
import Badge from '../../../components/ui/Badge.tsx'
import Icon from '../../../components/ui/Icon.tsx'

type Props = { stream: MediaStream | null; live: boolean; ended: boolean; children?: ReactNode }

// 미리보기는 소리를 내지 않는다(내 방송 소리가 마이크로 되돌아가는 것 방지).
export default function StudioPreview({ stream, live, ended, children }: Props) {
  const ref = useRef<HTMLVideoElement>(null)
  useEffect(() => {
    if (ref.current) ref.current.srcObject = stream
  }, [stream])
  return (
    <div className='relative aspect-video w-full overflow-hidden bg-bg'>
      <video ref={ref} autoPlay muted playsInline aria-label='송출 미리보기' className={`size-full object-contain ${stream && !ended ? '' : 'invisible'}`} />
      {(!stream || ended) && (
        <div className='absolute inset-0 flex flex-col items-center justify-center gap-3 p-4 text-center text-sm text-text-muted'>
          <Icon name='camera' className='size-8 text-text-dim' />
          {ended ? '화면 공유가 중단됐어요.' : '아직 미리보기가 없어요.'}
          {children}
        </div>
      )}
      {live && <Badge tone='live' className='absolute top-3 left-3' />}
    </div>
  )
}
