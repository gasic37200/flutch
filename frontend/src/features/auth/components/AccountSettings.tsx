import type { Me } from '../../../api/generated/Me.ts'
import Button from '../../../components/ui/Button.tsx'
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
        hint={locked ? `닉네임은 ${f.daysLeft}일 후에 바꿀 수 있어요.` : '한글·영문·숫자 2~12자. 바꾸면 30일 동안 다시 바꿀 수 없어요.'}
        disabled={locked}
        counter
        maxLength={12}
        required
      />
      <FormAlert message={f.error?.message} />
      <Button type='submit' loading={f.pending} disabled={locked}>
        닉네임 변경
      </Button>
    </form>
  )
}

function PasswordSection() {
  const f = usePasswordForm()
  return (
    <form onSubmit={f.submit} className='space-y-3'>
      <Field label='현재 비밀번호' type='password' value={f.values.currentPassword} onChange={(e) => f.set.currentPassword(e.target.value)} error={f.error?.fields.currentPassword} autoComplete='current-password' required />
      <Field label='새 비밀번호' type='password' hint='영문+숫자 8~64자' value={f.values.newPassword} onChange={(e) => f.set.newPassword(e.target.value)} error={f.error?.fields.newPassword} autoComplete='new-password' maxLength={64} required />
      <FormAlert message={f.error?.message} />
      <Button type='submit' loading={f.pending}>
        비밀번호 변경
      </Button>
    </form>
  )
}

type Props = { me: Me; onLogout: () => void }

export default function AccountSettings({ me, onLogout }: Props) {
  return (
    <div className='space-y-4'>
      <Panel innerClassName='p-5'>
        <h2 className='mb-3 font-display text-sm font-bold tracking-wide text-brand'>계정</h2>
        <dl className='text-sm'>
          <dt className='text-text-dim'>아이디</dt>
          <dd className='font-mono text-text'>{me.loginId ?? '카카오 계정'}</dd>
        </dl>
      </Panel>
      <Panel innerClassName='p-5'>
        <NicknameSection />
      </Panel>
      {me.hasPassword && (
        <Panel innerClassName='p-5'>
          <PasswordSection />
        </Panel>
      )}
      <Button variant='outline' onClick={onLogout}>
        로그아웃
      </Button>
    </div>
  )
}
