import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import type { useStudio } from '../hooks/useStudio.ts'
import LevelMeter from './LevelMeter.tsx'

// 모바일 송출 전 컨트롤: 카메라 고정, 큰 터치 버튼. 화면 공유·볼륨 슬라이더는 두지 않는다. 방송 중 화면은 MobileLive.
export default function MobileControls({ s }: { s: ReturnType<typeof useStudio> }) {
  const { media, env } = s
  const micMuted = media.muted.mic
  return (
    <div className='space-y-3'>
      <div className='flex flex-wrap gap-2'>
        {media.videoReady && env.canFlip && (
          <Button size='lg' variant='neutral' className='grow' onClick={() => void media.flipCamera()}>
            <Icon name='flip' />
            {media.facing === 'user' ? '후면 카메라로' : '전면 카메라로'}
          </Button>
        )}
        {media.videoReady && !media.micReady && (
          <Button size='lg' variant='secondary' className='grow' onClick={() => void media.captureMic()}>
            마이크 켜기
          </Button>
        )}
        {media.micReady && (
          <Button size='lg' variant={micMuted ? 'outline' : 'neutral'} className='grow' aria-pressed={micMuted} onClick={() => media.toggleMute('mic')}>
            <Icon name={micMuted ? 'micOff' : 'mic'} />
            {micMuted ? '음소거 해제' : '음소거'}
          </Button>
        )}
      </div>
      {media.micReady ? (
        <LevelMeter analyser={media.analysers.mic} label='마이크' />
      ) : (
        media.videoReady && <p className='text-xs text-text-dim'>마이크가 꺼져 있어요. 시청자에게 목소리가 가지 않아요.</p>
      )}
    </div>
  )
}
