import { Link } from 'react-router'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { ClutchGauge as Gauge } from '../../../api/generated/ClutchGauge.ts'
import Avatar from '../../../components/ui/Avatar.tsx'
import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import { Chip } from '../../../components/ui/Chip.tsx'
import ClutchGauge from '../../../components/ui/ClutchGauge.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { CATEGORY_LABEL } from '../categories.ts'
import type { HeroView } from '../hooks/useHero.ts'
import ThumbnailImage from './ThumbnailImage.tsx'

const count = (n: number) => n.toLocaleString('ko-KR')
const clutchGlow = 'drop-shadow-clutch motion-reduce:drop-shadow-none'

// 패널의 큰 로고: 임박은 뜨거운 로고 + 맥박, 쿨다운은 흐리게(ClutchGauge 단계와 같은 규칙)
const logoOf = (g: Gauge) =>
  g.phase === 'imminent'
    ? { src: '/brand/logo-clutch-time-hot.png', cls: 'animate-beat motion-reduce:animate-none' }
    : { src: '/brand/logo-clutch-time.png', cls: g.phase === 'cooldown' ? 'opacity-60' : '' }

/** 홈 메인 히어로: 라이브 카드 + 오른쪽 CLUTCH TIME 패널(모바일은 카드 아래 한 줄). 영상 대신 썸네일(T06c 가정). */
export default function HomeHero({ view }: { view: HeroView }) {
  const { hero } = view
  const b = hero.broadcast
  if (!b) return null
  const sc = hero.mode === 'super_clutch'
  return (
    <section aria-label='메인 라이브' className='flex flex-col gap-3 lg:flex-row lg:flex-wrap lg:gap-4'>
      <div className='min-w-0 lg:flex-[999_1_520px]'>
        <HeroCard broadcast={b} superClutch={sc} progress={view.active.length > 1 ? view.slotProgress : null} />
      </div>
      <div className='hidden min-w-0 lg:flex lg:flex-[1_1_300px]'>
        {sc ? <SuperClutchPanel view={view} broadcast={b} /> : hero.gauge && <ClutchTimePanel broadcast={b} gauge={hero.gauge} topGauge={hero.mode === 'top_gauge'} cooldownMin={view.cooldownMin} />}
      </div>
      <div className='lg:hidden'>
        {sc ? <SuperClutchStrip view={view} broadcast={b} /> : hero.gauge && <ClutchTimeStrip broadcast={b} gauge={hero.gauge} cooldownMin={view.cooldownMin} />}
      </div>
    </section>
  )
}

function HeroCard({ broadcast: b, superClutch, progress }: { broadcast: Broadcast; superClutch: boolean; progress: number | null }) {
  return (
    <Panel tone={superClutch ? 'clutch' : 'hero'} className={superClutch ? clutchGlow : ''} innerClassName='overflow-hidden'>
      <Link to={`/live/${b.id}`} className='block' aria-label={`${b.owner.nickname} 라이브 보기`}>
        <div className='relative aspect-video lg:aspect-auto lg:h-[min(34vw,800px)]'>
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
        </div>
        <div className='flex items-center gap-3.5 px-4 py-3 sm:px-[18px] sm:py-4'>
          <Avatar userId={b.owner.id} nickname={b.owner.nickname} size='sm' className='sm:size-12' />
          <div className='flex min-w-0 grow flex-col gap-1.5'>
            <div className='flex flex-wrap items-center gap-2'>
              <b className='text-[17px]'>{b.owner.nickname}</b>
              <Chip>{CATEGORY_LABEL[b.category]}</Chip>
              {b.tags.length > 0 && <span className='text-xs text-text-dim'>{b.tags.map((t) => `#${t}`).join(' ')}</span>}
            </div>
            <p className='truncate'>{b.title}</p>
          </div>
          <span className='inline-flex items-center gap-1.5 font-mono font-bold whitespace-nowrap text-live'>
            <Icon name='user' className='size-4' />
            {count(b.viewerCount)}명
          </span>
        </div>
      </Link>
    </Panel>
  )
}

function gaugeText(g: Gauge, cooldownMin: number | null) {
  if (g.phase === 'cooldown') return `다음 클러치까지 ${cooldownMin ?? 1}분`
  if (g.phase === 'imminent') return '곧 터져요!'
  return '지금, 클러치할 시간!'
}

// 심박 라인: 임박이면 맥박 연출
function Heartbeat({ fast }: { fast: boolean }) {
  return (
    <svg viewBox='0 0 240 28' fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round' strokeLinejoin='round' aria-hidden className={`relative h-7 w-full max-w-[260px] text-brand ${fast ? 'animate-beat motion-reduce:animate-none' : ''}`}>
      <path d='M0 14h80l8-10 10 20 8-14 6 4h128' />
    </svg>
  )
}

function ClutchTimePanel({ broadcast: b, gauge: g, topGauge, cooldownMin }: { broadcast: Broadcast; gauge: Gauge; topGauge: boolean; cooldownMin: number | null }) {
  const cooldown = g.phase === 'cooldown'
  return (
    <Panel tone='hero' className='flex-1' innerClassName='relative flex flex-col items-center justify-center gap-4 overflow-hidden p-[22px]'>
      <div className='bg-dots absolute top-0 right-0 h-20 w-36 opacity-60' aria-hidden />
      <div className='absolute bottom-3 left-3 h-[26px] w-[52px] border-b-2 border-l-2 border-brand' aria-hidden />
      <span className='relative self-start text-xs text-text-muted'>{topGauge ? '게이지 1위 라이브' : '지금 가장 많이 보는 라이브'}</span>
      <img src={logoOf(g).src} alt='CLUTCH TIME' className={`relative w-full max-w-[260px] object-contain ${logoOf(g).cls}`} />
      <Heartbeat fast={g.phase === 'imminent'} />
      <ClutchGauge percent={g.percent} cooldown={cooldown} logo={false} className='relative w-full' />
      <p className='relative text-base font-bold'>{gaugeText(g, cooldownMin)}</p>
      <Button to={`/live/${b.id}`} size='lg' className='relative'>
        클러치하러 가기
      </Button>
    </Panel>
  )
}

function ClutchTimeStrip({ broadcast: b, gauge: g, cooldownMin }: { broadcast: Broadcast; gauge: Gauge; cooldownMin: number | null }) {
  const cooldown = g.phase === 'cooldown'
  return (
    <Panel tone='hero' interactive>
      <Link to={`/live/${b.id}`} className='flex items-center gap-3 px-3.5 py-3' aria-label={`${b.owner.nickname} 라이브에서 클러치하기`}>
        <img src={logoOf(g).src} alt='CLUTCH TIME' className={`w-[88px] object-contain ${logoOf(g).cls}`} />
        <div className='flex min-w-0 grow flex-col gap-1.5'>
          <ClutchGauge percent={g.percent} cooldown={cooldown} logo={false} />
          <span className='text-[13px] font-bold'>{gaugeText(g, cooldownMin)}</span>
        </div>
        <Icon name='chevronDown' className='size-[18px] -rotate-90 text-brand' />
      </Link>
    </Panel>
  )
}

function SuperClutchPanel({ view, broadcast: b }: { view: HeroView; broadcast: Broadcast }) {
  return (
    <Panel tone='clutch' className={`flex-1 ${clutchGlow}`} innerClassName='flex flex-col items-center gap-3.5 p-[22px]'>
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
    <Link to={`/live/${b.id}`} className='flex items-center gap-2.5 px-1'>
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
