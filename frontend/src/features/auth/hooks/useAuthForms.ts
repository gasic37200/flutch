import { useQueryClient } from '@tanstack/react-query'
import { useState, type FormEvent } from 'react'
import { useSearchParams } from 'react-router'
import { ApiRequestError } from '../../../api/client.ts'
import type { TestLoginRequest } from '../../../api/generated/TestLoginRequest.ts'
import { useToast } from '../../../hooks/useToast.ts'
import { formError } from './formError.ts'
import { ME_KEY, useChangePassword, useKakaoEnabled, useKakaoPending, useKakaoSignup, useLogin, useMe, useSignup, useTestLogin, useUpdateNickname, useWithdraw } from './useAuth.ts'

const prevent = (fn: () => void) => (e: FormEvent) => {
  e.preventDefault()
  fn()
}

export function useLoginForm() {
  const [loginId, setLoginId] = useState('')
  const [password, setPassword] = useState('')
  const [remember, setRemember] = useState(false)
  const login = useLogin()
  const test = useTestLogin()
  const failed = login.isError ? login.error : test.isError ? test.error : null
  const kakao = useKakaoEnabled()
  const [params] = useSearchParams()
  return {
    kakaoEnabled: kakao.data === true,
    // 카카오 콜백이 실패하면 서버가 /login?kakao=error로 돌려보낸다
    kakaoFailed: params.get('kakao') === 'error',
    values: { loginId, password, remember },
    set: { loginId: setLoginId, password: setPassword, remember: setRemember },
    pending: login.isPending || test.isPending,
    error: failed ? formError(failed, ['loginId', 'password']) : null,
    submit: prevent(() => login.mutate({ loginId: loginId.trim(), password, remember })),
    testLogin: (account: TestLoginRequest['account']) => test.mutate({ account }),
  }
}

export function useSignupForm() {
  const [loginId, setLoginId] = useState('')
  const [password, setPassword] = useState('')
  const [nickname, setNickname] = useState('')
  const [agreeTerms, setAgreeTerms] = useState(false)
  const [agreePrivacy, setAgreePrivacy] = useState(false)
  const [over14, setOver14] = useState(false)
  const signup = useSignup()
  return {
    values: { loginId, password, nickname, agreeTerms, agreePrivacy, over14 },
    set: { loginId: setLoginId, password: setPassword, nickname: setNickname, agreeTerms: setAgreeTerms, agreePrivacy: setAgreePrivacy, over14: setOver14 },
    pending: signup.isPending,
    canSubmit: agreeTerms && agreePrivacy && over14,
    error: signup.isError ? formError(signup.error, ['loginId', 'password', 'nickname']) : null,
    submit: prevent(() => signup.mutate({ loginId: loginId.trim(), password, nickname: nickname.trim(), agreeTerms, agreePrivacy, over14 })),
  }
}

/** 카카오 첫 로그인의 닉네임 설정. 닉네임은 카카오 닉네임을 줄인 후보로 미리 채운다. */
export function useKakaoSignupForm() {
  const pending = useKakaoPending()
  const [draft, setDraft] = useState<string | null>(null)
  const [terms, setTerms] = useState(false)
  const [over14, setOver14] = useState(false)
  const signup = useKakaoSignup()
  const expired = pending.isError || (signup.error instanceof ApiRequestError && signup.error.code === 'AUTH_KAKAO_SIGNUP_EXPIRED')
  return {
    loading: pending.isPending,
    expired,
    values: { nickname: draft ?? pending.data?.nickname ?? '', terms, over14 },
    set: { nickname: setDraft, terms: setTerms, over14: setOver14 },
    pending: signup.isPending,
    canSubmit: terms && over14,
    error: signup.isError ? formError(signup.error, ['nickname']) : null,
    submit: prevent(() =>
      signup.mutate({ nickname: (draft ?? pending.data?.nickname ?? '').trim(), agreeTerms: terms, agreePrivacy: terms, over14 }),
    ),
  }
}

const DAY = 86_400_000

