import Icon from '../../../components/ui/Icon.tsx'
import Switch from '../../../components/ui/Switch.tsx'
import type { Channel, useStudioMedia } from '../hooks/useStudioMedia.ts'
import LevelMeter from './LevelMeter.tsx'

type Props = {
  channel: Channel
  label: string
  media: ReturnType<typeof useStudioMedia>
  /** 소리가 없는 상태의 안내(예: "공유한 화면에 소리가 없어요") */
  emptyHint?: string
  available: boolean
}

// 볼륨 슬라이더(0~150%) + 음소거 + 레벨 미터. 볼륨은 Web Audio GainNode로 송출 트랙에 반영된다.
export default function AudioControl({ channel, label, media, emptyHint, available }: Props) {
  const volume = media.volume[channel]
  const muted = media.muted[channel]
  return (
    <div className='space-y-2'>
      <div className='flex items-center justify-between gap-3'>
        <span className='flex items-center gap-2 text-sm font-semibold'>
          <Icon name='pulse' className='size-4 text-brand' />
          {label}
        </span>
        <span className='flex items-center gap-2 text-xs text-text-muted'>
          음소거
          <Switch label={`${label} 음소거`} checked={muted} onChange={() => media.toggleMute(channel)} disabled={!available} />
        </span>
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
