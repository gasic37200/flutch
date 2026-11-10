import type { Me } from '../../../api/generated/Me.ts'
import Button from '../../../components/ui/Button.tsx'
import Avatar from '../../../components/ui/Avatar.tsx'
import Field from '../../../components/ui/Field.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { useNicknameForm, usePasswordForm } from '../hooks/useAuthForms.ts'
import { FormAlert } from './FormParts.tsx'

function NicknameSection() {
  const f = useNicknameForm()
  const locked = f.daysLeft > 0
  return (
    <form onSubmit={f.submit} className='space-y-3'>
      <Field
        label='닉네임'
        value={f.nickname}
        onChange={(e) => f.setNickname(e.target.value)}
        error={f.error?.fields.nickname}
        hint={locked ? `${f.daysLeft}일 뒤에 다시 바꿀 수 있어요 (변경 후 30일 제한)` : '2~12자 한글·영문·숫자. 바꾸면 30일 동안 다시 바꿀 수 없어요.'}
        disabled={locked}
        counter
        maxLength={12}
        required
      />
      <FormAlert message={f.error?.message} />
      <Button type='submit' size='lg' loading={f.pending} disabled={locked} className='w-full'>
        닉네임 변경
      </Button>
    </form>
  )
}

function PasswordSection() {
  const f = usePasswordForm()
  return (
    <form onSubmit={f.submit} className='space-y-3'>
      <b className='block text-sm'>비밀번호 변경</b>
      <Field label='현재 비밀번호' type='password' value={f.values.currentPassword} onChange={(e) => f.set.currentPassword(e.target.value)} error={f.error?.fields.currentPassword} autoComplete='current-password' required />
      <Field label='새 비밀번호' type='password' hint='영문+숫자 8~64자' value={f.values.newPassword} onChange={(e) => f.set.newPassword(e.target.value)} error={f.error?.fields.newPassword} autoComplete='new-password' maxLength={64} required />
      <p className='text-xs text-text-dim'>바꾸면 이 기기를 뺀 다른 기기에서 모두 로그아웃돼요</p>
      <FormAlert message={f.error?.message} />
      <Button type='submit' size='lg' loading={f.pending} className='w-full'>
        비밀번호 변경
      </Button>
    </form>
  )
}

type Props = { me: Me; onLogout: () => void }

export default function AccountSettings({ me, onLogout }: Props) {
  return (
    <div className='space-y-4'>
      <Panel innerClassName='flex flex-col gap-3.5 p-5'>
        <div className='flex items-center gap-3'>
          <Avatar userId={me.id} nickname={me.nickname} size='lg' />
          <div className='flex min-w-0 flex-col gap-0.5'>
            <b className='truncate text-[17px]'>{me.nickname}</b>
            <span className='text-xs text-text-dim'>{me.loginId ? `아이디 ${me.loginId} · 바꿀 수 없어요` : '카카오 계정'}</span>
          </div>
        </div>
        <NicknameSection />
        {me.hasPassword && (
          <>
            <div className='h-px bg-surface-2' />
            <PasswordSection />
          </>
        )}
      </Panel>
      <Button variant='outline' onClick={onLogout}>
        로그아웃
      </Button>
    </div>
  )
}
