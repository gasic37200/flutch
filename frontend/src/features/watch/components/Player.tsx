import type { ReactNode } from 'react'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import { emoteSrc, type Emote } from '../../../components/ui/mascot.ts'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import { pipPosition } from '../../live/pip.ts'
import { formatDuration, type WatchView } from '../hooks/useWatch.ts'
import type { usePlayer } from '../hooks/usePlayer.ts'

type Props = {
  view: WatchView
  /** 마지막으로 본 송출 소스(영상 중단 문구용) */
  source?: 'screen' | 'camera'
  broadcast: BroadcastDetail | undefined
  player: ReturnType<typeof usePlayer>
  tts: { on: boolean; toggle: () => void }
  onUnmute: () => void
}

// 시안(WatchStates): 질감 배경 위에 어두운 막을 씌우고 가운데에 아이콘·제목·보조 문구. 영상이 없는 상태에서만 쓴다.
function Notice({ icon, title, big, children }: { icon?: ReactNode; title: string; big?: boolean; children?: ReactNode }) {
  return (
    <div role='status' className='absolute inset-0 flex flex-col items-center justify-center gap-2.5 bg-bg/80 px-4 text-center sm:gap-3.5'>
      {icon}
      <h2 className={`font-bold ${big ? 'text-lg sm:text-[22px]' : 'text-base sm:text-xl'}`}>{title}</h2>
      {children}
    </div>
  )
}

const Mascot = ({ emote, className }: { emote: Emote; className: string }) => <img src={emoteSrc(emote)} alt='' className={`w-auto object-contain ${className}`} />
const sub = (text: ReactNode) => <p className='text-sm text-text-muted'>{text}</p>
const dim = (text: ReactNode) => <p className='text-xs text-text-dim'>{text}</p>
const spinner = (cls: string) => <Icon name='spinner' className={`${cls} animate-spin text-brand motion-reduce:animate-none`} />

function Overlay({ view, broadcast, source }: Pick<Props, 'view' | 'broadcast' | 'source'>) {
  switch (view) {
    case 'waiting':
      return (
        <Notice title='곧 방송이 시작돼요' big icon={<Mascot emote='humming' className='h-16 sm:h-24' />}>
          {broadcast && sub(`${broadcast.title} · ${broadcast.owner.nickname}`)}
          {dim('시작되면 새로고침 없이 바로 재생돼요')}
        </Notice>
      )
    case 'connecting':
      return <Notice title='방송에 연결하는 중' icon={spinner('size-10 sm:size-12')} />
    case 'netReconnecting':
      return (
        <Notice title='재연결 중' icon={spinner('size-10 sm:size-12')}>
          {sub('네트워크가 불안정해요. 자동으로 다시 연결하고 있어요.')}
        </Notice>
      )
    case 'streamerReconnecting':
      return (
        <Notice title='스트리머 연결이 끊겼어요' icon={spinner('size-10 sm:size-12')}>
          {sub('재연결을 기다리는 중')}
          {dim('60초 안에 돌아오지 않으면 방송이 종료돼요')}
        </Notice>
      )
    case 'screenPaused':
      return (
        <Notice title={source === 'camera' ? '영상이 잠시 중단됐어요' : '잠시 화면 공유가 중단됐어요'} icon={<Icon name='screenOff' className='size-10 text-text-muted sm:size-11' />}>
          {sub('스트리머 목소리는 계속 들려요')}
        </Notice>
      )
    case 'failed':
      return (
        <Notice title='연결이 끊겼어요' icon={<Icon name='wifiOff' className='size-10 text-text-muted sm:size-11' />}>
          <Button size='lg' onClick={() => window.location.reload()}>
            다시 연결
          </Button>
        </Notice>
      )
    case 'ended': {
      const ms = broadcast?.startedAt && broadcast.endedAt ? Date.parse(broadcast.endedAt) - Date.parse(broadcast.startedAt) : undefined
      const rec = broadcast?.recording
      const ready = rec?.status === 'ready'
      const preparing = broadcast?.record && !ready && rec?.status !== 'failed'
      return (
        <Notice title='방송이 종료되었어요' big icon={<Mascot emote='bye' className='h-14 sm:h-22' />}>
          <p className='flex gap-6 text-sm text-text-muted'>
            {ms !== undefined && (
              <span>
                방송 시간 <b className='font-mono text-text'>{formatDuration(ms)}</b>
              </span>
            )}
            <span>
              최고 시청자 <b className='font-mono text-live'>{broadcast?.peakViewers ?? 0}</b>
            </span>
          </p>
          <div className='flex gap-2.5'>
            {/* 다시보기 자리: 녹화 도메인(T25)이 recording을 채우면 이 분기가 동작한다. */}
            {ready && (
              <Button size='lg' to={`/replay/${rec.id}`}>
                다시보기 보기
              </Button>
            )}
            {preparing && (
              <Button size='lg' variant='neutral' disabled>
                {spinner('size-4')}다시보기 준비 중
              </Button>
            )}
            {broadcast && (
              <Button size='lg' variant={ready ? 'secondary' : 'primary'} to={`/users/${broadcast.owner.id}`}>
                스트리머 페이지
              </Button>
            )}
          </div>
          {preparing && <p className='hidden text-xs text-text-dim sm:block'>준비되면 "다시보기 보기" 버튼으로 바뀌어요</p>}
        </Notice>
      )
    }
    default:
      return null
  }
}

