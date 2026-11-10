import { useEffect, useRef, type ReactNode } from 'react'

type Props = {
  stream: MediaStream | null
  ended: boolean
  /** tall=모바일 송출 전 세로 프레임, fill=모바일 방송 중 전체 화면 */
  tall?: boolean
  fill?: boolean
  /** 영상이 없을 때 안내 문구 */
  placeholder: string
  children?: ReactNode
}

// 미리보기는 소리를 내지 않는다(내 방송 소리가 마이크로 되돌아가는 것 방지). <video>는 useThumbnailCapture가 읽는 대상이라 항상 렌더한다.
export default function StudioPreview({ stream, ended, tall, fill, placeholder, children }: Props) {
  const ref = useRef<HTMLVideoElement>(null)
  useEffect(() => {
    if (ref.current) ref.current.srcObject = stream
  }, [stream])
  const frame = fill ? 'absolute inset-0' : tall ? 'relative aspect-[3/4] max-h-[55dvh] w-full' : 'relative aspect-video w-full'
  return (
    <div className={`overflow-hidden bg-bg ${frame}`}>
      <video ref={ref} autoPlay muted playsInline aria-label='송출 미리보기' className={`size-full ${fill ? 'object-cover' : 'object-contain'} ${stream && !ended ? '' : 'invisible'}`} />
      {(!stream || ended) && (
        <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-[url('/brand/bg-speed.jpg')] bg-cover bg-center p-4 text-center">
          <div className='absolute inset-0 bg-bg/60' />
          <span className='relative font-display text-[15px] tracking-widest text-text-muted'>{ended ? '영상이 중단됐어요' : placeholder}</span>
          {children && <div className='relative flex flex-col items-center gap-3'>{children}</div>}
        </div>
      )}
    </div>
  )
}
