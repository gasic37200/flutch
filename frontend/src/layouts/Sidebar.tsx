import { Link, NavLink } from 'react-router'
import Avatar from '../components/ui/Avatar.tsx'
import Badge from '../components/ui/Badge.tsx'
import Icon, { type IconName } from '../components/ui/Icon.tsx'

export type SidebarChannel = { broadcastId: number; userId: number; nickname: string; viewerCount: number }

// 팔로잉·다시보기·PULSE는 이후 단계에서 이 배열에 추가한다("준비 중" 메뉴는 노출하지 않는다).
const menu: { to: string; label: string; icon: IconName }[] = [
  { to: '/', label: '홈', icon: 'home' },
  { to: '/explore', label: '탐색', icon: 'compass' },
]

type Props = { channels: SidebarChannel[]; onNavigate?: () => void }

export default function Sidebar({ channels, onNavigate }: Props) {
  return (
    <nav aria-label='주 메뉴' className='flex h-full flex-col gap-4 overflow-y-auto bg-bg p-3'>
      <section aria-labelledby='sidebar-live'>
        <h2 id='sidebar-live' className='mb-2 flex items-center gap-2 font-display text-sm font-bold tracking-wide text-brand'>
          <Icon name='pulse' className='size-4' />
          인기 라이브
        </h2>
        {channels.length === 0 ? (
          <p className='px-2 text-sm text-text-dim'>지금은 라이브 중인 방송이 없어요</p>
        ) : (
          <ul className='space-y-1'>
            {channels.map((c) => (
              <li key={c.broadcastId}>
                <Link to={`/live/${c.broadcastId}`} onClick={onNavigate} className='flex items-center gap-2 rounded-xs p-2 hover:bg-surface-2'>
                  <Avatar userId={c.userId} nickname={c.nickname} size='sm' />
                  <span className='min-w-0 grow'>
                    <span className='block truncate text-sm font-semibold'>{c.nickname}</span>
                    <span className='font-mono text-xs text-text-muted'>{c.viewerCount.toLocaleString('ko-KR')}명</span>
                  </span>
                  <Badge tone='live' />
                </Link>
              </li>
            ))}
          </ul>
        )}
      </section>
      <ul className='space-y-1 border-t border-border pt-4'>
        {menu.map((m) => (
          <li key={m.to}>
            <NavLink
              to={m.to}
              end
              onClick={onNavigate}
              className={({ isActive }) => `flex items-center gap-3 rounded-xs px-3 py-2 text-sm font-semibold ${isActive ? 'bg-surface-2 text-brand' : 'text-text-muted hover:bg-surface-2 hover:text-text'}`}
            >
              <Icon name={m.icon} />
              {m.label}
            </NavLink>
          </li>
        ))}
      </ul>
      <Link to='/studio' onClick={onNavigate} className='mt-auto block'>
        <img src='/brand/banner-play-on-flutch.png' alt='PLAY ON FLUTCH — 방송하기' className='mx-auto w-full max-w-40' />
      </Link>
    </nav>
  )
}
