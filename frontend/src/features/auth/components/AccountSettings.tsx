import type { Me } from '../../../api/generated/Me.ts'
import Button from '../../../components/ui/Button.tsx'
import Avatar from '../../../components/ui/Avatar.tsx'
import BeatAmount from '../../../components/ui/BeatAmount.tsx'
import Field from '../../../components/ui/Field.tsx'
import Modal from '../../../components/ui/Modal.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { emoteSrc } from '../../../components/ui/mascot.ts'
import { useNicknameForm, usePasswordForm, useWithdrawForm } from '../hooks/useAuthForms.ts'
import { CheckRow, FormAlert } from './FormParts.tsx'

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
      {/* 30일 제한 중에는 시안대로 버튼 없이 읽기 입력 + 안내만 */}
      {!locked && (
        <Button type='submit' size='lg' loading={f.pending} className='w-full'>
          닉네임 변경
        </Button>
      )}
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

// 위험 확인은 빨강 대신 흰 테두리 버튼(DESIGN). 남는 쪽(라임)이 기본 선택.
function WithdrawSection({ me }: { me: Me }) {
  const f = useWithdrawForm(me.balance, me.hasPassword)
  return (
    <>
      <button type='button' onClick={f.show} className='self-start text-[13px] text-text-muted underline hover:text-text'>
        회원 탈퇴
      </button>
      <Modal
        open={f.open}
        onClose={f.close}
        title='정말 탈퇴할까요?'
        footer={
          <>
            <Button onClick={f.close} disabled={f.pending}>
              남아 있을게요
            </Button>
            <Button type='submit' form='withdraw-form' variant='outline' loading={f.pending} disabled={!f.canSubmit}>
              탈퇴
            </Button>
          </>
        }
      >
        <form id='withdraw-form' onSubmit={f.submit} className='flex flex-col gap-3.5'>
          <img src={emoteSrc('crying')} alt='' className='h-16 w-auto self-start object-contain' />
          <div className='chamfer flex flex-col gap-1.5 bg-surface-2 px-3.5 py-3 text-[13px] leading-relaxed [--c:6px]'>
            {f.needsAgree && (
              <span>
                남은 <BeatAmount amount={me.balance} unit />는 탈퇴하면 <b>사라지고 되돌릴 수 없어요.</b>
              </span>
            )}
            <span className='text-text-muted'>채팅·후원 기록은 "탈퇴한 사용자"로 남아요. {me.hasPassword ? '아이디는 다시 쓸 수 없고, ' : '카카오 연결이 해제되고, '}닉네임은 30일 뒤 풀려요.</span>
          </div>
          {me.hasPassword && (
            <Field
              label='비밀번호 확인'
              type='password'
              placeholder='비밀번호를 다시 입력해 주세요'
              value={f.values.password}
              onChange={(e) => f.set.password(e.target.value)}
              error={f.passwordError}
              autoComplete='current-password'
              required
            />
          )}
          {f.needsAgree && (
            <CheckRow checked={f.values.agreed} onChange={f.set.agreed}>
              비트 소멸과 위 내용을 이해했어요
            </CheckRow>
          )}
          <FormAlert message={f.message} />
          <span className='text-xs text-text-dim'>방송 중이면 먼저 방송을 종료해야 탈퇴할 수 있어요.</span>
        </form>
      </Modal>
    </>
  )
}

// 로그아웃은 헤더 프로필 메뉴에 있다(시안의 내 정보 화면에는 없음).
export default function AccountSettings({ me }: { me: Me }) {
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
        <WithdrawSection me={me} />
      </Panel>
    </div>
  )
}
