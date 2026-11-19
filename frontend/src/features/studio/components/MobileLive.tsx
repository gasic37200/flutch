import { useState } from 'react'
import DonationAlert from '../../donation/components/DonationAlert.tsx'
import ChatOverlay from '../../chat/components/ChatOverlay.tsx'
import { DEFAULT_LAYOUT } from '../../live/overlayLayout.ts'
import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Modal from '../../../components/ui/Modal.tsx'
import type { useStudio } from '../hooks/useStudio.ts'
import StudioInfoForm from './StudioInfoForm.tsx'
import StudioGauge from './StudioGauge.tsx'
import LevelMeter from './LevelMeter.tsx'
import StudioPreview from './StudioPreview.tsx'

const round = 'flex size-14 items-center justify-center rounded-full bg-surface-2/85 text-text'

// 모바일 방송 중: 카메라가 전체 화면, 그 위에 상태 줄·원형 버튼·마이크 레벨을 얹는다. 채팅은 읽기 전용 오버레이, 후원 알림은 그 위(넘기기 가능).
export default function MobileLive({ s }: { s: ReturnType<typeof useStudio> }) {
  const { media, env, data } = s
  const [editing, setEditing] = useState(false)
  if (!data) return null
  const error = s.pub.error ?? media.error
  return (
    <div className='@container fixed inset-0 z-50 overflow-hidden bg-bg text-text'>
      <StudioPreview fill stream={media.stream} ended={media.videoEnded} placeholder='카메라 미리보기'>
        {media.videoEnded && <Button onClick={() => void s.reshare()}>카메라 다시 켜기</Button>}
      </StudioPreview>
      <div className='pointer-events-none absolute inset-0 bg-gradient-to-b from-bg/55 via-bg/10 via-25% to-bg/85' />

      <div className='absolute inset-x-4 top-4 flex items-center gap-2.5'>
        <Badge tone='live' />
        <span className='font-mono text-sm font-bold' aria-label='경과 시간'>
          {s.elapsed}
        </span>
        <span className='flex items-center gap-1 font-mono text-[13px] font-bold text-live' aria-label='시청자 수'>
          <Icon name='eye' className='size-3.5' />
          {data.viewerCount}
        </span>
        <span className='grow' />
        <Button variant='danger' onClick={s.askEnd}>
          종료
        </Button>
      </div>
      <div className='absolute inset-x-4 top-16 flex items-center gap-3'>
        <h1 className='min-w-0 grow truncate text-[13px] text-text-muted'>{data.title}</h1>
        <StudioGauge s={s} className='w-40 shrink-0 rounded-sm bg-bg/60 px-2 py-1' />
      </div>

      {error && (
        <p role='alert' className='chamfer absolute inset-x-4 top-24 flex items-start gap-2 bg-surface-2/90 px-3 py-2 text-sm [--c:6px]'>
          <Icon name='alert' className='mt-0.5 size-4 text-brand' />
          {error}
        </p>
      )}

      {/* 위쪽 상태 줄을 피해 영상 높이 14%(약 top-28)에 둔다. 크기는 화면 폭 비례 */}
      <DonationAlert alerts={s.alerts} owner layout={{ ...DEFAULT_LAYOUT, alert: { x: 50, y: 14 } }} />
      <ChatOverlay items={s.chat.items} className='absolute right-20 bottom-36 left-4 max-h-44 text-[13px] leading-snug' />

      {media.micReady && (
        <div className='absolute right-4 bottom-30 flex flex-col items-center gap-1.5'>
          <LevelMeter vertical analyser={media.analysers.mic} label='마이크' />
          <span className='text-xs text-text-muted'>마이크</span>
        </div>
      )}

      <div className='absolute right-20 bottom-24 left-4 text-[11px] text-text-dim [text-shadow:0_1px_2px_var(--color-bg)]' role='status'>
        {s.wakeLockMissing ? '화면이 꺼지지 않게 설정해 주세요. 이 브라우저에서는 자동으로 막을 수 없어요.' : '화면 꺼짐 방지 켜짐'}
        <br />
        다른 앱으로 이동하면 카메라가 멈출 수 있어요.
      </div>

      <div className='absolute inset-x-0 bottom-7 flex justify-center gap-5'>
        {env.canFlip && (
          <button type='button' aria-label='카메라 전환' onClick={() => void s.flipCamera()} className={round}>
            <Icon name='flip' className='size-6' />
          </button>
        )}
        {media.micReady ? (
          <button
            type='button'
            aria-label='마이크 음소거'
            aria-pressed={media.muted.mic}
            onClick={() => media.toggleMute('mic')}
            className={media.muted.mic ? 'flex size-14 items-center justify-center rounded-full bg-text text-bg' : round}
          >
            <Icon name={media.muted.mic ? 'micOff' : 'mic'} className='size-6' />
          </button>
        ) : (
          <button type='button' aria-label='마이크 켜기' onClick={() => void media.captureMic()} className={round}>
            <Icon name='micOff' className='size-6' />
          </button>
        )}
        <button type='button' aria-label='방송 정보 수정' onClick={() => setEditing(true)} className={round}>
          {/* 공용 Icon에 연필이 없어 시안 경로를 그대로 둔다 */}
          <svg viewBox='0 0 24 24' fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round' strokeLinejoin='round' className='size-6' aria-hidden>
            <path d='M4 20h4L19 9l-4-4L4 16z' />
          </svg>
        </button>
      </div>
      <Modal open={editing} onClose={() => setEditing(false)} title='방송 정보 수정'>
        <StudioInfoForm broadcast={data} onSaved={() => setEditing(false)} />
      </Modal>
    </div>
  )
}
