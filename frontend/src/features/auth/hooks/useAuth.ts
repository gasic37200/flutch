import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { createContext, useContext, useEffect } from 'react'
import { useLocation, useNavigate } from 'react-router'
import { api, ApiRequestError } from '../../../api/client.ts'
import type { ChangePasswordRequest } from '../../../api/generated/ChangePasswordRequest.ts'
import type { KakaoPending } from '../../../api/generated/KakaoPending.ts'
import type { KakaoSignupRequest } from '../../../api/generated/KakaoSignupRequest.ts'
import type { LoginRequest } from '../../../api/generated/LoginRequest.ts'
import type { Me } from '../../../api/generated/Me.ts'
import type { SignupRequest } from '../../../api/generated/SignupRequest.ts'
import type { TestLoginRequest } from '../../../api/generated/TestLoginRequest.ts'
import type { WithdrawRequest } from '../../../api/generated/WithdrawRequest.ts'

export const ME_KEY = ['me']

/** 로그인 정보. 비로그인(401)이면 data는 null. 그 외 에러는 던진다. */
export function useMe() {
  return useQuery({
    queryKey: ME_KEY,
    queryFn: () =>
      api<Me>('/me').catch((e: unknown) => {
        if (e instanceof ApiRequestError && e.status === 401) return null
        throw e
      }),
    staleTime: 60_000,
    retry: false,
  })
}

export type AuthState = { me: Me | null; isPending: boolean }
export const AuthContext = createContext<AuthState>({ me: null, isPending: true })

/** 로그인 정보 Context. `me`가 null이면 비로그인, `isPending`이면 아직 확인 중. */
export const useAuth = () => useContext(AuthContext)

/** 로그인한 사용자가 바뀌면 이전 사용자 데이터가 남지 않게 `me` 외 캐시를 비우고 `me`를 채운다. */
function useSetMe() {
  const qc = useQueryClient()
  return (me: Me | null) => {
    qc.removeQueries({ predicate: (q) => q.queryKey[0] !== ME_KEY[0] })
    qc.setQueryData(ME_KEY, me)
  }
}

export const useLogin = () => {
  const setMe = useSetMe()
  return useMutation({ mutationFn: (json: LoginRequest) => api<Me>('/auth/login', { method: 'POST', json }), onSuccess: setMe })
}

export const useSignup = () => {
  const setMe = useSetMe()
  return useMutation({ mutationFn: (json: SignupRequest) => api<Me>('/auth/signup', { method: 'POST', json }), onSuccess: setMe })
}

/**
 * 카카오 로그인이 켜져 있는지. 새 API 없이 start를 한 번 불러 본다: 켜져 있으면 302(opaqueredirect, 따라가지 않음),
 * 설정이 없으면 404. 지금 이동하는 게 아니라 확인만 한다.
 */
export const useKakaoEnabled = () =>
  useQuery({
    queryKey: ['kakao-enabled'],
    queryFn: () =>
      fetch('/api/v1/auth/kakao/start', { redirect: 'manual' }).then(
        (res) => res.type === 'opaqueredirect',
        () => false,
      ),
    staleTime: Infinity,
    retry: false,
  })

/** 닉네임 설정 화면의 미리 채울 값. 임시 쿠키가 없거나 만료되면 401(isError). */
export const useKakaoPending = () =>
  useQuery({ queryKey: ['kakao-pending'], queryFn: () => api<KakaoPending>('/auth/kakao/signup'), staleTime: Infinity, retry: false })

export const useKakaoSignup = () => {
  const setMe = useSetMe()
  return useMutation({ mutationFn: (json: KakaoSignupRequest) => api<Me>('/auth/kakao/signup', { method: 'POST', json }), onSuccess: setMe })
}

export const useTestLogin = () => {
  const setMe = useSetMe()
  return useMutation({ mutationFn: (json: TestLoginRequest) => api<Me>('/auth/test-login', { method: 'POST', json }), onSuccess: setMe })
}

export const useLogout = () => {
  const setMe = useSetMe()
  return useMutation({ mutationFn: () => api<void>('/auth/logout', { method: 'POST' }), onSettled: () => setMe(null) })
}

/** 로그아웃하고 홈으로. 서버 호출이 실패해도 화면은 비로그인으로 돌린다. */
export function useLogoutToHome() {
  const logout = useLogout()
  const navigate = useNavigate()
  return () => logout.mutate(undefined, { onSettled: () => void navigate('/') })
}

export const useUpdateNickname = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (nickname: string) => api<Me>('/me', { method: 'PATCH', json: { nickname } }),
    onSuccess: (me) => qc.setQueryData(ME_KEY, me),
  })
}

export const useChangePassword = () =>
  useMutation({ mutationFn: (json: ChangePasswordRequest) => api<void>('/me/password', { method: 'PUT', json }) })

/** `?next=` 값이 같은 사이트 안의 경로일 때만 쓴다(오픈 리다이렉트 방지). */
export function safeNext(next: string | null): string {
  return next && next.startsWith('/') && !next.startsWith('//') && !next.startsWith('/\\') ? next : '/'
}

/** 로그인이 필요한 화면에서 호출. 비로그인이면 `/login?next=<현재 경로>`로 보내고, 로그인 후 돌아온다. */
export function useRequireLogin() {
  const auth = useAuth()
  const navigate = useNavigate()
  const { pathname, search } = useLocation()
  const needLogin = !auth.isPending && !auth.me
  useEffect(() => {
    if (needLogin) void navigate(`/login?next=${encodeURIComponent(pathname + search)}`, { replace: true })
  }, [needLogin, navigate, pathname, search])
  return auth
}

/** 회원 탈퇴. 성공하면 홈을 새로 불러 모든 캐시를 비운다(화면 안 이동은 `me`가 먼저 비면 로그인 화면으로 튕긴다). */
export const useWithdraw = () =>
  useMutation({ mutationFn: (json: WithdrawRequest) => api<void>('/me', { method: 'DELETE', json }), onSuccess: () => window.location.replace('/') })