export function useNicknameForm() {
  const { data: me } = useMe()
  const [draft, setDraft] = useState<string | null>(null)
  const [now] = useState(Date.now)
  const toast = useToast()
  const update = useUpdateNickname()
  const nickname = draft ?? me?.nickname ?? ''
  // 서버 시각 기준 변경 가능 시각까지 남은 일수(올림). 0이면 지금 바꿀 수 있다.
  const left = me?.nicknameChangeableAt ? new Date(me.nicknameChangeableAt).getTime() - now : 0
  // 방금 바꾼 직후에 31일로 보이지 않게 1분 여유를 두되, 잠겨 있으면 최소 1일
  const daysLeft = left > 0 ? Math.max(1, Math.ceil((left - 60_000) / DAY)) : 0
  return {
    nickname,
    setNickname: setDraft,
    daysLeft,
    pending: update.isPending,
    error: update.isError ? formError(update.error, ['nickname']) : null,
    submit: prevent(() =>
      update.mutate(nickname.trim(), {
        onSuccess: () => {
          setDraft(null)
          toast({ tone: 'success', message: '닉네임을 바꿨어요.' })
        },
      }),
    ),
  }
}

export function usePasswordForm() {
  const [currentPassword, setCurrent] = useState('')
  const [newPassword, setNew] = useState('')
  const toast = useToast()
  const change = useChangePassword()
  const error = change.isError ? formError(change.error, ['currentPassword', 'newPassword']) : null
  // 현재 비밀번호가 틀린 경우는 필드 아래에 보여 준다
  const wrong = change.error instanceof ApiRequestError && change.error.code === 'AUTH_WRONG_PASSWORD'
  return {
    values: { currentPassword, newPassword },
    set: { currentPassword: setCurrent, newPassword: setNew },
    pending: change.isPending,
    error: wrong && error ? { fields: { ...error.fields, currentPassword: error.message ?? '' }, message: undefined } : error,
    submit: prevent(() =>
      change.mutate(
        { currentPassword, newPassword },
        {
          onSuccess: () => {
            setCurrent('')
            setNew('')
            toast({ tone: 'success', message: '비밀번호를 바꿨어요. 다른 기기에서는 로그아웃돼요.' })
          },
        },
      ),
    ),
  }
}

/** 탈퇴 모달. 잔액이 있으면 소멸 확인 체크가 있어야 보낼 수 있다. 카카오 계정(`hasPassword` false)은 비밀번호 없이. 성공하면 홈으로. */
export function useWithdrawForm(balance: number, hasPassword: boolean) {
  const [open, setOpen] = useState(false)
  const [password, setPassword] = useState('')
  const [agreed, setAgreed] = useState(false)
  const qc = useQueryClient()
  const withdraw = useWithdraw()
  const code = withdraw.error instanceof ApiRequestError ? withdraw.error.code : null
  const error = withdraw.isError ? formError(withdraw.error, ['password']) : null
  const close = () => {
    setOpen(false)
    setPassword('')
    setAgreed(false)
    withdraw.reset()
  }
  return {
    open,
    show: () => setOpen(true),
    close,
    values: { password, agreed },
    set: { password: setPassword, agreed: setAgreed },
    needsAgree: balance > 0,
    canSubmit: (!hasPassword || password !== '') && (balance === 0 || agreed),
    pending: withdraw.isPending,
    // 비밀번호가 틀린 경우는 필드 아래, 테스트 계정 문구는 탈퇴 맥락에 맞게 바꿔 보여 준다
    passwordError: code === 'AUTH_WRONG_PASSWORD' ? '비밀번호가 올바르지 않아요.' : error?.fields.password,
    message: code === 'AUTH_WRONG_PASSWORD' ? undefined : code === 'AUTH_SEED_ACCOUNT_LOCKED' ? '테스트 계정은 탈퇴할 수 없어요.' : error?.message,
    submit: prevent(() =>
      withdraw.mutate(
        { password: hasPassword ? password : undefined, confirmForfeit: agreed },
        {
          // 그 사이 잔액이 바뀌었으면 최신 잔액으로 확인 체크를 다시 받는다
          onError: (e) => {
            if (e instanceof ApiRequestError && e.code === 'AUTH_WITHDRAW_CONFIRM_FORFEIT') void qc.invalidateQueries({ queryKey: ME_KEY })
          },
        },
      ),
    ),
  }
}
