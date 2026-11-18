import { Link } from 'react-router'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { ClutchGauge as Gauge } from '../../../api/generated/ClutchGauge.ts'
import Avatar from '../../../components/ui/Avatar.tsx'
import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import { Chip } from '../../../components/ui/Chip.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { CATEGORY_LABEL } from '../categories.ts'
import { useNow } from '../../watch/hooks/useWatch.ts'
import type { HeroView } from '../hooks/useHero.ts'
import ThumbnailImage from './ThumbnailImage.tsx'

const count = (n: number) => n.toLocaleString('ko-KR')
const clutchGlow = 'drop-shadow-clutch motion-reduce:drop-shadow-none'

// 패널의 큰 로고: 임박은 뜨거운 로고 + 맥박, 쿨다운은 흐리게(ClutchGauge 단계와 같은 규칙)
const logoOf = (g: Gauge) =>
  g.phase === 'imminent'
    ? { src: '/brand/logo-clutch-time-hot.png', cls: 'animate-beat motion-reduce:animate-none' }
    : { src: '/brand/logo-clutch-time.png', cls: g.phase === 'cooldown' ? 'opacity-60' : '' }

/** 홈 메인 히어로: 라이브 카드 + CLUTCH TIME 패널(데스크톱 오른쪽 340px, 좁으면 카드 아래 가로형). 영상 대신 썸네일(T06c 가정). */
export default function HomeHero({ view }: { view: HeroView }) {
  const { hero } = view
  const b = hero.broadcast
  if (!b) return null
  const sc = hero.mode === 'super_clutch'
  return (
    <section aria-label='메인 라이브' className='grid gap-3 lg:grid-cols-[minmax(0,1fr)_340px] lg:gap-4'>
      <div className='-mx-4 min-w-0 min-[601px]:mx-0'>
        <HeroCard broadcast={b} superClutch={sc} progress={view.active.length > 1 ? view.slotProgress : null} />
      </div>
      <div className='flex min-w-0'>
        {sc ? (
          <>
            <SuperClutchPanel view={view} broadcast={b} />
            <SuperClutchStrip view={view} broadcast={b} />
          </>
        ) : (
          hero.gauge && <ClutchTimePanel broadcast={b} gauge={hero.gauge} topGauge={hero.mode === 'top_gauge'} cooldownMin={view.cooldownMin} />
        )}
      </div>
    </section>
  )
}

const pad2 = (n: number) => String(n).padStart(2, '0')
const elapsed = (ms: number) => {
  const s = Math.max(0, Math.floor(ms / 1000))
  return `${pad2(Math.floor(s / 3600))}:${pad2(Math.floor((s % 3600) / 60))}:${pad2(s % 60)}`
}

