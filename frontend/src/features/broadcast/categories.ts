import type { Category } from '../../api/generated/Category.ts'

// 카테고리 코드 → 화면 라벨 (docs/api.md §1)
export const CATEGORY_LABEL: Record<Category, string> = {
  just_chatting: '저스트채팅',
  game: '종합게임',
  fps: 'FPS',
  rpg: 'RPG',
  horror: '공포게임',
  virtual: '버추얼',
  sports: '스포츠',
  music: '음악',
}

export const CATEGORIES = Object.keys(CATEGORY_LABEL) as Category[]

export const isCategory = (v: string | null): v is Category => v !== null && v in CATEGORY_LABEL
