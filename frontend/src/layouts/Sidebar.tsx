import { Link, NavLink } from 'react-router'
import Avatar from '../components/ui/Avatar.tsx'
import Icon, { type IconName } from '../components/ui/Icon.tsx'
import Panel from '../components/ui/Panel.tsx'

export type SidebarChannel = { broadcastId: number; userId: number; nickname: string; viewerCount: number }

// 팔로잉·PULSE는 이후 단계에서 이 배열에 추가한다("준비 중" 메뉴는 노출하지 않는다).
const menu: { to: string; label: string; icon: IconName }[] = [
  { to: '/', label: '홈', icon: 'home' },
  { to: '/explore', label: '탐색', icon: 'compass' },
  { to: '/replay', label: '다시보기', icon: 'video' },
]

type Props = { channels: SidebarChannel[]; onNavigate?: () => void }

export default function Sidebar({ channels, onNavigate }: Props) {
  return (
    <nav aria-label='주 메뉴' className='flex h-full flex-col gap-5 overflow-y-auto bg-bg px-4 py-5'>
      <Panel innerClassName='flex flex-col gap-1.5 bg-bg p-3'>
        <h2 className='flex items-center gap-2 px-1 pb-1.5 text-[15px] font-black'>
          <Icon name='pulse' className='size-[18px] text-brand' />
          인기 라이브
        </h2>
        {channels.length === 0 ? (
          <p className='px-1 py-1 text-sm text-text-dim'>지금은 라이브 중인 방송이 없어요</p>
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
      <ul className='space-y-1'>
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
      <Link to='/studio' onClick={onNavigate} aria-label='PLAY ON FLUTCH, 방송 시작하기' className='chamfer mt-auto block overflow-hidden [--c:12px]'>
        <img src='/brand/banner-play-on-flutch.png' alt='' className='block w-full' />
      </Link>
    </nav>
  )
}
