import type { ReactNode } from 'react'
import { useSearchParams } from 'react-router'
import Badge from '../components/ui/Badge.tsx'
import Button from '../components/ui/Button.tsx'
import { ChoiceChip } from '../components/ui/Chip.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Icon from '../components/ui/Icon.tsx'
import Panel from '../components/ui/Panel.tsx'
import Skeleton from '../components/ui/Skeleton.tsx'
import AudioControl from '../features/studio/components/AudioControl.tsx'
import MobileControls from '../features/studio/components/MobileControls.tsx'
import MobileLive from '../features/studio/components/MobileLive.tsx'
import StudioAside from '../features/studio/components/StudioAside.tsx'
import StudioDialogs from '../features/studio/components/StudioDialogs.tsx'
import StudioPreview from '../features/studio/components/StudioPreview.tsx'
import { useStudio } from '../features/studio/hooks/useStudio.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

function Bar({ icon = 'info', children, role }: { icon?: 'info' | 'alert'; children: ReactNode; role?: 'alert' | 'status' }) {
  return (
    <p role={role} className='chamfer flex items-start gap-2.5 bg-surface-2 px-3.5 py-2.5 text-sm [--c:6px]'>
      <Icon name={icon} className='mt-0.5 size-[18px] text-brand' />
      <span>{children}</span>
    </p>
  )
}

