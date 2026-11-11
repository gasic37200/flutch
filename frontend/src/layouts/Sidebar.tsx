import { Link, NavLink } from 'react-router'
import Avatar from '../components/ui/Avatar.tsx'
import Icon, { type IconName } from '../components/ui/Icon.tsx'
import Panel from '../components/ui/Panel.tsx'

export type SidebarChannel = { broadcastId: number; userId: number; nickname: string; viewerCount: number }

// "준비 중" 메뉴는 노출하지 않는다.
const menu: { to: string; label: string; icon: IconName }[] = [
  { to: '/', label: '홈', icon: 'home' },
  { to: '/explore', label: '탐색', icon: 'compass' },
  { to: '/pulse', label: 'PULSE', icon: 'pulse' },
  { to: '/replay', label: '다시보기', icon: 'video' },
]

type Props = { channels: SidebarChannel[]; /** 첫 패널 제목(기본 인기 라이브, 팔로우가 있으면 PULSE) */ title?: string; empty?: string; onNavigate?: () => void }

export default function Sidebar({ channels, title = '인기 라이브', empty = '지금은 라이브 중인 방송이 없어요', onNavigate }: Props) {
  return (
    <nav aria-label='주 메뉴' className='flex h-full flex-col gap-5 overflow-y-auto bg-bg px-4 py-5'>
      <Panel className='shrink-0' innerClassName='flex flex-col gap-1.5 bg-bg p-3'>
        <h2 className='flex items-center gap-2 px-1 pb-1.5 text-[15px] font-black'>
          <Icon name='pulse' className='size-[18px] text-brand' />
          {title}
        </h2>
        {channels.length === 0 ? (
          <p className='px-1 py-1 text-sm text-text-dim'>{empty}</p>
        ) : (
          <ul>
            {channels.map((c) => (
              <li key={c.broadcastId}>
                <Link to={`/live/${c.broadcastId}`} onClick={onNavigate} className='flex items-center gap-2.5 px-1 py-1.5 hover:bg-surface-2'>
                  <Avatar userId={c.userId} nickname={c.nickname} size='sm' />
                  <span className='min-w-0 grow truncate text-sm font-bold'>{c.nickname}</span>
                  <b className='font-mono text-xs text-live'>
                    {c.viewerCount.toLocaleString('ko-KR')}
                    <span className='sr-only'>명 시청 중</span>
                  </b>
                </Link>
              </li>
            ))}
          </ul>
        )}
      </Panel>
      <ul className='shrink-0 space-y-1'>
        {menu.map((m) => (
          <li key={m.to}>
            <NavLink
              to={m.to}
              end={m.to === '/'}
              onClick={onNavigate}
              className={({ isActive }) => `chamfer flex items-center gap-3 px-3 py-2.5 text-[15px] [--c:6px] ${isActive ? 'bg-brand/10 font-bold text-brand' : 'text-text-muted hover:bg-surface-2 hover:text-text'}`}
            >
              <Icon name={m.icon} />
              {m.label}
            </NavLink>
          </li>
        ))}
      </ul>
      {/* 낮은 화면에서는 마스코트가 남은 높이에 맞게 줄고, 너무 낮으면 배너를 숨긴다 */}
      <Link to='/studio' onClick={onNavigate} aria-label='PLAY ON FLUTCH, 방송 시작하기' className='mt-auto flex min-h-0 shrink flex-col [@media(max-height:560px)]:hidden'>
        <Panel tone='hero' className='flex min-h-0 flex-1 flex-col' innerClassName='flex min-h-0 flex-1 flex-col items-center gap-2 bg-surface p-3'>
          <span className='flex shrink-0 items-center gap-1.5 font-display text-xl font-bold italic leading-none text-brand'>
            <Icon name='bolt' className='size-5' />
            PLAY ON
          </span>
          <span className='shrink-0 font-display text-2xl font-bold italic leading-none text-text'>FLUTCH</span>
          <img src='/brand/mascot-controller.png' alt='' className='min-h-0 w-full flex-1 object-contain' style={{ maxHeight: 200 }} />
        </Panel>
      </Link>
    </nav>
  )
}
