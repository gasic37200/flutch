import { useSearchParams } from 'react-router'
import Badge from '../components/ui/Badge.tsx'
import Button from '../components/ui/Button.tsx'
import { ChoiceChip, Chip } from '../components/ui/Chip.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Icon from '../components/ui/Icon.tsx'
import Panel from '../components/ui/Panel.tsx'
import Skeleton from '../components/ui/Skeleton.tsx'
import { CATEGORY_LABEL } from '../features/broadcast/categories.ts'
import AudioControl from '../features/studio/components/AudioControl.tsx'
import MobileControls from '../features/studio/components/MobileControls.tsx'
import StudioDialogs from '../features/studio/components/StudioDialogs.tsx'
import StudioPreview from '../features/studio/components/StudioPreview.tsx'
import { useStudio } from '../features/studio/hooks/useStudio.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

// PC 송출 화면: 송출 전 확인(미리보기·마이크 레벨·환경 안내) → 송출 중 컨트롤. 로직은 useStudio.
// 채팅·방송 정보 수정 패널은 T17 자리(오른쪽 aside).
export default function StudioPage() {
  const id = Number(useSearchParams()[0].get('broadcast')) || undefined
  const s = useStudio(id)
  const { data, media, pub, phase } = s

  if (!s.me) return null
  const manage = <Button to='/manage'>내 방송 관리</Button>
  if (!id) return <EmptyState title='방송을 선택해 주세요' description='내 방송 관리에서 방송을 골라 스튜디오로 들어올 수 있어요.' action={manage} />
  if (s.broadcast.isError) return <ErrorScreen {...errorKindOf(s.broadcast.error)} onRetry={() => void s.broadcast.refetch()} />
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
  const bigSize = mobile ? 'lg' : 'md'

  return (
    <div className='flex flex-col gap-4 xl:flex-row'>
      <div className='min-w-0 grow space-y-4'>
        <header className='flex flex-wrap items-center gap-3'>
          <Badge tone={live || data.status === 'LIVE' ? 'live' : 'ready'} />
          <h1 className='min-w-0 grow truncate font-display text-xl font-bold tracking-wide'>{data.title}</h1>
          {live && (
            <>
              <span className='font-mono text-sm' aria-label='경과 시간'>
                {s.elapsed}
              </span>
              <span className='flex items-center gap-1 font-mono text-sm text-live' aria-label='시청자 수'>
                <Icon name='eye' className='size-4' />
                {data.viewerCount}
              </span>
              <Button variant='danger' size={bigSize} onClick={s.askEnd}>
                방송 종료
              </Button>
            </>
          )}
        </header>

        <Panel innerClassName='p-3'>
          <StudioPreview stream={media.stream} live={live} ended={media.videoEnded} tall={mobile}>
            {media.videoEnded && (
              <Button size={bigSize} onClick={() => void s.reshare()}>
                {screen ? '다시 공유하기' : '카메라 다시 켜기'}
              </Button>
            )}
          </StudioPreview>
        </Panel>

        {(pub.error || media.error) && (
          <p role='alert' className='flex items-start gap-2 bg-surface-2 p-3 text-sm'>
            <Icon name='alert' className='mt-0.5 size-4 shrink-0 text-brand' />
            {pub.error ?? media.error}
          </p>
        )}
        {live && !s.connected && (
          <p role='status' className='text-xs text-text-dim'>
            실시간 시청자 정보를 다시 연결하는 중이에요.
          </p>
        )}

        {mobile ? (
          <MobileControls s={s} live={live} />
        ) : (
        <div className='grid gap-4 md:grid-cols-2'>
          <Panel innerClassName='space-y-3 p-4'>
            <h2 className='font-display text-sm font-bold tracking-wide'>영상</h2>
            <div className='flex flex-wrap gap-2' role='group' aria-label='송출 모드'>
              <ChoiceChip selected={screen} onClick={() => void s.chooseMode('screen')}>
                화면 공유
              </ChoiceChip>
              <ChoiceChip selected={!screen} onClick={() => void s.chooseMode('camera')}>
                웹캠 모드
              </ChoiceChip>
            </div>
            {!s.env.canScreen && <p className='text-xs text-text-dim'>이 환경에서는 화면 공유를 쓸 수 없어 카메라 모드만 쓸 수 있어요.</p>}
            <div className='flex flex-wrap gap-2'>
              {!media.videoReady && !media.videoEnded && (
                <Button variant={live ? 'primary' : 'secondary'} onClick={() => void s.startPreview()}>
                  {screen ? '화면 선택하고 미리보기' : '카메라 켜고 미리보기'}
                </Button>
              )}
              {media.videoReady && screen && (
                <Button variant='secondary' size='sm' onClick={() => void s.reshare()}>
                  화면 소스 변경
                </Button>
              )}
              {!media.micReady && media.videoReady && (
                <Button variant='secondary' size='sm' onClick={() => void media.captureMic()}>
                  마이크 켜기
                </Button>
              )}
            </div>
            {screen && !live && (
              <p className='text-xs text-text-muted'>
                화면 선택 창 아래의 <strong className='text-text'>시스템 소리 공유</strong> 또는 <strong className='text-text'>탭 소리 공유</strong>를 체크해야 시청자에게 소리가 가요.
              </p>
            )}
          </Panel>

          <Panel innerClassName='space-y-4 p-4'>
            <h2 className='font-display text-sm font-bold tracking-wide'>소리</h2>
            <AudioControl channel='mic' label='마이크' media={media} available={media.micReady} emptyHint='마이크가 꺼져 있어요. 시청자에게 목소리가 가지 않아요.' />
            {screen && (
              <AudioControl
                channel='sys'
                label='화면 소리'
                media={media}
                available={media.hasScreenAudio}
                emptyHint={media.videoReady ? '공유한 화면에 소리가 없어요. 화면 선택 창에서 소리 공유를 체크했는지 확인해 주세요.' : '화면을 공유하면 소리를 조절할 수 있어요.'}
              />
            )}
          </Panel>
        </div>
        )}

        {s.env.notices.length > 0 && (
          <ul className='space-y-2' aria-label='지원 환경 안내'>
            {s.env.notices.map((n) => (
              <li key={n} className='flex items-start gap-2 bg-surface-2 p-3 text-sm'>
                <Icon name='info' className='mt-0.5 size-4 shrink-0 text-brand' />
                {n}
              </li>
            ))}
          </ul>
        )}

        {!live && (
          <div className='flex flex-wrap items-center gap-3'>
            <Button size={bigSize} className={mobile ? 'w-full' : ''} onClick={() => void s.goLive()} loading={pub.status === 'starting'} disabled={!media.videoReady}>
              {data.status === 'LIVE' ? '이어서 송출' : '방송 시작'}
            </Button>
            {!media.videoReady && <span className='text-xs text-text-dim'>미리보기를 켜면 방송을 시작할 수 있어요.</span>}
          </div>
        )}
      </div>

      <aside className='w-full shrink-0 space-y-3 xl:w-80'>
        <Panel innerClassName='space-y-2 p-4'>
          <h2 className='font-display text-sm font-bold tracking-wide'>방송 정보</h2>
          <p className='text-sm'>{CATEGORY_LABEL[data.category]}</p>
          {data.tags.length > 0 && (
            <ul className='flex flex-wrap gap-1' aria-label='태그'>
              {data.tags.map((t) => (
                <li key={t}>
                  <Chip>#{t}</Chip>
                </li>
              ))}
            </ul>
          )}
          <p className='text-xs text-text-dim'>{data.record ? '방송이 끝나면 다시보기로 남아요.' : '녹화하지 않는 방송이에요.'}</p>
        </Panel>
      </aside>

      <StudioDialogs
        status={pub.status}
        onConfirmTakeover={() => void pub.confirmTakeover()}
        onCancelTakeover={pub.cancelTakeover}
        onReconnect={() => void pub.start()}
        endOpen={s.endConfirm}
        ending={s.ending}
        onEnd={() => void s.stop()}
        onCancelEnd={s.cancelEnd}
      />
    </div>
  )
}
