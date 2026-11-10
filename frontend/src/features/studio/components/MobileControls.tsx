import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import type { useStudio } from '../hooks/useStudio.ts'
import LevelMeter from './LevelMeter.tsx'

// 모바일 송출 컨트롤: 카메라 고정, 큰 터치 버튼(48px). 화면 공유·볼륨 슬라이더는 두지 않는다.
export default function MobileControls({ s, live }: { s: ReturnType<typeof useStudio>; live: boolean }) {
  const { media, env } = s
  const micMuted = media.muted.mic
  return (
    <div className='space-y-3'>
      <div className='flex flex-wrap gap-2'>
        {!media.videoReady && !media.videoEnded && (
          <Button size='lg' className='w-full' onClick={() => void s.startPreview()}>
            카메라 켜고 미리보기
          </Button>
        )}
        {media.videoReady && env.canFlip && (
          <Button size='lg' variant='secondary' className='grow' onClick={() => void media.flipCamera()}>
            <Icon name='camera' />
            {media.facing === 'user' ? '후면 카메라로' : '전면 카메라로'}
          </Button>
        )}
        {media.videoReady && !media.micReady && (
          <Button size='lg' variant='secondary' className='grow' onClick={() => void media.captureMic()}>
            마이크 켜기
          </Button>
        )}
        {media.micReady && (
          <Button size='lg' variant={micMuted ? 'outline' : 'secondary'} className='grow' aria-pressed={micMuted} onClick={() => media.toggleMute('mic')}>
            {micMuted ? '음소거 해제' : '음소거'}
          </Button>
        )}
      </div>
      {media.micReady ? (
        <LevelMeter analyser={media.analysers.mic} label='마이크' />
      ) : (
        media.videoReady && <p className='text-xs text-text-dim'>마이크가 꺼져 있어요. 시청자에게 목소리가 가지 않아요.</p>
      )}
      {live && (
        <ul className='space-y-2 text-sm' aria-label='송출 중 안내'>
          <li className='flex items-start gap-2 bg-surface-2 p-3'>
            <Icon name='info' className='mt-0.5 size-4 shrink-0 text-brand' />
            다른 앱으로 이동하거나 화면을 끄면 카메라가 멈출 수 있어요. 방송하는 동안 이 화면을 켜 두세요.
          </li>
          {s.wakeLockMissing && (
            <li className='flex items-start gap-2 bg-surface-2 p-3'>
              <Icon name='alert' className='mt-0.5 size-4 shrink-0 text-brand' />
              화면이 꺼지지 않게 설정해 주세요. 이 브라우저에서는 자동으로 막을 수 없어요.
            </li>
          )}
        </ul>
      )}
    </div>
  )
}
