import { useState } from 'react'
import { Form, Link } from 'react-router'
import Avatar from '../components/ui/Avatar.tsx'
import BeatAmount from '../components/ui/BeatAmount.tsx'
import Button from '../components/ui/Button.tsx'
import Dropdown from '../components/ui/Dropdown.tsx'
import Field from '../components/ui/Field.tsx'
import Icon from '../components/ui/Icon.tsx'
import Tooltip from '../components/ui/Tooltip.tsx'

export type ShellUser = { id: number; nickname: string; avatarUrl?: string | null; balance: number }

type Props = { user: ShellUser | null; onMenu: () => void; onLogout?: () => void }

export default function Header({ user, onMenu, onLogout }: Props) {
  // 모바일에서만 의미 있는 검색창 펼침(md 이상은 항상 보임)
  const [searchOpen, setSearchOpen] = useState(false)
  return (
    <header className='sticky top-0 z-30 flex h-14 items-center gap-3 border-b border-border bg-bg/90 px-4 backdrop-blur'>
      <button type='button' onClick={onMenu} aria-label='메뉴 열기' className='text-text-muted hover:text-text lg:hidden'>
        <Icon name='menu' className='size-6' />
      </button>
      <Link to='/' aria-label='FLUTCH 홈' className='shrink-0'>
        <img src='/brand/logo-flutch.png' alt='FLUTCH' className='h-7 w-auto' />
      </Link>

      <Form
        action='/explore'
        role='search'
        className={searchOpen ? 'absolute inset-x-0 top-14 border-b border-border bg-bg p-3 md:static md:border-0 md:p-0' : 'hidden md:block'}
        // 제출하면 탐색 화면으로 이동하므로 펼침을 닫는다
        onSubmit={() => setSearchOpen(false)}
      >
        <Field label='검색' hideLabel name='q' type='search' placeholder='방송, 스트리머 검색' className='md:w-80 lg:w-96' />
      </Form>

      <div className='ml-auto flex items-center gap-2'>
        <button type='button' onClick={() => setSearchOpen(!searchOpen)} aria-label='검색' aria-expanded={searchOpen} className='p-2 text-text-muted hover:text-text md:hidden'>
          <Icon name='search' />
        </button>
        <Tooltip text='방송하기'>
          <Button to='/studio' variant='secondary' size='icon' aria-label='방송하기'>
            <Icon name='camera' />
          </Button>
        </Tooltip>
        {user ? (
          <>
            <Link to='/wallet' aria-label='내 지갑' className='hidden h-9 items-center rounded-full border border-brand px-3 sm:inline-flex'>
              <BeatAmount amount={user.balance} />
            </Link>
            <Dropdown
              label={`${user.nickname} 메뉴`}
              trigger={<Avatar userId={user.id} nickname={user.nickname} src={user.avatarUrl} size='sm' />}
              header={
                <div className='border-b border-border px-4 py-2 sm:hidden'>
                  <BeatAmount amount={user.balance} />
                </div>
              }
              items={[
                { label: '내 채널', to: `/users/${user.id}` },
                { label: '내 지갑', to: '/wallet' },
                { label: '내 정보', to: '/me' },
                { label: '로그아웃', onSelect: onLogout },
              ]}
            />
          </>
        ) : (
          <Button to='/login'>로그인</Button>
        )}
      </div>
    </header>
  )
}
