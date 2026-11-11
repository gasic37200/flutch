export type StudioEnv = {
  /** HTTPS(또는 localhost)가 아니면 카메라·화면 공유를 쓸 수 없다 */
  secure: boolean
  canScreen: boolean
  /** 휴대폰·태블릿: 카메라 고정 송출 화면을 쓴다 */
  mobile: boolean
  /** 전·후면 카메라 전환 가능(모바일 + facingMode 지원) */
  canFlip: boolean
  /** 시스템 소리: Windows Chrome/Edge만 */
  systemAudio: boolean
  /** 탭 소리: 데스크톱 Chrome/Edge */
  tabAudio: boolean
  /** 화면 아래에 보여 줄 안내(비어 있으면 모두 지원) */
  notices: string[]
}

/** 브라우저 감지는 UA 문자열만 본다. shortcut: Chromium 계열 판별이 거칠다, 지원 표를 넓힐 때 기능 검사로 교체. */
export function detectEnv(): StudioEnv {
  const ua = navigator.userAgent
  const mobile = /Android|iPhone|iPad|Mobile/i.test(ua)
  const chromium = /Chrome\/|Edg\//.test(ua) && !/OPR\/|Firefox\//.test(ua)
  const secure = window.isSecureContext && !!navigator.mediaDevices
  const canScreen = secure && !mobile && typeof navigator.mediaDevices.getDisplayMedia === 'function'
  const canFlip = secure && mobile && !!navigator.mediaDevices.getSupportedConstraints?.().facingMode
  const tabAudio = canScreen && chromium
  const systemAudio = tabAudio && /Windows/.test(ua)

  const notices: string[] = []
  if (!secure) notices.push('HTTPS 주소에서만 화면 공유와 카메라를 쓸 수 있어요. 주소가 https://로 시작하는지 확인해 주세요.')
  else if (mobile) {
    // 모바일은 카메라 송출이 기본이라 화면 공유 불가 안내를 띄우지 않는다
  } else if (!canScreen) notices.push('이 기기에서는 화면 공유를 쓸 수 없어요. 카메라 모드로 방송해 주세요.')
  else if (!tabAudio) notices.push('이 브라우저에서는 화면 소리를 보낼 수 없어요. 소리까지 보내려면 PC의 Chrome 또는 Edge를 써 주세요.')
  else if (!systemAudio) notices.push('이 환경에서는 탭 소리만 송출할 수 있어요. 시스템 소리는 Windows의 Chrome·Edge에서만 지원돼요.')
  return { secure, canScreen, mobile, canFlip, systemAudio, tabAudio, notices }
}
