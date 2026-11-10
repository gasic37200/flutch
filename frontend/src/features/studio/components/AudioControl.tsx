import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import type { Channel, useStudioMedia } from '../hooks/useStudioMedia.ts'
import LevelMeter from './LevelMeter.tsx'

type Props = {
  channel: Channel
  label: string
  /** 라벨 옆 보조 설명(예: 시스템 소리) */
  sub?: string
  media: ReturnType<typeof useStudioMedia>
  /** 소리가 없는 상태의 안내(예: "공유한 화면에 소리가 없어요") */
  emptyHint?: string
  available: boolean
}

// 볼륨 슬라이더(0~150%) + 음소거 버튼 + 레벨 미터. 볼륨은 Web Audio GainNode로 송출 트랙에 반영된다.
export default function AudioControl({ channel, label, sub, media, emptyHint, available }: Props) {
  const volume = media.volume[channel]
  const muted = media.muted[channel]
  const icon = channel === 'mic' ? (muted ? 'micOff' : 'mic') : muted ? 'volumeOff' : 'volume'
  return (
    <div className='space-y-1.5'>
      <div className='flex items-center gap-2'>
        <span className='grow text-sm'>
          {label} {sub && <span className='text-xs text-text-dim'>{sub}</span>}
        </span>
        <Button variant='neutral' size='sm' className='[&_span]:px-3' aria-label={`${label} 음소거`} aria-pressed={muted} disabled={!available} onClick={() => media.toggleMute(channel)}>
          <Icon name={icon} className='size-4' />
        </Button>
      </div>
      <label className='flex items-center gap-3 text-xs text-text-muted'>
        <span className='sr-only'>{label} 볼륨</span>
        <input
          type='range'
          min={0}
          max={150}
          step={5}
          value={volume}
          disabled={!available}
          onChange={(e) => media.changeVolume(channel, Number(e.target.value))}
          aria-valuetext={`${volume}%`}
          className='h-2 grow accent-brand disabled:opacity-50'
        />
        <span className='w-10 text-right font-mono'>{volume}%</span>
      </label>
      <LevelMeter analyser={available ? media.analysers[channel] : null} label={label} />
      {!available && emptyHint && <p className='text-xs text-text-dim'>{emptyHint}</p>}
    </div>
  )
}