// 송출 화면: 송출 전 확인(미리보기·소리·환경 안내) → 방송 중 컨트롤. 로직은 useStudio.
// 채팅·후원 알림·방송 정보 수정은 T17·T22·T09 이후 자리(오른쪽 aside는 읽기 전용).
export default function StudioPage() {
  const id = Number(useSearchParams()[0].get('broadcast')) || undefined
  const s = useStudio(id)
  const { data, media, pub, phase } = s

  if (!s.me) return null
  const manage = <Button to='/manage'>내 방송 관리</Button>
  if (!id) return <EmptyState title='방송을 선택해 주세요' description='내 방송 관리에서 방송을 골라 스튜디오로 들어올 수 있어요.' action={manage} />
  const e = s.broadcast.isError ? errorKindOf(s.broadcast.error) : undefined
  // 송출 중 백그라운드 재조회가 실패해도 화면을 갈아엎지 않는다(삭제된 방송 404만 예외).
  if (e && (!data || e.kind === 'notFound')) return <ErrorScreen {...e} onRetry={() => void s.broadcast.refetch()} />
  if (!data) return <Skeleton className='aspect-video w-full max-w-4xl' />
  if (!s.isOwner) return <EmptyState face='grumpy' title='내 방송이 아니에요' description='내가 등록한 방송만 송출할 수 있어요.' action={manage} />
  if (phase === 'ended')
    return (
      <EmptyState
        face='happy'
        title='방송이 종료되었어요'
        description={`방송 시간 ${s.elapsed} · 최고 시청자 ${data.peakViewers}명`}
        action={
          <div className='flex gap-2'>
            {manage}
            <Button to='/' variant='secondary'>
              홈으로
            </Button>
          </div>
        }
      />
    )

  const live = phase === 'live'
  const screen = media.mode === 'screen'
  const mobile = s.env.mobile

  const dialogs = (
    <StudioDialogs
      status={pub.status}
      onConfirmTakeover={() => void pub.confirmTakeover()}
      onCancelTakeover={pub.cancelTakeover}
      onReconnect={() => void pub.start()}
      onAskEnd={s.askEnd}
      endOpen={s.endConfirm}
      ending={s.ending}
      onEnd={() => void s.stop()}
      onCancelEnd={s.cancelEnd}
    />
  )
  if (mobile && live)
    return (
      <>
        <MobileLive s={s} />
        {dialogs}
      </>
    )

  const startButton = (
    <Button size='lg' className={mobile ? 'w-full' : ''} onClick={() => void s.goLive()} loading={pub.status === 'starting'} disabled={!media.videoReady}>
      <span className='size-2.5 rounded-full bg-live' aria-hidden />
      {data.status === 'LIVE' ? '이어서 송출' : '방송 시작'}
    </Button>
  )

  return (
    <div className='space-y-4'>
      <header className='flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-surface-2 pb-4'>
        <span className='font-display font-bold tracking-widest text-text-muted'>STUDIO</span>
        {live ? (
          <>
            <Badge tone='live' />
            <span className='font-mono text-[15px] font-bold' aria-label='경과 시간'>
              {s.elapsed}
            </span>
            <span className='flex items-center gap-1.5 font-mono font-bold text-live' aria-label='시청자 수'>
              <Icon name='user' className='size-4' />
              {data.viewerCount}
            </span>
            {data.record && (
              <Badge tone='recording'>
                <span className='size-2 rounded-full bg-live' aria-hidden />
                녹화 중
              </Badge>
            )}
            <h1 className='sr-only'>{data.title}</h1>
          </>
        ) : (
          <>
            <Badge tone={data.status === 'LIVE' ? 'live' : 'ready'} />
            <h1 className='min-w-0 truncate text-[15px] font-bold'>{data.title}</h1>
          </>
        )}
        <span className='grow' />
        {live ? (
          <Button variant='danger' size='lg' onClick={s.askEnd}>
            방송 종료
          </Button>
        ) : (
          !mobile && startButton
        )}
      </header>

      <div className='flex flex-col gap-5 xl:flex-row'>
        <div className='min-w-0 grow space-y-3.5'>
          {live && <Bar role='alert'>방송 시작 3시간이 되면 자동으로 종료돼요.</Bar>}
          {s.env.notices.map((n) => (
            <Bar key={n} role='status'>
              {n}
            </Bar>
          ))}

          <StudioPreview stream={media.stream} ended={media.videoEnded} tall={mobile} placeholder={live ? '송출 미리보기' : '미리보기 · 아직 송출되지 않아요'}>
            {media.videoEnded ? (
              <Button onClick={() => void s.reshare()}>{screen ? '다시 공유하기' : '카메라 다시 켜기'}</Button>
            ) : (
              <>
                <Button variant='secondary' size='lg' onClick={() => void s.startPreview()}>
                  {screen ? '공유할 화면 선택' : '카메라 켜고 미리보기'}
                </Button>
                {screen && <span className='text-xs text-text-dim'>창을 고를 때 &quot;탭 오디오 공유&quot; 또는 &quot;시스템 오디오 공유&quot;를 켜 주세요</span>}
              </>
            )}
          </StudioPreview>

          {(pub.error || media.error) && <Bar role='alert' icon='alert'>{pub.error ?? media.error}</Bar>}
          {live && !s.connected && (
            <p role='status' className='text-xs text-text-dim'>
              실시간 시청자 정보를 다시 연결하는 중이에요.
            </p>
          )}

          {mobile ? (
            <MobileControls s={s} />
          ) : (
            <div className='grid gap-3.5 md:grid-cols-2'>
              <Panel innerClassName='flex flex-col gap-3.5 p-4'>
                <h2 className='text-[15px] font-bold'>영상</h2>
                <div className='flex gap-2' role='group' aria-label='송출 모드'>
                  <ChoiceChip selected={screen} onClick={() => void s.chooseMode('screen')}>
                    화면 공유
                  </ChoiceChip>
                  <ChoiceChip selected={!screen} onClick={() => void s.chooseMode('camera')}>
                    웹캠만
                  </ChoiceChip>
                </div>
                {!s.env.canScreen && <p className='text-xs text-text-dim'>이 환경에서는 화면 공유를 쓸 수 없어 카메라 모드만 쓸 수 있어요.</p>}
                {media.videoReady && screen && (
                  <Button variant='neutral' onClick={() => void s.reshare()}>
                    공유할 화면 바꾸기
                  </Button>
                )}
              </Panel>

              <Panel innerClassName='flex flex-col gap-3.5 p-4'>
                <h2 className='text-[15px] font-bold'>소리</h2>
                <AudioControl channel='mic' label='마이크' media={media} available={media.micReady} emptyHint='마이크가 꺼져 있어요. 시청자에게 목소리가 가지 않아요.' />
                {!media.micReady && media.videoReady && (
                  <Button variant='secondary' size='sm' onClick={() => void media.captureMic()}>
                    마이크 켜기
                  </Button>
                )}
                {screen && (
                  <AudioControl
                    channel='sys'
                    label='화면 소리'
                    sub={s.env.systemAudio ? '(시스템 소리)' : s.env.tabAudio ? '(탭 소리)' : undefined}
                    media={media}
                    available={media.hasScreenAudio}
                    emptyHint={media.videoReady ? '공유한 화면에 소리가 없어요. 화면 선택 창에서 소리 공유를 체크했는지 확인해 주세요.' : '화면을 공유하면 소리를 조절할 수 있어요.'}
                  />
                )}
                {screen && <span className='text-xs text-text-dim'>같은 PC에서 내 방송을 소리 켠 채로 보면 소리가 되돌아와요.</span>}
              </Panel>
            </div>
          )}

          {mobile && !live && startButton}
        </div>

        <aside className='w-full shrink-0 space-y-3.5 xl:w-[340px]'>
          <StudioAside s={s} live={live} />
        </aside>
      </div>
      {dialogs}
    </div>
  )
}