const iconBtn = 'inline-flex size-10 items-center justify-center text-text hover:text-brand focus-visible:outline-2 focus-visible:outline-text'

// 표현만: 상태·트랙·볼륨·PIP 숨김은 useWatch·usePlayer가 준다.
export default function Player({ view, source, broadcast, player: p, tts, onUnmute }: Props) {
  const { videoRef, pipRef, micRef, shareAudioRef, boxRef } = p
  if (view === 'loading') return <Skeleton className='aspect-video w-full' />
  const playing = view === 'playing'
  const live = broadcast?.status === 'LIVE'
  return (
    <div
      ref={boxRef}
      className='relative aspect-video w-full overflow-hidden bg-bg bg-cover bg-center'
      style={playing ? undefined : { backgroundImage: 'url(/brand/bg-speed.jpg)' }}
    >
      <video ref={videoRef} muted playsInline autoPlay aria-label='방송 화면' className={`size-full object-contain ${playing ? '' : 'invisible'}`} />
      {/* PIP: 숨겨도 요소는 남겨 다시 보일 때 트랙이 붙은 채로 있게 한다. 탭하면 숨김(모바일은 이 탭이 숨기기 버튼) */}
      <button
        type='button'
        onClick={p.togglePip}
        aria-label='웹캠 숨기기'
        className={`absolute aspect-video w-[24%] ring-1 ring-brand/60 ${pipPosition(broadcast?.pipCorner ?? 'bottom_right', { top: 'top-12', bottom: 'bottom-14' })} ${playing && p.hasPip && !p.pipHidden ? '' : 'hidden'}`}
      >
        <video ref={pipRef} muted playsInline autoPlay aria-label='웹캠 화면' className='size-full bg-bg object-cover' />
      </button>
      <audio ref={micRef} muted={p.muted} autoPlay />
      <audio ref={shareAudioRef} muted={p.muted} autoPlay />
      <Overlay view={view} broadcast={broadcast} source={source} />
      {live && (
        <div className='absolute top-3 right-3 flex gap-2 sm:top-3.5 sm:right-3.5'>
          {broadcast.superClutchUntil && <Badge tone='superClutch' />}
          <Badge tone='live' />
        </div>
      )}
      {playing && (
        <div className='absolute inset-x-0 bottom-0 flex h-12 items-center gap-1.5 bg-linear-to-t from-bg/90 to-bg/0 px-2.5'>
          {p.muted ? (
            <Button size='sm' variant='neutral' onClick={onUnmute}>
              <Icon name='volume' className='size-4' />
              소리 켜기
            </Button>
          ) : (
            <>
              <Icon name='volume' className='mx-2.5 size-5' />
              <input
                type='range'
                min={0}
                max={1}
                step={0.05}
                value={p.volume}
                onChange={(e) => p.setVolume(Number(e.target.value))}
                aria-label='볼륨'
                className='w-24 accent-text'
              />
            </>
          )}
          <button type='button' onClick={tts.toggle} aria-pressed={tts.on} aria-label={tts.on ? '후원 음성 읽기 끄기' : '후원 음성 읽기 켜기'} className={`${iconBtn} ml-auto`}>
            <Icon name='tts' className={tts.on ? '' : 'text-text-dim'} />
          </button>
          {p.hasPip && (
            <button type='button' onClick={p.togglePip} aria-pressed={p.pipHidden} aria-label={p.pipHidden ? '웹캠 보이기' : '웹캠 숨기기'} className={iconBtn}>
              <Icon name='camera' className={p.pipHidden ? 'text-text-dim' : ''} />
            </button>
          )}
          <button type='button' onClick={p.toggleFullscreen} aria-label={p.fullscreen ? '전체화면 해제' : '전체화면'} className={iconBtn}>
            <Icon name={p.fullscreen ? 'collapse' : 'expand'} />
          </button>
        </div>
      )}
    </div>
  )
}
