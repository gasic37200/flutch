import { useEffect, useRef, type ReactNode } from 'react'
import type { OverlayLayout } from '../../../api/generated/OverlayLayout.ts'
import { camStyle, DEFAULT_LAYOUT } from '../../live/overlayLayout.ts'

type Props = {
  stream: MediaStream | null
  ended: boolean
  /** tall=모바일 송출 전 세로 프레임, fill=모바일 방송 중 전체 화면 */
  tall?: boolean
  fill?: boolean
  /** 영상이 없을 때 안내 문구 */
  placeholder: string
  /** 화면 모드에서 같이 보내는 웹캠: 시청자 화면과 같은 배치 위치에 겹쳐 보여 준다 */
  pip?: MediaStream
  layout?: OverlayLayout
  /** 영상 위 층(후원 알림·배치 편집). 프레임이 @container라 알림 크기가 미리보기 폭을 따른다 */
  overlay?: ReactNode
  className?: string
  children?: ReactNode
}

// 미리보기는 소리를 내지 않는다(내 방송 소리가 마이크로 되돌아가는 것 방지). <video>는 useThumbnailCapture가 읽는 대상이라 항상 렌더한다.
export default function StudioPreview({ stream, ended, tall, fill, placeholder, pip, layout = DEFAULT_LAYOUT, overlay, className = '', children }: Props) {
  const ref = useRef<HTMLVideoElement>(null)
  const pipRef = useRef<HTMLVideoElement>(null)
  useEffect(() => {
    if (pipRef.current) pipRef.current.srcObject = pip ?? null
  }, [pip])
  useEffect(() => {
    if (ref.current) ref.current.srcObject = stream
  }, [stream])
  // 데스크톱 프레임은 낮은 화면에서도 16:9를 지켜(높이 상한 → 폭을 줄임) 배치 %가 시청자 화면과 같게 한다
  const frame = fill ? 'absolute inset-0' : tall ? 'relative aspect-[3/4] max-h-[45dvh] w-full' : 'relative mx-auto aspect-video w-full max-w-[calc(max(15rem,calc(100dvh-22rem))*16/9)]'
  return (
    <div className={`@container overflow-hidden bg-bg ${frame} ${className}`}>
      <video ref={ref} autoPlay muted playsInline aria-label='송출 미리보기' className={`size-full ${fill ? 'object-cover' : 'object-contain'} ${stream && !ended ? '' : 'invisible'}`} />
      {pip && stream && !ended && (
        <video ref={pipRef} autoPlay muted playsInline aria-label='웹캠 미리보기' style={camStyle(layout)} className='absolute aspect-video bg-surface-2 object-cover ring-1 ring-brand' />
      )}
      {(!stream || ended) && (
        <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-[url('/brand/bg-speed.jpg')] bg-cover bg-center p-4 text-center">
          <div className='absolute inset-0 bg-bg/60' />
          <span className='relative font-display text-[15px] tracking-widest text-text-muted'>{ended ? '영상이 중단됐어요' : placeholder}</span>
          {children && <div className='relative flex flex-col items-center gap-3'>{children}</div>}
        </div>
      )}
      {overlay}
    </div>
  )
}
