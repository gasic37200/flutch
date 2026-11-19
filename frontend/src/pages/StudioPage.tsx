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
import PipControl, { LayoutEditRow } from '../features/studio/components/PipControl.tsx'
import { LayoutLayer, LayoutToolbar } from '../features/studio/components/LayoutEditor.tsx'
import StudioAside from '../features/studio/components/StudioAside.tsx'
import StudioDialogs from '../features/studio/components/StudioDialogs.tsx'
import DonationAlert from '../features/donation/components/DonationAlert.tsx'
import StudioGauge from '../features/studio/components/StudioGauge.tsx'
import StudioPreview from '../features/studio/components/StudioPreview.tsx'
import StudioTopBar from '../features/studio/components/StudioTopBar.tsx'
import { useStudio } from '../features/studio/hooks/useStudio.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

function Bar({ icon = 'info', children, role }: { icon?: 'info' | 'alert'; children: ReactNode; role?: 'alert' | 'status' }) {
  return (
    <p role={role} className='chamfer flex items-center gap-2.5 bg-surface-2 px-3.5 py-2.5 text-sm [--c:6px]'>
      <Icon name={icon} className='size-[18px] shrink-0 text-brand' />
      <span>{children}</span>
    </p>
  )
}

// 시안의 카드 제목 줄 + 안쪽 여백
function Card({ title, children }: { title: string; children: ReactNode }) {
  return (
    <Panel innerClassName='flex flex-col gap-3.5 p-4'>
      <h2 className='text-[15px] font-bold'>{title}</h2>
      {children}
    </Panel>
  )
}

const MAX_LIVE_SEC = 3 * 3600
// 경과 시간(H:MM:SS)으로 자동 종료까지 남은 시간을 구한다
function remaining(elapsed: string) {
  const [h, m, sec] = elapsed.split(':').map(Number)
  const left = Math.max(0, MAX_LIVE_SEC - (h * 3600 + m * 60 + sec))
  return `${Math.floor(left / 3600)}:${String(Math.floor(left / 60) % 60).padStart(2, '0')}:${String(left % 60).padStart(2, '0')}`
}

