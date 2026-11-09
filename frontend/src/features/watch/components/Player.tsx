import type { ReactNode } from 'react'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import { formatDuration, type WatchView } from '../hooks/useWatch.ts'
import type { usePlayer } from '../hooks/usePlayer.ts'

type Props = {
  view: WatchView
  broadcast: BroadcastDetail | undefined
  player: ReturnType<typeof usePlayer>
  onUnmute: () => void
}

function Notice({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <div role='status' className='absolute inset-0 flex flex-col items-center justify-center gap-3 bg-bg/85 px-4 text-center'>
      <h2 className='text-lg font-bold'>{title}</h2>
      {children}
    </div>
  )
}

const hint = (text: string) => <p className='max-w-sm text-sm text-text-muted'>{text}</p>

function Overlay({ view, broadcast }: Pick<Props, 'view' | 'broadcast'>) {
  switch (view) {
    case 'waiting':
      return <Notice title='방송 시작을 기다리는 중'>{hint('곧 방송이 시작돼요. 시작되면 자동으로 재생돼요.')}</Notice>
    case 'connecting':
      return <Notice title='방송에 연결하는 중' />
    case 'netReconnecting':
      return <Notice title='재연결 중'>{hint('네트워크가 불안정해요. 자동으로 다시 연결하고 있어요.')}</Notice>
    case 'streamerReconnecting':
      return <Notice title='스트리머 연결을 기다리는 중'>{hint('스트리머가 돌아오면 이어서 재생돼요.')}</Notice>
    case 'screenPaused':
      return <Notice title='잠시 화면 공유가 중단됐어요'>{hint('스트리머가 다시 공유하면 이어서 보여요.')}</Notice>
    case 'failed':
      return (
        <Notice title='연결이 끊겼어요'>
          <Button onClick={() => window.location.reload()}>다시 연결</Button>
        </Notice>
      )
    case 'ended': {
      const ms = broadcast?.startedAt && broadcast.endedAt ? Date.parse(broadcast.endedAt) - Date.parse(broadcast.startedAt) : undefined
      const rec = broadcast?.recording
      return (
        <Notice title='방송이 종료되었어요'>
          <p className='text-sm text-text-muted'>
            최고 시청자 <span className='font-mono text-text'>{broadcast?.peakViewers ?? 0}</span>명
            {ms !== undefined && (
              <>
                {' · '}방송 시간 <span className='font-mono text-text'>{formatDuration(ms)}</span>
              </>
            )}
          </p>
          {/* 다시보기 자리: 녹화 도메인(T25)이 recording을 채우면 이 분기가 동작한다. */}
          {rec?.status === 'ready' ? (
            <Button to={`/replay/${rec.id}`}>다시보기 보기</Button>
          ) : (
            broadcast?.record && rec?.status !== 'failed' && hint('다시보기 준비 중이에요.')
          )}
          <Button variant='secondary' to='/'>
            홈으로
          </Button>
        </Notice>
      )
    }
    default:
      return null
  }
}

const iconBtn = 'inline-flex size-9 items-center justify-center rounded-xs text-text hover:text-brand focus-visible:outline-2 focus-visible:outline-text'

// 표현만: 상태·트랙·볼륨은 useWatch·usePlayer가 준다. 웹캠 PIP 겹치기는 T09b가 이 영역(relative 박스)에 얹는다.
export default function Player({ view, broadcast, player: p, onUnmute }: Props) {
  const { videoRef, micRef, shareAudioRef, boxRef } = p
  if (view === 'loading') return <Skeleton className='aspect-video w-full' />
  const playing = view === 'playing'
  return (
    <div ref={boxRef} className='relative aspect-video w-full overflow-hidden rounded-sm bg-bg'>
      <video ref={videoRef} muted playsInline autoPlay aria-label='방송 화면' className={`size-full object-contain ${playing ? '' : 'invisible'}`} />
      <audio ref={micRef} muted={p.muted} autoPlay />
      <audio ref={shareAudioRef} muted={p.muted} autoPlay />
      <Overlay view={view} broadcast={broadcast} />
      {playing && (
        <div className='absolute inset-x-0 bottom-0 flex items-center gap-2 bg-bg/70 px-3 py-2'>
          {p.muted ? (
            <Button size='sm' onClick={onUnmute}>
              <Icon name='play' className='size-3.5' />
              소리 켜기
            </Button>
          ) : (
            <label className='flex items-center gap-2 text-xs text-text-muted'>
              볼륨
              <input
                type='range'
                min={0}
                max={1}
                step={0.05}
                value={p.volume}
                onChange={(e) => p.setVolume(Number(e.target.value))}
                aria-label='볼륨'
                className='w-24 accent-brand'
              />
            </label>
          )}
          <button type='button' onClick={p.toggleFullscreen} aria-label={p.fullscreen ? '전체화면 해제' : '전체화면'} className={`${iconBtn} ml-auto`}>
            <Icon name={p.fullscreen ? 'collapse' : 'expand'} />
          </button>
        </div>
      )}
    </div>
  )
}
