import { ApiRequestError } from '../../../api/client.ts'

/** 화면에 보일 에러 문구. 서버 문구를 그대로, 네트워크 끊김은 공통 문구. */
export const errorText = (e: unknown) => (e instanceof ApiRequestError ? e.message : '네트워크 연결을 확인하고 다시 시도해 주세요.')