// 송출 화면: 송출 전 확인(미리보기·소리·환경 안내) → 방송 중 컨트롤. 로직은 useStudio. 상단바는 스튜디오 전용(시안).
export default function StudioPage() {
  const id = Number(useSearchParams()[0].get('broadcast')) || undefined
  const s = useStudio(id)
  const { data, media, pub, phase } = s

  if (!s.me) return null
  const manage = <Button to='/manage'>내 방송 관리</Button>
  // 상태 화면(안내·오류·종료)도 같은 상단바 아래에 둔다
  const frame = (body: ReactNode) => (
    <>
      <StudioTopBar />
      <div className='mx-auto max-w-[1920px] p-4 md:px-6 md:py-5'>{body}</div>
    </>
  )
  if (!id) return frame(<EmptyState title='방송을 선택해 주세요' description='내 방송 관리에서 방송을 골라 스튜디오로 들어올 수 있어요.' action={manage} />)
  const e = s.broadcast.isError ? errorKindOf(s.broadcast.error) : undefined
  // 송출 중 백그라운드 재조회가 실패해도 화면을 갈아엎지 않는다(삭제된 방송 404만 예외).
  if (e && (!data || e.kind === 'notFound')) return frame(<ErrorScreen {...e} onRetry={() => void s.broadcast.refetch()} />)
  if (!data) return frame(<Skeleton className='aspect-video w-full max-w-4xl' />)
  if (!s.isOwner) return frame(<EmptyState emote='hiding' title='내 방송이 아니에요' description='내가 등록한 방송만 송출할 수 있어요.' action={manage} />)
  if (phase === 'ended')
    return frame(
      // 상단바(4rem+1px)와 바깥 여백을 뺀 높이의 가운데
      <div className='mx-auto flex min-h-[calc(100dvh-6.5rem)] max-w-xl items-center justify-center md:min-h-[calc(100dvh-6.75rem)]'>
        <EmptyState
          emote='bye'
          title='방송이 종료되었어요'
          description={`방송 시간 ${s.elapsed} · 최고 시청자 ${data.peakViewers}명${data.record ? ' · 다시보기는 녹화 처리가 끝나면 스트리머 페이지에서 볼 수 있어요.' : ''}`}
          action={
            <div className='flex flex-wrap justify-center gap-2'>
              {data.record && <Button to={`/users/${data.owner.id}`}>다시보기 보러 가기</Button>}
              <Button to='/manage' variant={data.record ? 'secondary' : 'primary'}>
                내 방송 관리
              </Button>
              <Button to='/' variant='neutral'>
                홈으로
              </Button>
            </div>
          }
        />
      </div>,
    )

  const live = phase === 'live'
  const screen = media.mode === 'screen'
  const mobile = s.env.mobile
  // 새로고침·재진입: 서버는 LIVE인데 이 화면은 송출을 아직 잇지 않은 상태. 소스를 켜지 않아도 종료할 수 있어야 한다.
  const resumable = !live && data.status === 'LIVE'
  const startHint = media.videoReady ? undefined : screen ? '공유할 화면을 고르면 시작할 수 있어요' : '카메라를 켜면 시작할 수 있어요'

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
    <Button size='lg' className={mobile ? 'w-full' : ''} onClick={() => void s.goLive()} loading={pub.status === 'starting'} disabled={!media.videoReady} aria-describedby='studio-start-hint'>
      <span className='size-2.5 rounded-full bg-live' aria-hidden />
      {data.status === 'LIVE' ? '이어서 송출' : '방송 시작'}
    </Button>
  )

  // 화면 공유 ↔ 웹캠만 전환(데스크톱). 방송 중엔 영상 카드, 송출 전엔 미리보기 위에 둔다.
  const modeChips = (
    <div className='flex gap-2' role='group' aria-label='송출 모드'>
      <ChoiceChip selected={screen} onClick={() => void s.chooseMode('screen')}>
        화면 공유
      </ChoiceChip>
      <ChoiceChip selected={!screen} onClick={() => void s.chooseMode('camera')}>
        웹캠만
      </ChoiceChip>
    </div>
  )
  const editing = s.layout.editing
  const layoutRow = <LayoutEditRow editing={editing} onToggle={editing ? s.layout.cancel : s.layout.start} />
  const pip = screen && <PipControl on={s.pipOn} onToggle={(on) => void s.togglePip(on)} />
  const micHeader = (
    <span className='flex grow items-center gap-2'>
      <b className='text-[15px]'>마이크</b>
      {media.micReady && <span className='ml-auto text-xs text-brand'>● 소리가 들어오고 있어요</span>}
    </span>
  )
  const sysEmpty = media.videoReady ? '공유한 화면에 소리가 없어요. 화면 선택 창에서 소리 공유를 체크했는지 확인해 주세요.' : '화면을 공유하면 소리를 조절할 수 있어요.'

  return (
    <>
      <StudioTopBar>
        {live ? (
          <>
            <Badge tone='live' />
            <span className='font-mono text-[15px] font-bold' aria-label='경과 시간'>
              {s.elapsed}
            </span>
            <span className='flex items-center gap-1.5 font-mono font-bold text-live' aria-label='시청자 수'>
              <Icon name='user' className='size-4' />
              {data.viewerCount.toLocaleString('ko-KR')}
            </span>
            {data.record && (
              <span className='flex items-center gap-1.5 text-[13px] text-text-muted'>
                <span className='size-2 rounded-full bg-live' aria-hidden />
                녹화 중
              </span>
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
        {!live && !mobile && (
          <>
            <span id='studio-start-hint' className='sr-only'>
              {startHint}
            </span>
            {startButton}
          </>
        )}
        {(live || resumable) && (
          <Button variant='danger' onClick={s.askEnd}>
            방송 종료
          </Button>
        )}
      </StudioTopBar>

      <div className='mx-auto grid max-w-[1920px] items-start gap-4 p-4 lg:grid-cols-[minmax(0,1fr)_320px] xl:grid-cols-[minmax(0,1fr)_360px] xl:gap-5 xl:px-6 xl:py-5'>
        {/* 방송 중 1023 이하(시안 StudioW768): 한 열로 풀어 안내·미리보기 → 오른쪽 열(채팅·대기열·정보) → 영상·소리 카드 순서 */}
        <div className={`flex min-w-0 flex-col gap-3.5 ${live ? 'max-lg:contents' : ''}`}>
          {editing && <LayoutToolbar editor={s.layout} />}
          {live && !editing && (
            <div role='alert' className='chamfer flex flex-wrap items-center gap-x-2.5 gap-y-2 bg-surface-2 px-3.5 py-2.5 text-sm [--c:6px]'>
              <Icon name='info' className='size-[18px] shrink-0 text-brand' />
              <span>방송 시작 3시간이 되면 자동으로 종료돼요. 남은 시간 {remaining(s.elapsed)}</span>
              {/* 시안엔 없지만 스트리머가 송출 중 클러치 게이지를 봐야 한다(U1): 안내 줄 오른쪽 끝 */}
              <StudioGauge s={s} className='ml-auto w-full sm:w-56' />
            </div>
          )}
          {resumable && <Bar role='status'>방송이 아직 진행 중이에요. 소스를 다시 켜고 이어서 송출하거나, 지금 방송을 종료할 수 있어요.</Bar>}
          {!live &&
            s.env.notices.map((n) => (
              <Bar key={n} role='status'>
                {n}
              </Bar>
            ))}

          <div className='flex flex-col gap-2'>
            <StudioPreview
              stream={media.stream}
              ended={media.videoEnded}
              tall={mobile}
              pip={!mobile && s.pipOn && media.pipStream ? media.pipStream : undefined}
              layout={s.layout.layout}
              className={editing ? 'ring-2 ring-brand' : ''}
              overlay={editing ? <LayoutLayer editor={s.layout} layout={s.layout.layout} selected={s.layout.selected} hasCam={!!(s.pipOn && media.pipStream)} /> : <DonationAlert alerts={s.alerts} owner layout={s.layout.layout} />}
              placeholder={live ? `송출 미리보기 · ${screen ? '화면 공유' : '웹캠'}` : '미리보기 · 아직 송출되지 않아요'}
            >
              {media.videoEnded ? (
                <Button onClick={() => void s.reshare()}>{screen ? '다시 공유하기' : '카메라 다시 켜기'}</Button>
              ) : (
                <>
                  <Button variant='secondary' size='lg' onClick={() => void s.startPreview()}>
                    {screen ? '공유할 화면 선택' : '카메라 켜고 미리보기'}
                  </Button>
                  {screen && <span className='text-xs text-text-dim'>창을 고를 때 &quot;탭 오디오 공유&quot; 또는 &quot;시스템 오디오 공유&quot;를 켜 주세요</span>}
                  {!live && !mobile && s.env.canScreen && modeChips}
                </>
              )}
            </StudioPreview>
            {editing && <span className='text-xs text-text-dim'>영상 밖으로는 못 나가요. 완료하면 시청자 화면에 바로 반영 · 기본 위치: 알림 위 가운데, 웹캠 오른쪽 아래, 채팅 왼쪽 아래</span>}
          </div>

          {(pub.error || media.error) && (
            <Bar role='alert' icon='alert'>
              {pub.error ?? media.error}
            </Bar>
          )}
          {live && !s.connected && (
            <p role='status' className='text-xs text-text-dim'>
              실시간 시청자 정보를 다시 연결하는 중이에요.
            </p>
          )}

          {mobile ? (
            <MobileControls s={s} />
          ) : live ? (
            <div className='grid grid-cols-[repeat(auto-fit,minmax(300px,1fr))] gap-3.5 max-lg:order-4'>
              <Card title='영상'>
                {modeChips}
                {!s.env.canScreen && <p className='text-xs text-text-dim'>이 환경에서는 화면 공유를 쓸 수 없어 카메라 모드만 쓸 수 있어요.</p>}
                {media.videoReady && screen && (
                  <Button variant='neutral' onClick={() => void s.reshare()}>
                    공유할 화면 바꾸기
                  </Button>
                )}
                {pip}
                {layoutRow}
              </Card>
              <Card title='소리'>
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
                    emptyHint={sysEmpty}
                  />
                )}
                {screen && <span className='text-xs text-text-dim'>같은 PC에서 내 방송을 소리 켠 채로 보면 소리가 되돌아와요.</span>}
              </Card>
            </div>
          ) : (
            // 송출 전(시안): 마이크 · 화면 소리(+PIP) 두 카드. 모드 전환은 미리보기 위 칩, 화면 바꾸기는 화면 소리 카드.
            <div className='grid grid-cols-[repeat(auto-fit,minmax(280px,1fr))] gap-3.5'>
              <Panel innerClassName='flex flex-col gap-2.5 p-4'>
                <AudioControl channel='mic' label='마이크' header={micHeader} media={media} available={media.micReady} emptyHint='마이크가 꺼져 있어요. 시청자에게 목소리가 가지 않아요.' />
                {!media.micReady && media.videoReady && (
                  <Button variant='secondary' size='sm' onClick={() => void media.captureMic()}>
                    마이크 켜기
                  </Button>
                )}
              </Panel>
              {screen ? (
                <Panel innerClassName='flex flex-col gap-2.5 p-4'>
                  <AudioControl
                    channel='sys'
                    label='화면 소리'
                    header={
                      <span className='flex grow items-center gap-2'>
                        <b className='text-[15px]'>화면 소리</b>
                        {!media.hasScreenAudio && <span className='ml-auto text-xs text-text-dim'>화면을 선택하면 표시돼요</span>}
                      </span>
                    }
                    media={media}
                    available={media.hasScreenAudio}
                    emptyHint={media.videoReady ? sysEmpty : undefined}
                  />
                  {media.videoReady && (
                    <Button variant='neutral' onClick={() => void s.reshare()}>
                      공유할 화면 바꾸기
                    </Button>
                  )}
                  {pip}
                  {layoutRow}
                  <span className='text-xs text-text-dim'>같은 PC에서 내 방송을 소리 켠 채로 보면 소리가 되돌아와요.</span>
                </Panel>
              ) : (
                !s.env.canScreen && <p className='text-xs text-text-dim'>이 환경에서는 화면 공유를 쓸 수 없어 카메라 모드만 쓸 수 있어요.</p>
              )}
            </div>
          )}

          {mobile && !live && (
            <div className='space-y-2'>
              {startHint && (
                <p id='studio-start-hint' className='text-center text-xs text-text-muted'>
                  {startHint}
                </p>
              )}
              {startButton}
            </div>
          )}
        </div>

        <aside
          className={`grid w-full min-w-0 grid-cols-[repeat(auto-fit,minmax(280px,1fr))] gap-3.5 lg:flex lg:flex-col ${live ? 'max-lg:order-3 max-lg:flex max-lg:flex-col lg:sticky lg:top-5 lg:h-[calc(100vh-104px)] lg:min-h-[560px] lg:gap-3' : ''}`}
        >
          <StudioAside s={s} live={live} />
        </aside>
      </div>
      {dialogs}
    </>
  )
}