function HeroCard({ broadcast: b, superClutch, progress }: { broadcast: Broadcast; superClutch: boolean; progress: number | null }) {
  const now = useNow(true)
  const body = (
    <>
      <div className='relative'>
        <Link to={`/live/${b.id}`} className='relative block aspect-video' aria-label={`${b.owner.nickname} 라이브 보기`}>
          <ThumbnailImage src={b.thumbnailUrl} category={CATEGORY_LABEL[b.category]} />
          <div className='absolute inset-0 bg-linear-to-b from-bg/15 to-bg/55' />
          <div className='absolute top-3 left-3 flex gap-2 sm:top-4 sm:left-4'>
            <Badge tone='live' />
            {superClutch && <Badge tone='superClutch' />}
          </div>
          {superClutch && progress !== null && (
            <div className='absolute inset-x-0 bottom-0 h-1 bg-surface-2/80 sm:h-1.5' aria-hidden>
              <div className='h-full bg-clutch transition-[width] duration-1000 ease-linear' style={{ width: `${progress}%` }} />
            </div>
          )}
        </Link>
        {/* 홈 히어로는 썸네일이라 소리는 시청 화면에서 켠다 */}
        <Link
          to={`/live/${b.id}`}
          className='chamfer absolute right-4 bottom-4 hidden h-9 items-center gap-1.5 bg-bg/75 px-3.5 text-[13px] [--c:6px] min-[601px]:inline-flex'
        >
          <Icon name='volumeOff' className='size-4' />
          소리 켜기
        </Link>
      </div>
      <Link to={`/live/${b.id}`} className='flex items-center gap-3.5 px-4 py-3 min-[601px]:px-[18px] min-[601px]:py-4' aria-label={`${b.owner.nickname} 라이브 보기`}>
        <Avatar userId={b.owner.id} nickname={b.owner.nickname} size='sm' className='min-[601px]:size-12' />
        <div className='flex min-w-0 grow flex-col gap-1.5'>
          <div className='flex flex-wrap items-center gap-2'>
            <b className='text-[17px]'>{b.owner.nickname}</b>
            <Chip>{CATEGORY_LABEL[b.category]}</Chip>
            {b.tags.length > 0 && <span className='text-xs text-text-dim max-[600px]:hidden'>{b.tags.map((t) => `#${t}`).join(' ')}</span>}
          </div>
          <p className='truncate'>{b.title}</p>
        </div>
        <div className='flex shrink-0 flex-col items-end gap-1'>
          <span className='inline-flex items-center gap-1.5 font-mono font-bold whitespace-nowrap text-live' aria-label='시청자 수'>
            <Icon name='user' className='size-4' />
            {count(b.viewerCount)}
          </span>
          {b.startedAt && <span className='font-mono text-[13px] text-text-muted max-[600px]:hidden'>{elapsed(now - Date.parse(b.startedAt))}</span>}
        </div>
      </Link>
    </>
  )
  return (
    <>
      {/* 600px 이하: 시안은 테두리·글로우 없이 전체 폭 */}
      <div className='min-[601px]:hidden'>{body}</div>
      <Panel tone={superClutch ? 'clutch' : 'hero'} className={`hidden min-[601px]:block ${superClutch ? clutchGlow : ''}`} innerClassName='overflow-hidden'>
        {body}
      </Panel>
    </>
  )
}

function gaugeText(g: Gauge, cooldownMin: number | null) {
  if (g.phase === 'cooldown') return `다음 클러치까지 ${cooldownMin ?? 1}분`
  if (g.phase === 'imminent') return '곧 터져요!'
  return '지금, 클러치할 시간!'
}

// 심박 파동: 임박이면 맥박 연출
function Heartbeat({ fast }: { fast: boolean }) {
  return (
    <span className='hidden lg:block'>
      <Icon name='pulse' className={`relative h-7 w-5 text-brand ${fast ? 'animate-beat motion-reduce:animate-none' : ''}`} />
    </span>
  )
}

function ClutchTimePanel({ broadcast: b, gauge: g, topGauge, cooldownMin }: { broadcast: Broadcast; gauge: Gauge; topGauge: boolean; cooldownMin: number | null }) {
  const cooldown = g.phase === 'cooldown'
  const p = Math.round(Math.min(100, Math.max(0, g.percent)))
  return (
    <Panel
      tone='hero'
      className='flex-1 min-[601px]:mx-0'
      innerClassName='relative flex flex-row flex-wrap items-center gap-3 overflow-hidden p-4 lg:flex-col lg:justify-center lg:gap-4 lg:p-[22px]'
    >
      <div className='bg-dots absolute top-0 right-0 hidden h-20 w-36 opacity-60 lg:block' aria-hidden />
      <div className='absolute bottom-3 left-3 hidden h-[26px] w-[52px] border-b-2 border-l-2 border-brand lg:block' aria-hidden />
      <span className='relative hidden self-start text-xs text-text-muted lg:block'>{topGauge ? '게이지 1위 라이브' : '지금 가장 많이 보는 라이브'}</span>
      <img src={logoOf(g).src} alt='CLUTCH TIME' className={`relative w-[120px] shrink-0 object-contain lg:w-full lg:max-w-[260px] ${logoOf(g).cls}`} />
      <Heartbeat fast={g.phase === 'imminent'} />
      <div className='relative flex min-w-0 grow basis-[200px] flex-col gap-2 lg:w-full lg:grow-0 lg:basis-auto'>
        <div className='flex items-center gap-3'>
          <div role='progressbar' aria-label='클러치 게이지' aria-valuemin={0} aria-valuemax={100} aria-valuenow={p} className='chamfer h-3 grow bg-surface-2 [--c:6px]'>
            <div className={`chamfer h-full [--c:6px] ${cooldown ? 'bg-text-dim' : 'bg-brand'}`} style={{ width: `${p}%` }} />
          </div>
          <b className='font-mono text-lg'>{p}%</b>
        </div>
        <span className='text-base font-bold'>{gaugeText(g, cooldownMin)}</span>
      </div>
      <Button to={`/live/${b.id}`} size='lg' className='relative'>
        클러치하러 가기
        <svg viewBox='0 0 24 24' fill='none' stroke='currentColor' strokeWidth={2.4} strokeLinecap='round' strokeLinejoin='round' className='size-4' aria-hidden>
          <path d='M5 12h14M13 6l6 6-6 6' />
        </svg>
      </Button>
    </Panel>
  )
}

