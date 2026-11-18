import { useState } from 'react'
import { Form, Link } from 'react-router'
import Avatar from '../components/ui/Avatar.tsx'
import BeatAmount from '../components/ui/BeatAmount.tsx'
import Button from '../components/ui/Button.tsx'
import Dropdown from '../components/ui/Dropdown.tsx'
import Icon from '../components/ui/Icon.tsx'
import Tooltip from '../components/ui/Tooltip.tsx'

export type ShellUser = { id: number; nickname: string; avatarUrl?: string | null; balance: number }

type Props = { user: ShellUser | null; onMenu: () => void; onLogout?: () => void; /** 안쪽 콘텐츠 최대 폭(셸과 같은 축) */ maxClass?: string; /** 600 이하에서 숨김(시청 화면은 영상 위 뒤로가기를 쓴다) */ hideBelowSm?: boolean }

export default function Header({ user, onMenu, onLogout, maxClass = 'max-w-[1840px]', hideBelowSm }: Props) {
  // 모바일에서만 의미 있는 검색창 펼침(md 이상은 항상 보임)
  const [searchOpen, setSearchOpen] = useState(false)
  return (
    <header className={`sticky top-0 z-30 border-b border-surface-2 bg-bg ${hideBelowSm ? 'max-[601px]:hidden' : ''}`}>
      <div className={`mx-auto flex h-14 items-center gap-2 px-2 sm:gap-4 sm:px-6 md:h-16 ${maxClass}`}>
      <button type='button' onClick={onMenu} aria-label='메뉴 열기' className='flex size-11 items-center justify-center text-text lg:hidden'>
        <Icon name='menu' className='size-[22px]' />
      </button>
      <Link to='/' aria-label='FLUTCH 홈' className='shrink-0'>
        <img src='/brand/logo-flutch.png' alt='FLUTCH' className='h-6 w-auto md:h-[30px]' />
      </Link>

      {/* 검색창은 가운데에서 늘어난다. 모바일은 아이콘으로 펼친다. */}
      <Form
        action='/explore'
        role='search'
        className={searchOpen ? 'absolute inset-x-0 top-full border-b border-surface-2 bg-bg p-3 md:static md:flex md:min-w-0 md:flex-1 md:justify-center md:border-0 md:p-0' : 'hidden md:flex md:min-w-0 md:flex-1 md:justify-center'}
        // 제출하면 탐색 화면으로 이동하므로 펼침을 닫는다
        onSubmit={() => setSearchOpen(false)}
      >
        <label className='chamfer flex h-10 w-full max-w-[520px] items-center gap-2.5 bg-surface px-3.5 [--c:6px] focus-within:outline-2 focus-within:outline-text'>
          <Icon name='search' className='size-[18px] text-text-muted' />
          <span className='sr-only'>검색</span>
          <input type='search' name='q' placeholder='게임, 버추얼, 스트리머 검색…' className='min-w-0 flex-1 bg-transparent text-sm text-text outline-none placeholder:text-text-dim' />
        </label>
      </Form>

      <div className='ml-auto flex items-center gap-1 sm:gap-3 md:ml-0'>
        <button type='button' onClick={() => setSearchOpen(!searchOpen)} aria-label='검색' aria-expanded={searchOpen} className='flex size-11 items-center justify-center text-text md:hidden'>
          <Icon name='search' className='size-5' />
        </button>
        {user && (
          <Link
            to='/wallet'
            aria-label={`내 지갑, ${user.balance.toLocaleString('ko-KR')} 비트`}
            className='hidden h-11 shrink-0 items-center whitespace-nowrap rounded-full border border-brand bg-brand/8 px-3.5 md:inline-flex'
          >
            <BeatAmount amount={user.balance} unit className='text-sm' />
          </Link>
        )}
        <Tooltip text='방송하기'>
          <Button to={user ? '/studio' : '/login'} variant='neutral' size='icon' aria-label={user ? '방송하기' : '방송하기 (로그인 필요)'}>
            <Icon name='video' className='size-[22px]' />
          </Button>
        </Tooltip>
        {user ? (
          <Dropdown
            label={`${user.nickname} 메뉴`}
            trigger={<Avatar userId={user.id} nickname={user.nickname} src={user.avatarUrl} size='sm' />}
            header={
              <div className='flex flex-col gap-0.5'>
                <b className='text-sm'>{user.nickname}</b>
                <BeatAmount amount={user.balance} unit className='text-xs' />
              </div>
            }
            items={[
              { label: '내 채널', to: `/users/${user.id}` },
              { label: '내 지갑', to: '/wallet' },
              { label: '내 방송 관리', to: '/manage' },
              { label: '내 정보', to: '/me' },
              { label: '로그아웃', onSelect: onLogout, muted: true },
            ]}
          />
        ) : (
          <Button to='/login' variant='secondary' size='lg'>
            로그인
          </Button>
        )}
      </div>
      </div>
    </header>
  )
}