function SuperClutchPanel({ view, broadcast: b }: { view: HeroView; broadcast: Broadcast }) {
  return (
    <Panel tone='clutch' className={`hidden flex-1 lg:flex ${clutchGlow}`} innerClassName='flex flex-col items-center gap-3.5 p-[22px]'>
      <img src='/brand/logo-super-clutch.png' alt='SUPER CLUTCH' className='w-full max-w-[280px]' />
      {/* 혼자뿐이면 넘어갈 다음 방송이 없으니 타이머·진행 바를 숨긴다 */}
      {view.active.length > 1 && (
        <>
          <div className='flex w-full justify-between text-[13px] text-text-muted'>
            <span>다음 방송까지</span>
            <b className='font-mono text-base text-text' aria-live='off'>
              {view.slotLeft}
            </b>
          </div>
          <div className='chamfer h-2.5 w-full bg-surface-2 [--c:6px]' aria-hidden>
            <div className='chamfer h-full bg-clutch transition-[width] duration-1000 ease-linear [--c:6px]' style={{ width: `${view.slotProgress}%` }} />
          </div>
        </>
      )}
      <Link
        to={`/live/${b.id}`}
        className='chamfer flex h-[46px] w-full items-center justify-center bg-clutch text-[15px] font-bold text-bg [--c:6px] focus-visible:outline-offset-4'
      >
        지금 보러 가기
      </Link>
      <div className='h-px w-full bg-surface-2' />
      <h2 className='self-start font-display text-[13px] font-bold tracking-wider text-text-muted'>진행 중인 SUPER CLUTCH · {view.active.length}</h2>
      <ol className='flex w-full flex-col gap-2'>
        {view.active.map((a, i) => (
          <li key={a.broadcast.id}>
            <Link to={`/live/${a.broadcast.id}`} className='flex items-center gap-2.5 text-sm hover:text-brand'>
              <b className='w-4 font-mono text-text-dim'>{i === 0 ? '▶' : i}</b>
              <Avatar userId={a.broadcast.owner.id} nickname={a.broadcast.owner.nickname} size='xs' />
              <span className='min-w-0 grow truncate'>
                {a.broadcast.owner.nickname}
                {i === 0 && ' (지금)'}
              </span>
              <span className='font-mono text-xs text-text-muted'>{a.left} 남음</span>
            </Link>
          </li>
        ))}
      </ol>
    </Panel>
  )
}

function SuperClutchStrip({ view, broadcast: b }: { view: HeroView; broadcast: Broadcast }) {
  return (
    <Link to={`/live/${b.id}`} className='flex items-center gap-2.5 px-1 lg:hidden'>
      <img src='/brand/logo-super-clutch.png' alt='SUPER CLUTCH' className='w-[90px]' />
      <span className='min-w-0 grow truncate text-xs text-text-muted'>
        {view.active.length > 1 ? (
          <>
            <b className='font-mono text-text'>{view.slotLeft}</b> 뒤 {view.next?.owner.nickname} · 진행 중 {view.active.length}
          </>
        ) : (
          '진행 중 1'
        )}
      </span>
    </Link>
  )
}
